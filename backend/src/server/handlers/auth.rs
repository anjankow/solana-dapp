use axum::extract::{Json, State};
use bincode::Options;
use serde::{Deserialize, Serialize};

use super::{parse_pubkey, TransactionResp};
use crate::app_state::AppState;
use crate::domain::services::user_service::AuthTokens;
use crate::server::handlers::SignedTransaction;
use crate::server::ErrorResp;
use crate::utils;

#[derive(Deserialize)]
pub struct PostRegisterReq {
    pubkey: String,
    username: String,
}

#[axum_macros::debug_handler]
pub async fn post_register(
    State(state): State<AppState>,
    Json(req): Json<PostRegisterReq>,
) -> Result<Json<TransactionResp>, ErrorResp> {
    let pubkey = parse_pubkey(&req.pubkey)?;
    let transaction_to_sign = state
        .user_service
        .register_init(&pubkey, req.username)
        .inspect_err(|err| {
            println!("Failed to init registration: {}", err);
        })?;

    // We want /register/complete to be called next
    let request_uri = http::uri::Builder::new()
        .authority(state.cfg.server_config.bind_address)
        .scheme(state.cfg.server_config.scheme)
        .path_and_query("/api/v1/auth/register/complete")
        .build()
        .expect("Host is validated by extractor, path should be always valid");
    Ok(Json(
        TransactionResp::new(&transaction_to_sign, request_uri.to_string())
            .inspect_err(|e| println!("{}", e.error))?,
    ))
}

#[derive(Deserialize, Clone, Serialize)]
pub struct LoginCompleteReq {
    pubkey: String,
    refresh_token: String,
    signature: String,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct LoginCompleteResp {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String, // e.g. Bearer
}

pub async fn login_complete(
    State(state): State<AppState>,
    Json(req): Json<LoginCompleteReq>,
) -> Result<Json<LoginCompleteResp>, ErrorResp> {
    let pubkey = parse_pubkey(&req.pubkey)?;
    let tokens = state
        .user_service
        .login_complete(&pubkey, req.refresh_token, req.signature)?;

    Ok(Json(LoginCompleteResp {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: state.cfg.server_config.access_token_type,
    }))
}

#[derive(Deserialize, Clone, Serialize)]
pub struct RefreshReq {
    pubkey: String,
    refresh_token: String,
    signature: Vec<u8>,
}

pub async fn post_refresh(
    state: State<AppState>,
    req: Json<LoginCompleteReq>,
) -> Result<Json<LoginCompleteResp>, ErrorResp> {
    login_complete(state, req).await
}

#[derive(Deserialize, Clone, Serialize)]
pub struct LoginInitReq {
    pubkey: String,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct LoginInitResp {
    pub refresh_token: String,
}

pub async fn post_login_init(
    State(state): State<AppState>,
    Json(req): Json<LoginInitReq>,
) -> Result<Json<LoginInitResp>, ErrorResp> {
    let pubkey = parse_pubkey(&req.pubkey)?;
    let refresh_token = state.user_service.login_init(&pubkey).inspect_err(|err| {
        println!("Failed to login user: {}", err);
    })?;

    Ok(Json(LoginInitResp { refresh_token }))
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
    let (transaction_id, transaction) = req.data.parse().inspect_err(|err| {
        println!("Failed to parse transaction: {}", err.error);
    })?;

    // First, execute the transaction creating user's PDA.
    state
        .solana_service
        .execute_transaction(&pubkey, transaction_id, transaction)
        .inspect_err(|err| {
            println!("Failed to execute registration transaction: {}", err);
        })?;

    // Now update the user
    let tokens = state
        .user_service
        .register_complete(&pubkey)
        .inspect_err(|err| {
            println!("Failed to complete registration: {}", err);
        })?;

    Ok(Json(LoginCompleteResp {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: state.cfg.server_config.access_token_type,
    }))
}

#[cfg(test)]
mod tests {
    use bincode::Options;
    use http::StatusCode;
    use serde::{Deserialize, Serialize};
    use solana_sdk::{
        hash::Hasher,
        instruction::{AccountMeta, Instruction},
        message,
        signature::Keypair,
        signer::Signer,
        transaction,
    };

    use crate::{server::ErrorResp, utils};

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

        let serialized = utils::bincode::serialize(&transaction).unwrap();
        println!("{:?}", String::from_utf8_lossy(&serialized));

        let deserialized: transaction::Transaction =
            utils::bincode::deserialize(serialized).unwrap();
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
