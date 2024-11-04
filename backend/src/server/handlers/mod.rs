pub mod auth;
pub mod users;
use crate::domain::error::Error;
use crate::{domain::model, server::ErrorResp, utils};
use bincode::Options;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use solana_sdk::{message::Message, pubkey::Pubkey, transaction::Transaction};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionResp {
    pub message: Vec<u8>,
    pub transaction_id: String,
    pub valid_until: SystemTime,
    pub request_uri: String,
}

impl TransactionResp {
    pub fn new(model: &model::TransactionToSign, request_uri: String) -> Result<Self, ErrorResp> {
        let serialized_message = utils::bincode::serialize(&model.message).map_err(|_| {
            Error::GeneralError("Failed to serialized message to TransactionToSign".to_string())
        })?;

        Ok(TransactionResp {
            message: serialized_message,
            transaction_id: model.transaction_id.to_string(),
            valid_until: model.valid_until,
            request_uri: request_uri,
        })
    }

    pub fn deserialize_message(&self) -> Result<Message, Error> {
        let message: Message = bincode::deserialize(&self.message).map_err(|e| {
            Error::InvalidTransaction(format!("Failed to deserialize message: {}", e.to_string()))
        })?;
        Ok(message)
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
                    &format!("Failed to deserialize transaction, should be serialized to bincode with little endian: {}", err.to_string()),
                )
            })?;
        let transaction_id = Uuid::from_str(&self.transaction_id).map_err(|_| {
            ErrorResp::new(StatusCode::BAD_REQUEST, "Invalid transaction id format")
        })?;

        Ok((transaction_id, transaction))
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use solana_sdk::{instruction::AccountMeta, message::Message};
    use uuid::Uuid;

    use crate::domain::model::TransactionToSign;

    use super::TransactionResp;

    #[test]
    fn serialize_transaction_message() {
        let message = get_example_message();

        let model_transaction = TransactionToSign {
            message: message.clone(),
            transaction_id: Uuid::new_v4(),
            valid_until: SystemTime::now(),
        };
        let transaction =
            TransactionResp::new(&model_transaction, "localhost:9999".to_string()).unwrap();
        assert!(transaction.message.len() > 0);
        assert_eq!(&transaction.request_uri, "localhost:9999");

        let deserialized_message = transaction.deserialize_message().unwrap();
        assert_eq!(message.hash(), deserialized_message.hash());
    }

    #[test]
    fn serialize_transaction() {}

    fn get_example_message() -> Message {
        let accounts = vec![AccountMeta::new_readonly(
            solana_sdk::system_program::ID,
            true,
        )];
        let instruction = solana_sdk::instruction::Instruction::new_with_bytes(
            solana_sdk::system_program::ID,
            &vec![1, 2, 3, 4, 5, 6, 7, 8],
            accounts,
        );
        let message = Message::new(&[instruction], Some(&solana_sdk::system_program::ID));
        message
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
