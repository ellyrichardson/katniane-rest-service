use katniane_rest::models::{AuditLogToBeOpenedForClaiming, IncomingAuditLog, AuditLogToBeClaimed};
use chrono::{Local, DateTime};
use substrate_api_client::{Api, compose_extrinsic, UncheckedExtrinsicV4, XtStatus};
use substrate_api_client::rpc::WsRpcClient;
use sp_core::ed25519::{self};
use sp_core::crypto::{Pair, Ss58Codec};

pub fn submit_to_save_log(api: Api<sp_core::sr25519::Pair, WsRpcClient>, incoming_audit_log: IncomingAuditLog, now: DateTime<Local>, datetime: std::string::String) {
    // Compose the extrinsic
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
}

pub fn submit_to_open_log(api: Api<sp_core::sr25519::Pair, WsRpcClient>, log_to_be_opened: AuditLogToBeOpenedForClaiming) {
    match ed25519::Public::from_ss58check(&log_to_be_opened.claimer_pubkey) {
      Ok(olc_res) => {
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
}

pub fn submit_to_claim_log(api: Api<sp_core::sr25519::Pair, WsRpcClient>, log_to_be_claimed: AuditLogToBeClaimed) {
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
}