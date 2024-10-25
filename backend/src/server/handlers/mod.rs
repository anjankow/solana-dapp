pub mod auth;
pub mod users;
use crate::{
    domain::{error::Error, model},
    server::ErrorResp,
};
use bincode::Options;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};
use std::str::FromStr;
use std::time::SystemTime;
use uuid::Uuid;

pub async fn handler() -> axum::response::Html<&'static str> {
    axum::response::Html("<h1>Hello, World!</h1>")
}

pub fn parse_pubkey(pubkey: &String) -> Result<Pubkey, Error> {
    pubkey
        .parse::<solana_sdk::pubkey::Pubkey>()
        .map_err(|e| Error::from(e))
        .inspect_err(|e| println!("Failed to parse pubkey: {}", e))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionResp {
    pub message: Vec<u8>,
    pub transaction_id: String,
    pub valid_until: SystemTime,
    pub request_uri: String,
}

impl TransactionResp {
    fn from(model: &model::TransactionToSign, request_uri: String) -> Self {
        TransactionResp {
            message: model.message.serialize(),
            transaction_id: model.transaction_id.to_string(),
            valid_until: model.valid_until,
            request_uri: request_uri,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SignedTransaction {
    pub transaction: Vec<u8>, // serialized with borsh
    pub transaction_id: String,
}

impl SignedTransaction {
    fn parse(&self) -> Result<(Uuid, Transaction), ErrorResp> {
        let transaction: Transaction = bincode::options()
            .with_little_endian()
            .deserialize(&self.transaction)
            .map_err(|err| {
                ErrorResp::new(
                    StatusCode::BAD_REQUEST,
                    &format!("Failed to deserialized transaction: {}", err.to_string()),
                )
            })?;
        let transaction_id = Uuid::from_str(&self.transaction_id).map_err(|_| {
            ErrorResp::new(StatusCode::BAD_REQUEST, "Invalid transaction id format")
        })?;

        Ok((transaction_id, transaction))
    }
}

// #[derive(Serialize, Deserialize, Clone)]
// pub enum PostTransactionRespDataType {
//     NoData,
// }

// #[derive(Serialize, Deserialize, Clone)]
// pub struct PostTransactionResp {
//     pub data_type: PostTransactionRespDataType,
//     pub data: Vec<u8>,
// }

// impl PostTransactionResp {
//     fn default() -> PostTransactionResp {
//         PostTransactionResp {
//             data_type: PostTransactionRespDataType::NoData,
//             data: vec![],
//         }
//     }
// }

// #[axum_macros::debug_handler]
// pub async fn post_transaction(

//     State(state): State<AppState>,
//     Json(req): Json<PostTransactionReq>,
// ) -> Result<Json<PostTransactionResp>, ErrorResp> {
//     state.solana_service.execute_transaction(wallet_pubkey, transaction_id, signed_transaction)
//     Ok(Json(PostTransactionResp::default()))
// }
