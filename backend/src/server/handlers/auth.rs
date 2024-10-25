use axum::extract::{Json, State};
use serde::{Deserialize, Serialize};

use crate::server::handlers::SignedTransaction;
use crate::server::middleware::host_extractor::ExtractHostname;
use crate::server::ErrorResp;
use crate::server::{AppState, ACCESS_TOKEN_TYPE};

use super::{parse_pubkey, TransactionResp};

#[derive(Deserialize)]
pub struct PostRegisterReq {
    pubkey: String,
    username: String,
}

#[derive(Serialize)]
pub struct PostRegisterResp {
    nonce: u64,
}

#[axum_macros::debug_handler]
pub async fn post_register(
    ExtractHostname(host): ExtractHostname,
    State(state): State<AppState>,
    Json(req): Json<PostRegisterReq>,
) -> Result<Json<TransactionResp>, ErrorResp> {
    let pubkey = parse_pubkey(&req.pubkey)?;
    let transaction_to_sign = state.user_service.register_init(&pubkey, req.username)?;

    // We want /register/complete to be called next
    let request_uri = http::uri::Builder::new()
        .authority(host)
        .path_and_query("/api/v1/auth/register/complete")
        .build()
        .expect("Host is validated by extractor, path should be always valid");
    Ok(Json(TransactionResp::from(
        &transaction_to_sign,
        request_uri.to_string(),
    )))
}

#[derive(Clone, Serialize)]
pub struct LoginCompleteResp {
    access_token: String,
    refresh_token: String,
    token_type: String, // e.g. Bearer
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PostRegisterCompleteReq {
    data: SignedTransaction,
    pubkey: String,
}

#[axum_macros::debug_handler]
pub async fn post_register_complete(
    State(state): State<AppState>,
    Json(req): Json<PostRegisterCompleteReq>,
) -> Result<Json<LoginCompleteResp>, ErrorResp> {
    let pubkey = parse_pubkey(&req.pubkey)?;
    let (transaction_id, transaction) = req.data.parse()?;

    // First, execute the transaction creating user's PDA.
    state
        .solana_service
        .execute_transaction(&pubkey, transaction_id, transaction)?;

    // Now update the user
    let (access_token, refresh_token) = state.user_service.register_complete(&pubkey)?;

    Ok(Json(LoginCompleteResp {
        access_token: access_token,
        refresh_token: refresh_token,
        token_type: ACCESS_TOKEN_TYPE.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use bincode::Options;
    use serde::{Deserialize, Serialize};
    use solana_sdk::{
        hash::Hasher,
        instruction::{AccountMeta, Instruction},
        message,
        signature::Keypair,
        signer::Signer,
        transaction,
    };

    #[test]
    fn test_serialize_transaction_bincode() {
        let from = Keypair::new();
        let to = Keypair::new();
        let data: Vec<u8> = vec![1, 2, 3, 66, 4, 44, 2, 17, 65, 6, 75];

        let instr = Instruction::new_with_bytes(
            to.pubkey(),
            data.as_slice(),
            vec![
                AccountMeta::new(from.pubkey(), true),
                AccountMeta::new(to.pubkey(), true),
            ],
        );
        let msg = message::Message::new(&[instr], Some(&to.pubkey()));
        let mut h = Hasher::default();
        h.hash(&[1, 2, 3]);
        let transaction = transaction::Transaction::new(&[&from, &to], msg, h.result());

        let serialized = bincode::options()
            .with_little_endian()
            .serialize(&transaction)
            .unwrap();
        println!("{:?}", String::from_utf8_lossy(&serialized));

        let deserialized: transaction::Transaction = bincode::options()
            .with_little_endian()
            .deserialize(&serialized)
            .unwrap();
        assert_eq!(deserialized.message().hash(), transaction.message().hash());
        assert_eq!(deserialized.signatures.len(), transaction.signatures.len());
        assert_eq!(deserialized.signatures[0], transaction.signatures[0]);
    }

    #[test]
    fn test_serialize_transaction_json() {
        let from = Keypair::new();
        let to = Keypair::new();
        let data: Vec<u8> = vec![1, 2, 3, 66, 4, 44, 2, 17, 65, 6, 75];

        let instr = Instruction::new_with_bytes(
            to.pubkey(),
            data.as_slice(),
            vec![
                AccountMeta::new(from.pubkey(), true),
                AccountMeta::new(to.pubkey(), true),
            ],
        );
        let msg = message::Message::new(&[instr], Some(&to.pubkey()));
        let mut h = Hasher::default();
        h.hash(&[1, 2, 3]);
        let transaction = transaction::Transaction::new(&[&from, &to], msg, h.result());

        let buf = Vec::<u8>::new();
        let mut serializer = serde_json::ser::Serializer::new(buf);
        transaction.serialize(&mut serializer).unwrap();
        let serialized = serializer.into_inner();
        println!("{:?}", String::from_utf8_lossy(&serialized));

        let deserialized = transaction::Transaction::deserialize(
            &mut serde_json::Deserializer::from_slice(&serialized),
        )
        .unwrap();

        assert_eq!(deserialized.message().hash(), transaction.message().hash());
        assert_eq!(deserialized.signatures.len(), transaction.signatures.len());
        assert_eq!(deserialized.signatures[0], transaction.signatures[0]);
    }
}
