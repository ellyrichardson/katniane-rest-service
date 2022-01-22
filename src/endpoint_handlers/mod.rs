use sp_core::sr25519;
use std::convert::TryFrom;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, Metadata, compose_extrinsic, UncheckedExtrinsicV4, XtStatus};
use sp_core::crypto::Pair;
use sp_keyring::AccountKeyring;
use warp::{http, Filter};
use chrono::{DateTime, Local};
use std::str;

mod models;

// TODO: Move this to a yml config file
// instantiate an Api that connects to the given address
static URL: &str = "ws://127.0.0.1:9945";

pub fn log_body() -> impl Filter<Extract = (models::IncomingAuditLog,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    println!("json log_body called");
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub fn validator_body() -> impl Filter<Extract = (models::ValidatorAccount,), Error = warp::Rejection> + Clone {
  // When accepting a body, we want a JSON body
  // (and to reject huge payloads)...
  println!("json validator_body called");
  warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
  
pub fn ping_chain() -> std::string::String {
  let client = WsRpcClient::new(URL);
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
  //let result: Vec<models::AuditLog> = collect_file_logs_from_timestamp_range(&log_filename, &log_date, &Utc::now().to_rfc3339());

  let client = WsRpcClient::new(URL);
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
  let client = WsRpcClient::new(URL);
  let from = AccountKeyring::Alice.pair();
  let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();

  let now: DateTime<Local> = Local::now();
  //println!("date_time of added_log {}", &now.to_rfc3339().to_string()[..]);

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

pub async fn add_validator(validator_account: models::ValidatorAccount) -> Result<impl warp::Reply, warp::Rejection> {
  let client = WsRpcClient::new(URL);
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
}