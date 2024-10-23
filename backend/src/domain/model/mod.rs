use std::time::SystemTime;

use solana_sdk::message::Message;
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct User {
    pub pubkey: solana_sdk::pubkey::Pubkey,
    pub username: String,
    // pda_pubkey solana PDA of this user holding the user's basic data
    pub pda_pubkey: Option<solana_sdk::pubkey::Pubkey>,
}

#[derive(Clone, PartialEq)]
pub struct AccessToken {
    // pubkey of this token's owner
    pub pubkey: solana_sdk::pubkey::Pubkey,
    // access_token used to access protected API endpoints
    pub access_token: String,
    // token_type e.g. Bearer
    pub token_type: String,
    // expires_in time in seconds
    pub expires_in: u32,
    // nonce is used to refresh the access token once it expires
    pub nonce: u64,
}

// a transaction to be signed by a user on frontend
#[derive(Clone, PartialEq)]
pub struct TransactionRecord {
    pub id: uuid::Uuid,
    pub pubkey: solana_sdk::pubkey::Pubkey,
    pub message_hash: solana_sdk::hash::Hash,
    pub valid_until: std::time::SystemTime,
}

pub struct TransactionToSign {
    pub message: Message,
    pub transaction_id: Uuid,
    pub valid_until: SystemTime,
    pub callback: String,
}
