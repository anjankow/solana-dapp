use anti_loneliness_solana_dapp::app_state;
use anti_loneliness_solana_dapp::domain;
use anti_loneliness_solana_dapp::server;
use anti_loneliness_solana_dapp::server::ErrorResp;
use anti_loneliness_solana_dapp::utils;
use anti_loneliness_solana_dapp::utils::jwt;
use bincode::Options;
use jwt_simple::claims::NoCustomClaims;
use jwt_simple::prelude::HS256Key;
use jwt_simple::prelude::MACLike;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use solana_sdk::instruction::CompiledInstruction;
use solana_sdk::message::Message;
use solana_sdk::transaction::Transaction;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::process::Command;

mod common;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_register_new_pubkey() {
    let auth_secret = HS256Key::generate();
    let test_server = common::new_test_server_with_auth_secret(&auth_secret);

    let wallet = Keypair::new();
    println!("WALLET: {}", wallet.to_base58_string());
    let wallet_pubkey = wallet.pubkey();
    println!("WALLET PUBKEY: {}", wallet_pubkey.to_string());

    // INIT THIS WALLET ON THE BLOCKCHAIN
    Command::new("solana")
        .arg("airdrop")
        .arg("--commitment")
        .arg("finalized")
        .arg("1")
        .arg(format!("{}", wallet_pubkey.to_string()))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    // REGISTER INIT
    let response = test_server
        .post("/api/v1/auth/register")
        .json(&json!({
            "username":"paulinka",
            "pubkey": wallet_pubkey.to_string(),
        }))
        .await;

    // Check the response
    response.assert_status_ok();
    let register_resp: server::handlers::TransactionResp = response.json();
    assert_eq!(
        register_resp.request_uri,
        "http://127.0.0.1:3000/api/v1/auth/register/complete"
    );
    assert!(register_resp.transaction_id.len() > 0);

    // Create the transaction
    let message = register_resp.deserialize_message().unwrap();

    // Sign it
    let signed_transaction = Transaction::new(&[wallet], message.clone(), message.recent_blockhash);
    let signed_hash = signed_transaction.verify_and_hash_message().unwrap();
    assert_eq!(signed_hash, message.hash());

    // Pass it back to the server
    // REGISTER COMPLETE
    let serialized_transaction = common::serialize_transaction(signed_transaction);
    let response = test_server
        .post(&register_resp.request_uri)
        .json(&json!({
            "data": {
                "transaction":serialized_transaction,
                "transaction_id": register_resp.transaction_id,
            },
            "pubkey": wallet_pubkey.to_string(),
        }))
        .await;

    // Check the response
    response.assert_status_ok();
    let register_compl_resp: server::handlers::auth::LoginCompleteResp = response.json();
    assert!(register_compl_resp.access_token.len() > 0);
    assert!(
        jwt_simple::algorithms::HS256Key::verify_token::<NoCustomClaims>(
            &auth_secret,
            &register_compl_resp.access_token,
            None
        )
        .is_ok()
    );

    println!("{}", response.text());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_register_new_pubkey_manipulated_message() {
    let auth_secret = HS256Key::generate();
    let test_server = common::new_test_server_with_auth_secret(&auth_secret);

    let wallet = Keypair::new();
    println!("WALLET: {}", wallet.to_base58_string());
    let wallet_pubkey = wallet.pubkey();
    println!("WALLET PUBKEY: {}", wallet_pubkey.to_string());

    // REGISTER INIT
    let response = test_server
        .post("/api/v1/auth/register")
        .json(&json!({
            "username":"paulinka",
            "pubkey": wallet_pubkey.to_string(),
        }))
        .await;

    // Check the response
    response.assert_status_ok();
    let register_resp: server::handlers::TransactionResp = response.json();
    assert_eq!(
        register_resp.request_uri,
        "http://127.0.0.1:3000/api/v1/auth/register/complete"
    );
    assert!(register_resp.transaction_id.len() > 0);

    // MANIPULATE THE MESSAGE
    let mut message = register_resp.deserialize_message().unwrap();
    message.instructions.push(message.instructions[0].clone());

    // Sign it
    let signed_transaction = Transaction::new(&[wallet], message.clone(), message.recent_blockhash);

    // Pass it back to the server
    // REGISTER COMPLETE
    let serialized_transaction = common::serialize_transaction(signed_transaction);
    let response = test_server
        .post(&register_resp.request_uri)
        .json(&json!({
            "data": {
                "transaction":serialized_transaction,
                "transaction_id": register_resp.transaction_id,
            },
            "pubkey": wallet_pubkey.to_string(),
        }))
        .await;

    // Check the response
    response.assert_status_bad_request();
    let err: ErrorResp = response.json();
    assert!(err
        .error
        .contains(&domain::error::Error::InvalidTransaction("".to_string()).to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_register_new_pubkey_account_not_found() {
    let test_server = common::new_default_test_server();

    // this wallet does't exist on solana
    let wallet = Keypair::new();
    println!("WALLET: {}", wallet.to_base58_string());
    let wallet_pubkey = wallet.pubkey();
    println!("WALLET PUBKEY: {}", wallet_pubkey.to_string());

    // REGISTER INIT
    let response = test_server
        .post("/api/v1/auth/register")
        .json(&json!({
            "username":"paulinka",
            "pubkey": wallet_pubkey.to_string(),
        }))
        .await;

    // Check the response
    response.assert_status_ok();
    let register_resp: server::handlers::TransactionResp = response.json();

    // Create the transaction
    let message = register_resp.deserialize_message().unwrap();

    // Sign it
    let signed_transaction = Transaction::new(&[wallet], message.clone(), message.recent_blockhash);

    // Pass it back to the server
    // REGISTER COMPLETE
    let serialized_transaction = common::serialize_transaction(signed_transaction);
    let response = test_server
        .post(&register_resp.request_uri)
        .json(&json!({
            "data": {
                "transaction":serialized_transaction,
                "transaction_id": register_resp.transaction_id,
            },
            "pubkey": wallet_pubkey.to_string(),
        }))
        .await;

    // Check the response
    response.assert_status_not_found();
    let err: ErrorResp = response.json();
    assert!(err
        .error
        .contains(&domain::error::Error::WalletNotFound.to_string()));
}
