use std::{cell::Ref, time::SystemTime};

use solana_sdk::{message::Message, pubkey::Pubkey, signature::Signature};
use uuid::Uuid;

use super::error::Error;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct User {
    pub pubkey: Pubkey,
    pub username: String,

    pub pda_pubkey: Option<Pubkey>, // completed the registration process
    pub refresh_token: Option<RefreshToken>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RefreshToken {
    pub token: String,
    pub valid_until: SystemTime,
}

impl RefreshToken {
    pub fn verify(&self, input_token: &String) -> Result<(), Error> {
        let _ = Uuid::parse_str(&input_token);
        if self.token.ne(input_token) {
            println!("Refresh token mismatch");
            return Err(super::error::Error::InvalidAuthToken);
        }

        // Check if the refresh_token is still valid,
        // new login is necessary.
        if self.valid_until.le(&SystemTime::now()) {
            println!("Refresh token expired");
            return Err(super::error::Error::AuthTokenExpired);
        }

        Ok(())
    }
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
