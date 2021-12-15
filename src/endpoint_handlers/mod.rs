use sp_core::sr25519;
use std::convert::TryFrom;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, Metadata, compose_extrinsic, UncheckedExtrinsicV4, XtStatus};
use sp_core::crypto::Pair;
use sp_keyring::AccountKeyring;
use warp::{http, Filter};
use chrono::{DateTime, Local};
use chrono::prelude::*;
use std::str;

mod models;

// TODO: Move this to a yml config file
// instantiate an Api that connects to the given address
static URL: &str = "ws://127.0.0.1:9944";

pub fn json_body() -> impl Filter<Extract = (models::IncomingAuditLog,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    println!("json_body called");
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

/*
// TODO: Check if can be removed
pub async fn get_file_logs_from_date_range(log_filename: String, log_date_begin: String, log_date_end: String) -> Result<impl warp::Reply, warp::Rejection> {
  let result: Vec<models::AuditLog> = collect_file_logs_from_timestamp_range(&log_filename, &log_date_begin, &log_date_end);
  let result_summary = models::AuditLogSummary {
    filename: log_filename,
    date: log_date_begin,
    contents: result
  };
  println!("{:?}", serde_json::to_string(&result_summary).unwrap());
  Ok(warp::reply::json(
    &result_summary
  ))
}*/

// NOTE: This currently has O(n) so it needs refactoring for performance at some point
// NOTE: This function can be in a utility class
// TODO: Must deal with nanoseconds items
// TODO: Check if can be removed
/*
fn collect_file_logs_from_timestamp_range(log_filename: &String, log_date_begin: &String, log_date_end: &String) -> Vec<models::AuditLog> {

  let str_date_begin = DateTime::parse_from_rfc3339(&log_date_begin[..]).unwrap();
  let str_date_end = DateTime::parse_from_rfc3339(&log_date_end[..]).unwrap();
  let date_diff: i64 = str_date_end.signed_duration_since(str_date_begin).num_seconds();

  let mut time_seconds = 0;
  let mut audit_logs: Vec<models::AuditLog> = Vec::new();

while time_seconds < date_diff {
    let adjusted_log_date = str_date_begin + Duration::seconds(time_seconds);
    println!("date_time of retrieval {}", adjusted_log_date);
    let result = retrieve_log_from(&log_filename, &adjusted_log_date.to_rfc3339().to_string());

    if !&result.content.trim().is_empty() {
      println!("adding audit_log result {:?}", result);
      audit_logs.push(result);
    }
    time_seconds = time_seconds + 1;
  }

  audit_logs
}*/

// NOTE: This function can be in a utility class
// TODO: Check if can be removed
/*
fn retrieve_log_from(log_filename: &String, log_date: &String) -> Vec<models::AuditLog> {
  let client = WsRpcClient::new(URL);
  let api = Api::<sr25519::Pair, _>::new(client).unwrap();
  api.get_storage_double_map("Auditor", "AuditLogStorage", log_filename.to_string().into_bytes(), log_date.to_string().into_bytes(), None)
        .unwrap()
        .or_else(|| Some(Vec::default()))
        .unwrap()
}*/

pub async fn save_log(incoming_audit_log: models::IncomingAuditLog) -> Result<impl warp::Reply, warp::Rejection> {
  let client = WsRpcClient::new(URL);
  let from = AccountKeyring::Alice.pair();
  let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();

  let now: DateTime<Local> = Local::now();
  //println!("date_time of added_log {}", &now.to_rfc3339().to_string()[..]);

  let mut datetime = now.clone().to_rfc3339().to_string();
  // Remove time items and leave only the date of format YYYY-MM-DD
  let datetime_trails = datetime.split_off(10);
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