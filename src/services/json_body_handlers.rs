use katniane_rest::models::{AuditLogToBeOpenedForClaiming, IncomingAuditLog, AuditLogToBeClaimed};
use warp::Filter;

pub fn incoming_audit_log_body() -> impl Filter<Extract = (IncomingAuditLog,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    println!("json log_body called");
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub fn open_log_for_claim_body() -> impl Filter<Extract = (AuditLogToBeOpenedForClaiming,), Error = warp::Rejection> + Clone {
  // When accepting a body, we want a JSON body
  // (and to reject huge payloads)...
  println!("json validator_body called");
  warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub fn claim_log_body() -> impl Filter<Extract = (AuditLogToBeClaimed,), Error = warp::Rejection> + Clone {
  // When accepting a body, we want a JSON body
  // (and to reject huge payloads)...
  println!("json validator_body called");
  warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}