use std::time::SystemTime;

use solana_sdk::{message::Message, pubkey::Pubkey, signature::Signature};
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct User {
    pub pubkey: Pubkey,
    pub username: String,

    pub pda_pubkey: Option<Pubkey>, // completed the registration process
    pub refresh_token: Option<String>,
}

// a transaction to be signed by a user on frontend
#[derive(Clone, PartialEq)]
pub struct TransactionRecord {
    pub id: uuid::Uuid,
    pub pubkey: Pubkey,
    pub message_hash: solana_sdk::hash::Hash,
    pub valid_until: std::time::SystemTime,
    pub client_signature: Option<Signature>,
}

pub struct TransactionToSign {
    pub message: Message,
    pub transaction_id: Uuid,
    pub valid_until: SystemTime,
}
