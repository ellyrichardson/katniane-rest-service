extern crate toml;

use sp_core::sr25519;
use sp_core::ed25519::{self, Public};
// use sp_core::ed25519::Public::FromStr;
use std::convert::{TryFrom, TryInto};
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, Metadata, compose_extrinsic, UncheckedExtrinsicV4, XtStatus};
use sp_core::crypto::{Pair, Ss58Codec};
use sp_keyring::AccountKeyring;
use warp::{http, Filter};
use chrono::{DateTime, Local};
use std::str;
use std::fs::{File, OpenOptions};
use std::io::Read;
use toml::Value;

use std::env;

mod utilities;
mod models;

// TODO: Move JSON bodies in it's own file
pub fn log_body() -> impl Filter<Extract = (models::IncomingAuditLog,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    println!("json log_body called");
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

// TODO: Move JSON bodies in it's own file
pub fn validator_body() -> impl Filter<Extract = (models::ValidatorAccount,), Error = warp::Rejection> + Clone {
  // When accepting a body, we want a JSON body
  // (and to reject huge payloads)...
  println!("json validator_body called");
  warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub fn open_log_for_claim_body() -> impl Filter<Extract = (models::AuditLogToBeOpenedForClaiming,), Error = warp::Rejection> + Clone {
  // When accepting a body, we want a JSON body
  // (and to reject huge payloads)...
  println!("json validator_body called");
  warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub fn claim_log_body() -> impl Filter<Extract = (models::AuditLogToBeClaimed,), Error = warp::Rejection> + Clone {
  // When accepting a body, we want a JSON body
  // (and to reject huge payloads)...
  println!("json validator_body called");
  warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
  
pub fn ping_chain() -> std::string::String {

  let config = utilities::load_service_config();
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

  let config = utilities::load_service_config();
  let chain_ws_url = format!("ws://{}:{}", &config.katniane_chain_address, &config.katniane_chain_port);

  let client = WsRpcClient::new(&chain_ws_url);

  //let client = WsRpcClient::new(URL);
  let api = Api::<sr25519::Pair, _>::new(client).unwrap();
  let result: Vec<models::AuditLog> = api.get_storage_double_map("Auditor", "AuditLogStorage", &log_filename.to_string().into_bytes(), &log_date.to_string().into_bytes(), None)
    .unwrap()
    .or_else(|| Some(Vec::default()))
    .unwrap();

  let result_summary = models::AuditLogSummary {
    filename: log_filename,
    date: log_date,
    contents: result
  };

  println!("{:?}", serde_json::to_string(&result_summary).unwrap());
  Ok(warp::reply::json(
    &result_summary
  ))
}

pub async fn save_log(incoming_audit_log: models::IncomingAuditLog) -> Result<impl warp::Reply, warp::Rejection> {

  let config = utilities::load_service_config();
  let chain_ws_url = format!("ws://{}:{}", &config.katniane_chain_address, &config.katniane_chain_port);

  let client = WsRpcClient::new(&chain_ws_url);

  let from = AccountKeyring::Bob.pair();
  let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();

  let now: DateTime<Local> = Local::now();

  let mut datetime = now.clone().to_rfc3339().to_string();

  // Remove time items and leave only the date of format YYYY-MM-DD
  datetime.split_off(10);

  println!("date_time of added_log {}", &datetime);

  // NOTE: save_audit_log exists in Auditor pallet thats why this works
  #[allow(clippy::redundant_clone)]
  let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
    api.clone(),
    "Auditor",
    "save_audit_log",
    incoming_audit_log.filename.to_string().into_bytes(),
    datetime.to_string().into_bytes(),
    incoming_audit_log.title.to_string().into_bytes(),
    incoming_audit_log.content.to_string().into_bytes(),
    now.to_rfc3339().to_string().into_bytes()
  );

  
  println!("[+] Composed Extrinsic:\n {:?}\n", xt);
  
  // send and watch extrinsic until finalized
  let blockh = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock).unwrap();

  println!("[+] Transaction got included in block {:?}", blockh);

  Ok(warp::reply::with_status(
    "Added logs to blockchain",
    http::StatusCode::CREATED,
  ))
}

/*
fn vec_to_fixed_size<T, const N: usize>(vec_item: Vec<T>) -> [u8; 32] {
  vec_item.try_into()
      .unwrap_or_else(|vec_item: Vec<T>| panic!("Expected a Vec of length {} but it was {}", 32, vec_item.len()))
}*/

fn vec_to_fixed_size<T>(v: Vec<T>) -> [T; 32] {
  let boxed_slice = v.into_boxed_slice();
  let boxed_array: Box<[T; 32]> = match boxed_slice.try_into() {
      Ok(ba) => ba,
      Err(o) => panic!("Account Public Key is not of length {} but it was {}", 32, o.len()),
  };
  *boxed_array
}


pub async fn open_log_for_ownership_claim(log_to_be_opened: models::AuditLogToBeOpenedForClaiming) -> Result<impl warp::Reply, warp::Rejection> {

  let config = utilities::load_service_config();
  let chain_ws_url = format!("ws://{}:{}", &config.katniane_chain_address, &config.katniane_chain_port);

  let client = WsRpcClient::new(&chain_ws_url);

  // Get the private key of the sender here
  let from = AccountKeyring::Bob.pair();
  let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();

  println!("[+] Test of outputting Charlie key:\n {:?}\n", AccountKeyring::Charlie.to_raw_public_vec());

  let now: DateTime<Local> = Local::now();

  // NOTE: datetime is currently not used right now
  let mut datetime = now.clone().to_rfc3339().to_string();

  // Remove time items and leave only the date of format YYYY-MM-DD
  datetime.split_off(10);

  println!("date_time of added_log {}", &datetime);

  // Convert claimer from String to Vec<u8>
  //let open_log_claimer = log_to_be_opened.claimer_pubkey.as_bytes().to_vec();

  // TODO: Fix this
  // let open_log_claimer = ed25519::Public::from_ss58check(&log_to_be_opened.claimer_pubkey);
  match ed25519::Public::from_ss58check(&log_to_be_opened.claimer_pubkey) {
    Ok(olc_res) => {

        println!("[+] Test of outputting Charlie key2 :\n {:?}\n", &olc_res.as_array_ref());

      // Compose the extrinsic
      #[allow(clippy::redundant_clone)]
      let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        api.clone(),
        "Auditor",
        "open_log_for_ownership_claim",
        log_to_be_opened.filename.to_string().into_bytes(),
        olc_res.as_array_ref()
        // TODO: maybe get the date here
      );

      
      println!("[+] Composed Extrinsic:\n {:?}\n", xt);
      
      // send and watch extrinsic until finalized
      let blockh = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock).unwrap();

      println!("[+] Transaction got included in block {:?}", blockh);
    },
    Err(error) => {
        println!("[-] Error encountered while processing claimer pubkey {:?}", error);
    }
  }

  Ok(warp::reply::with_status(
    "Added logs to blockchain",
    http::StatusCode::CREATED,
  ))
}

