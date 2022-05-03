extern crate toml;

//use sp_core::{sr25519::Pair, Pair as TraitPair};
use sp_core::{sr25519, Pair};
use std::convert::TryFrom;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, Metadata};
use sp_keyring::AccountKeyring;
use warp::http;
use chrono::{DateTime, Local};
use katniane_rest::models::{AuditLogToBeOpenedForClaiming, IncomingAuditLog, AuditLogToBeClaimed, AuditLog, AuditLogSummary};
use katniane_rest::utilities::load_service_config;

mod extrinsic_submitters;
  
pub fn ping_chain() -> std::string::String {

  let config = load_service_config();
  let chain_ws_url = format!("ws://{}:{}", &config.katniane_chain_address, &config.katniane_chain_port);

  let client = WsRpcClient::new(&chain_ws_url);
  let api = Api::<sr25519::Pair, _>::new(client).unwrap();
  let meta = Metadata::try_from(api.get_metadata().unwrap()).unwrap();

  meta.print_overview();
  meta.print_pallets();
  meta.print_pallets_with_calls();
  meta.print_pallets_with_events();
  meta.print_pallets_with_errors();
  meta.print_pallets_with_constants();
  println!("ping-chain successful");

  Metadata::pretty_format(&api.get_metadata().unwrap())
          .unwrap_or_else(|| "pretty format failed".to_string())
}

pub async fn get_file_logs_from_date(log_filename: String, log_date: String) -> Result<impl warp::Reply, warp::Rejection> {

  let config = load_service_config();
  let chain_ws_url = format!("ws://{}:{}", &config.katniane_chain_address, &config.katniane_chain_port);

  let client = WsRpcClient::new(&chain_ws_url);

  //let client = WsRpcClient::new(URL);
  let api = Api::<sr25519::Pair, _>::new(client).unwrap();
  let result: Vec<AuditLog> = api.get_storage_double_map("Auditor", "AuditLogStorage", &log_filename.to_string().into_bytes(), &log_date.to_string().into_bytes(), None)
    .unwrap()
    .or_else(|| Some(Vec::default()))
    .unwrap();

  let result_summary = AuditLogSummary {
    filename: log_filename,
    date: log_date,
    contents: result
  };

  println!("{:?}", serde_json::to_string(&result_summary).unwrap());
  Ok(warp::reply::json(
    &result_summary
  ))
}

pub async fn save_log(incoming_audit_log: IncomingAuditLog) -> Result<impl warp::Reply, warp::Rejection> {

  let config = load_service_config();
  let chain_ws_url = format!("ws://{}:{}", &config.katniane_chain_address, &config.katniane_chain_port);

  let client = WsRpcClient::new(&chain_ws_url);

  // Get the private key of the sender here
  let priv_key = &config.private_key;
  let piv_key_pass: Option<&str> = Some(&config.private_key_password);
  let from = Pair::from_string(&priv_key[..], piv_key_pass).unwrap();
  let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();

  let now: DateTime<Local> = Local::now();

  let mut datetime = now.clone().to_rfc3339().to_string();

  // Remove time items and leave only the date of format YYYY-MM-DD
  datetime.split_off(10);

  println!("[*] date_time of added_log: {}", &datetime);

  // NOTE: save_audit_log exists in Auditor pallet thats why this works
  extrinsic_submitters::submit_to_save_log(api, incoming_audit_log, now, datetime);

  Ok(warp::reply::with_status(
    "Added logs to blockchain",
    http::StatusCode::CREATED,
  ))
}

pub async fn open_log_for_ownership_claim(log_to_be_opened: AuditLogToBeOpenedForClaiming) -> Result<impl warp::Reply, warp::Rejection> {

  let config = load_service_config();
  let chain_ws_url = format!("ws://{}:{}", &config.katniane_chain_address, &config.katniane_chain_port);

  let client = WsRpcClient::new(&chain_ws_url);

  // Get the private key of the sender here
  let priv_key = &config.private_key;
  let piv_key_pass: Option<&str> = Some(&config.private_key_password);
  let from = Pair::from_string(&priv_key[..], piv_key_pass).unwrap();
  let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();

  // TODO: Put this whole match statement in its own function
  extrinsic_submitters::submit_to_open_log(api, log_to_be_opened);

  Ok(warp::reply::with_status(
    "Opened a log for claiming",
    http::StatusCode::CREATED,
  ))
}

pub async fn claim_log_for_ownership(log_to_be_claimed: AuditLogToBeClaimed) -> Result<impl warp::Reply, warp::Rejection> {

  let config = load_service_config();
  let chain_ws_url = format!("ws://{}:{}", &config.katniane_chain_address, &config.katniane_chain_port);

  let client = WsRpcClient::new(&chain_ws_url);

  // Get the private key of the sender here
  let priv_key = &config.private_key;
  let piv_key_pass: Option<&str> = Some(&config.private_key_password);
  let from = Pair::from_string(&priv_key[..], piv_key_pass).unwrap();
  let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();

  extrinsic_submitters::submit_to_claim_log(api, log_to_be_claimed);

  Ok(warp::reply::with_status(
    "Opened log is claimed",
    http::StatusCode::CREATED,
  ))
}