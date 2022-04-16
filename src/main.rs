#![feature(decl_macro)]
extern crate futures;
extern crate hyper;
extern crate openssl;

use warp::Filter;
use crate::services::json_body_handlers::open_log_for_claim_body;
use crate::services::json_body_handlers::claim_log_body;
use crate::services::json_body_handlers::incoming_audit_log_body;
use crate::services::route_actions::claim_log_for_ownership;
use crate::services::route_actions::open_log_for_ownership_claim;
use crate::services::route_actions::save_log;
use crate::services::route_actions::get_file_logs_from_date;
use crate::services::route_actions::ping_chain;

mod services;

#[tokio::main]
async fn main() {
    let ping_chain = warp::path!("v1" / "ping-chain")
        .map(|| ping_chain());

    /* 
      USAGE: http://127.0.0.1:3030/v1/logs/test_log_file1/2021-12-14
      Timestamp (UTC) format example: 2021-12-14
    */
    let get_logs_with_filename_and_date = warp::path!("v1" / "logs" / String / String)
      .and_then(get_file_logs_from_date);
      
    /*
      CURL SAMPLE USAGE: curl -X POST 127.0.0.1:3030/v1/logs -H 'Content-Type: application/json' -d '{"filename":"test_log_file1","title":"test_four_title","content":"content4"}'
     */
    let save_log = warp::post()
      .and(warp::path("v1"))
      .and(warp::path("logs"))
      .and(warp::path::end())
      .and(incoming_audit_log_body())
      .and_then(save_log);

    /*
      CURL SAMPLE USAGE: curl -X POST 127.0.0.1:3030/v1/log_ownership/open -H 'Content-Type: application/json' -d '{"filename":"test_log_file1","claimer_pubkey":"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"}'
     */
    let open_log_for_ownership_claim = warp::post()
      .and(warp::path("v1"))
      .and(warp::path("log_ownership"))
      .and(warp::path("open"))
      .and(warp::path::end())
      .and(open_log_for_claim_body())
      .and_then(open_log_for_ownership_claim);

    /*
      CURL SAMPLE USAGE: curl -X POST 127.0.0.1:3030/v1/log_ownership/claim -H 'Content-Type: application/json' -d '{"filename":"test_log_file1"}'
     */
    let claim_log_for_ownership = warp::post()
      .and(warp::path("v1"))
      .and(warp::path("log_ownership"))
      .and(warp::path("claim"))
      .and(warp::path::end())
      .and(claim_log_body())
      .and_then(claim_log_for_ownership);

    /*
      CURL SAMPLE USAGE: curl -X POST 127.0.0.1:3030/v1/participants -H 'Content-Type: application/json' -d '{"validator_id":"0x16b215a5fd8b8caa03e75313e78c8ee344ba6ee7c6c2d6ce0a0a2e4e3a0d2377f775fc2682db3bb79376a61e773a87cddd1e5e5935cdea8a9ab3a54be011a62b"}'
     */
    // NOTE: THIS add_validator function is not yet functional
    /*
    let add_validator = warp::post()
      .and(warp::path("v1"))
      .and(warp::path("participants"))
      .and(warp::path::end())
      .and(endpoint_handlers::validator_body())
      .and_then(endpoint_handlers::add_validator);*/

    let cors = warp::cors()
      .allow_any_origin()
      .allow_headers(vec!["User-Agent", "Sec-Fetch-Mode", "Referer", "Origin", "Access-Control-Request-Method", "Access-Control-Request-Headers", "content-type"])
      .allow_methods(vec!["POST", "GET"]);

    let routes = ping_chain
      .or(get_logs_with_filename_and_date)
      .or(save_log)
      .or(open_log_for_ownership_claim)
      .or(claim_log_for_ownership)
      //.or(add_validator)
      .with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
