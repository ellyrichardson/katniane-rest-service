#![feature(decl_macro)]
#[macro_use] extern crate rocket;
extern crate futures;
extern crate hyper;

use futures::executor::block_on;
use rocket::response::content::Json;
use serde::{Serialize, Deserialize};
use reqwest::header::CONTENT_TYPE;

use sp_core::sr25519;

use std::convert::TryFrom;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, Metadata};

use codec::{Decode, Encode};
use sp_core::crypto::Pair;
use sp_keyring::AccountKeyring;

#[derive(Serialize, Deserialize, Debug)]
struct Sample {
  person_id: i32,
  person_name: String
}

#[launch]
#[tokio::main]
async fn rocket() -> _ {
  rocket::build().mount("/api", routes![pingchain, pingchaintwo, hello])
}

#[get("/hello")]
fn hello() -> Json<&'static str> {
  println!("hello called");
  Json("{
    'status': 'success',
    'message': 'Hello API!'
  }")
}

#[get("/ping-chain-two")]
fn pingchaintwo() -> Json<&'static str> {
  // instantiate an Api that connects to the given address
  let url = "ws://127.0.0.1:9944";
  
  
  let client = WsRpcClient::new(&url);
    let api = Api::<sr25519::Pair, _>::new(client).unwrap();

    let meta = Metadata::try_from(api.get_metadata().unwrap()).unwrap();

    meta.print_overview();
    meta.print_pallets();
    meta.print_pallets_with_calls();
    meta.print_pallets_with_events();
    meta.print_pallets_with_errors();
    meta.print_pallets_with_constants();

    // print full substrate metadata json formatted
    println!(
        "{}",
        Metadata::pretty_format(&api.get_metadata().unwrap())
            .unwrap_or_else(|| "pretty format failed".to_string())
    );
  
  Json("{
    'status': 'success',
    'message': 'Hello API!'
  }")
}

#[get("/ping-chain")]
fn pingchain() -> Json<&'static str> {
  println!("test item");
  //let item = requeststuff();

  println!("item beloow");
  // println!("ahahaha {:#?}", block_on(item));
  println!("itemabove");
  
  Json("{
    'status': 'success',
    'message': 'Hello API!'
  }")
}

/*
async fn requeststuff() -> Result<(), reqwest::Error> {
  println!("test item 2");
        let client = reqwest::Client::new();
        let resp = client.post("http://127.0.0.1:9933/")
            //.header("Content-Type", "application/json")
            .header(CONTENT_TYPE,"Content-Type: application/json")
            .body(r#"{"id":1, "jsonrpc":"2.0", "method": "state_getMetadata"}"#)
            .send()
            .await.unwrap();
    println!("ahahaha222 {:#?}", resp);

    Ok(())
}*/

