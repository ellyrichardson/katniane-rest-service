#![feature(decl_macro)]
extern crate futures;
extern crate hyper;

use warp::Filter;

mod endpoint_handlers;

#[tokio::main]
async fn main() {
    let ping_chain = warp::path!("v1" / "ping-chain")
        .map(|| endpoint_handlers::ping_chain());

    /* 
      USAGE: <address>/v1/logs/<filename>/<timestamp_begin>
      Timestamp (UTC) format example: 2021-12-10T02:53:00+00:00
    */
    let get_logs_with_filename_and_date = warp::path!("v1" / "logs" / String / String)
      .and_then(endpoint_handlers::get_file_logs_from_date);
      
    /*
      CURL SAMPLE USAGE: curl -X POST 127.0.0.1:3030/v1/logs -H 'Content-Type: application/json' -d '{"filename":"test_log_file1","title":"test_four_title","content":"content4"}'
     */
    let save_log = warp::post()
      .and(warp::path("v1"))
      .and(warp::path("logs"))
      .and(warp::path::end())
      .and(endpoint_handlers::json_body())
      .and_then(endpoint_handlers::save_log);

    /*
      CURL SAMPLE USAGE: curl -X POST 127.0.0.1:3030/v1/participants -H 'Content-Type: application/json' -d '{"validator_id":"0x1a4cc824f6585859851f818e71ac63cf6fdc81018189809814677b2a4699cf45"}'
     */
    let add_validator = warp::post()
      .and(warp::path("v1"))
      .and(warp::path("participants"))
      .and(warp::path::end())
      .and(endpoint_handlers::validator_json_body())
      .and_then(endpoint_handlers::add_validator);

  let routes = ping_chain
    .or(get_logs_with_filename_and_date)
    .or(save_log)
    .or(add_validator);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
