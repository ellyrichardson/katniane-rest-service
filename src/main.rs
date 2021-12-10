#![feature(decl_macro)]
extern crate futures;
extern crate hyper;

use serde::{Serialize, Deserialize};

use sp_core::sr25519;

use std::convert::TryFrom;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, Metadata, compose_extrinsic, UncheckedExtrinsicV4, XtStatus};

use codec::{Decode, Encode};
use sp_core::crypto::Pair;
use sp_keyring::AccountKeyring;

use warp::{http, Filter};

// Dependencies for hash string converter
use sp_core::sr25519::Public;

use chrono::{DateTime, Utc, Duration};
use chrono::prelude::*;

#[derive(Encode, Decode, Clone, Default, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct AuditLog {
    // Reporter determines which system sent the log
    content: String,
    reporter: Public,
}

#[derive(Encode, Decode, Clone, Default, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct AuditLogSummary {
    // Reporter determines which system sent the log
    filename: String,
    contents: Vec<AuditLog>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct IncomingAuditLog {
    filename: String,
    content: String,
}

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let ping_chain = warp::path!("v1" / "ping-chain")
        .map(|| ping_chain());


    let get_logs_with_filename_and_timestamp = warp::path!("v1" / "logs" / String / String)
      .map(|log_filename, log_timestamp| 
        get_file_logs_from_timestamp(log_filename, log_timestamp)
      );

    let save_log = warp::post()
      .and(warp::path("v1"))
      .and(warp::path("logs"))
      .and(warp::path::end())
      .and(json_body())
      .and_then(save_log);

  let routes = ping_chain
    .or(get_logs_with_filename_and_timestamp)
    .or(save_log);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

// TODO: Place in its own file
fn json_body() -> impl Filter<Extract = (IncomingAuditLog,), Error = warp::Rejection> + Clone {
  // When accepting a body, we want a JSON body
  // (and to reject huge payloads)...
  println!("json_body called");
  warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn ping_chain() -> std::string::String {
  // instantiate an Api that connects to the given address
  let url = "ws://127.0.0.1:9944";
  
  
  let client = WsRpcClient::new(url);
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

// TODO: The output in the Response is quite dirty, need to clean it up
fn get_file_logs_from_timestamp(log_filename: String, log_timestamp: String) -> std::string::String {
  let result: Vec<AuditLog> = collect_file_logs_from_timestamp_range(&log_filename, &log_timestamp, &Utc::now().to_rfc3339());
  let result_summary = AuditLogSummary {
    filename: log_filename,
    contents: result
  };
  format!("{:?}", serde_json::to_string(&result_summary).unwrap())
}

// NOTE: This currently has O(n) so it needs refactoring for performance at some point
// NOTE: This function can be in a utility class
// TODO: Must deal with nanoseconds items
fn collect_file_logs_from_timestamp_range(log_filename: &String, log_timestamp_begin: &String, log_timestamp_end: &String) -> Vec<AuditLog> {

  let str_timestmp_begin = DateTime::parse_from_rfc3339(&log_timestamp_begin[..]).unwrap();
  let str_timestmp_end = DateTime::parse_from_rfc3339(&log_timestamp_end[..]).unwrap();
  let timestamp_diff: i64 = str_timestmp_end.signed_duration_since(str_timestmp_begin).num_seconds();

  let mut time_seconds = 0;
  let mut audit_logs: Vec<AuditLog> = Vec::new();

  while time_seconds < timestamp_diff {
    let adjusted_log_timestamp = str_timestmp_begin + Duration::seconds(time_seconds);
    println!("date_time of retrieval {}", adjusted_log_timestamp);
    let result = retrieve_log_from(&log_filename, &adjusted_log_timestamp.to_rfc3339().to_string());

    if !&result.content.trim().is_empty() {
      println!("adding audit_log result {:?}", result);
      audit_logs.push(result);
    }
    time_seconds = time_seconds + 1;
  }

  audit_logs
}

// NOTE: This function can be in a utility class
fn retrieve_log_from(log_filename: &String, log_timestamp: &String) -> AuditLog {
  let url = "ws://127.0.0.1:9944";
  
  let client = WsRpcClient::new(url);
  let api = Api::<sr25519::Pair, _>::new(client).unwrap();
  api.get_storage_double_map("Auditor", "AuditLogStorage", log_filename.to_string().into_bytes(), log_timestamp.to_string().into_bytes(), None)
        .unwrap()
        .or_else(|| Some(AuditLog::default()))
        .unwrap()
}

async fn save_log(incoming_audit_log: IncomingAuditLog) -> Result<impl warp::Reply, warp::Rejection> {

  let url = "ws://127.0.0.1:9944";
  
  let client = WsRpcClient::new(url);
  let from = AccountKeyring::Alice.pair();
  let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();

  let now: DateTime<Utc> = Utc::now().round_subsecs(0);
  println!("date_time of added_log {}", &now.to_rfc3339().to_string());

  // NOTE: save_audit_log exists in Auditor pallet thats why this works
  #[allow(clippy::redundant_clone)]
  let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
    api.clone(),
    "Auditor",
    "save_audit_log",
    incoming_audit_log.filename.to_string().into_bytes(),
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
