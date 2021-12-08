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

use warp::Filter;

// Dependencies for hash string converter
use sp_core::sr25519::Public;

// TEST, should remove after test
#[derive(Deserialize, Serialize)]
struct Employee {
    name: String,
    rate: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Sample {
  person_id: i32,
  person_name: String
}

#[derive(Encode, Decode, Clone, Default, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct AuditLog {
    // Change the timestamp to a timestamp type handled by Substrate itself
    // Reporter determines which system sent the log
    content: String,
    reporter: Public,
}

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let ping_chain = warp::path!("ping-chain")
        //.map(|name| format!("Hello, {}!", name));
        .map(|| ping_chain());

    let get_balances = warp::path!("get-balances")
    .map(|| get_balances());

    let get_log_storage = warp::path!("get-log-storage")
    .map(|| get_log_storage());

    let add_log = warp::path!("add-log" / String)
        .map(|log_content| 
          add_log(log_content)
        );

    let routes = warp::get().and(
      ping_chain
          .or(get_balances)
          .or(get_log_storage)
          .or(add_log),
  );

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
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

fn get_balances() -> std::string::String {
  // instantiate an Api that connects to the given address
  let url = "ws://127.0.0.1:9944";
  
  
  let client = WsRpcClient::new(url);
  let api = Api::<sr25519::Pair, _>::new(client).unwrap();

  let result: u128 = api
    .get_storage_value("Balances", "TotalIssuance", None)
    .unwrap()
    .unwrap();

  format!("[+] TotalIssuance is {}", result)
}

fn get_log_storage() -> std::string::String {
  // instantiate an Api that connects to the given address
  let url = "ws://127.0.0.1:9944";
  
  
  let client = WsRpcClient::new(url);
  let api = Api::<sr25519::Pair, _>::new(client).unwrap();

  let result: AuditLog = api
        .get_storage_double_map("Auditor", "AuditLogStorage", "test_filename2".to_string().into_bytes(), "test_timestamp3".to_string().into_bytes(), None)
        .unwrap()
        .or_else(|| Some(AuditLog::default()))
        .unwrap();
        
  format!("[+] log items are {:?}", result)
}

fn add_log(log_content: String) -> std::string::String {

  let url = "ws://127.0.0.1:9944";
  
  let client = WsRpcClient::new(url);
  let from = AccountKeyring::Alice.pair();
  let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();

  // NOTE: save_audit_log exists in Auditor pallet thats why this works
  #[allow(clippy::redundant_clone)]
  let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
    api.clone(),
    "Auditor",
    "save_audit_log",
    "test_filename2".to_string().into_bytes(),
    log_content.to_string().into_bytes(),
    "test_timestamp3".to_string().into_bytes()
  );

  
  println!("[+] Composed Extrinsic:\n {:?}\n", xt);
  
  // send and watch extrinsic until finalized
  let blockh = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock).unwrap();

  format!("[+] Transaction got included in block {:?}", blockh)
}