pub async fn claim_log_for_ownership(log_to_be_claimed: models::AuditLogToBeClaimed) -> Result<impl warp::Reply, warp::Rejection> {

  let config = utilities::load_service_config();
  let chain_ws_url = format!("ws://{}:{}", &config.katniane_chain_address, &config.katniane_chain_port);

  let client = WsRpcClient::new(&chain_ws_url);

  // Get the private key of the sender here
  let from = AccountKeyring::Charlie.pair();
  let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();

  let now: DateTime<Local> = Local::now();

  // NOTE: datetime is currently not used right now
  let mut datetime = now.clone().to_rfc3339().to_string();

  // Remove time items and leave only the date of format YYYY-MM-DD
  datetime.split_off(10);
  println!("date_time of added_log {}", &datetime);

  // Compose the extrinsic
  #[allow(clippy::redundant_clone)]
  let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
    api.clone(),
    "Auditor",
    "claim_log",
    log_to_be_claimed.filename.to_string().into_bytes()
    // TODO: maybe get the date here
  );

  
  println!("[+] Composed Extrinsic:\n {:?}\n", xt);
  
  // send and watch extrinsic until finalized
  let blockh = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock).unwrap();

  println!("[+] Transaction got included in block {:?}", blockh);

  Ok(warp::reply::with_status(
    "Added logs to blockchain",
    http::StatusCode::CREATED,
  ))
}

// NOTE: This add_validator() is still not used anywhere
/*pub async fn add_validator(validator_account: models::ValidatorAccount) -> Result<impl warp::Reply, warp::Rejection> {
  let chain_ws_url = format!("ws://{}:{}", utilities::load_service_config().katniane_chain_address, utilities::load_service_config().katniane_chain_port);

  let client = WsRpcClient::new(&chain_ws_url);
  let from = AccountKeyring::Alice.pair();
  let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();

  //let x: String = validator_account_id;

  let setkey_tx: UncheckedExtrinsicV4<_> = compose_extrinsic!(
    api.clone(),
    "Session",
    "set_keys",
    validator_account.clone().validator_id,
    "0x".to_string().into_bytes()
  );

  println!("[+] validator id: :\n {:?}\n", validator_account.clone().validator_id);
  println!("[+] Composed Extrinsic:\n {:?}\n", setkey_tx);
  
  // send and watch extrinsic until finalized
  let blockh1 = api.clone().send_extrinsic(setkey_tx.hex_encode(), XtStatus::InBlock).unwrap();

  println!("[+] Transaction got included in block {:?}", blockh1);

  // NOTE: save_audit_log exists in Auditor pallet thats why this works
  #[allow(clippy::redundant_clone)]
  let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
    api.clone(),
    "ValidatorSet",
    "add_validator",
    validator_account.validator_id
  );
  
  println!("[+] Composed Extrinsic:\n {:?}\n", xt);
  
  // send and watch extrinsic until finalized
  let blockh = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock).unwrap();

  println!("[+] Transaction got included in block {:?}", blockh);

  Ok(warp::reply::with_status(
    "Added logs to blockchain",
    http::StatusCode::CREATED,
  ))
}*/