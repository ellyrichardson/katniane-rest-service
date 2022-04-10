extern crate rustc_serialize;

use serde::{Serialize, Deserialize};
use codec::{Decode, Encode};
use rustc_serialize::Decodable;
use rustc_serialize::Decoder;

// Dependencies for hash string converter
use sp_core::sr25519::Public;

#[derive(Encode, Decode, Clone, Default, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct AuditLog {
    // Reporter determines which system sent the log
    pub title: String,
    pub content: String,
    pub timestamp: String,
    pub reporter: Public,
}

#[derive(Encode, Decode, Clone, Default, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct AuditLogSummary {
    // Reporter determines which system sent the log
    pub filename: String,
    pub date: String,
    pub contents: Vec<AuditLog>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IncomingAuditLog {
    pub filename: String,
    pub title: String,
    pub content: String,
}

#[derive(Encode, Decode, Clone, Default, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct AuditLogToBeOpenedForClaiming {
    pub filename: String,
    pub claimer_pubkey: String,
}

#[derive(Encode, Decode, Clone, Default, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct AuditLogToBeClaimed {
    pub filename: String,
}

#[derive(Encode, Decode, Clone, Default, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct ValidatorAccount {
    pub validator_id: String,
}

