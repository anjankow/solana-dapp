use anti_loneliness_solana_dapp::app_state;
use anti_loneliness_solana_dapp::server;
use anti_loneliness_solana_dapp::utils;
use anti_loneliness_solana_dapp::utils::jwt;
use bincode::Options;
use jwt_simple::claims::NoCustomClaims;
use jwt_simple::prelude::HS256Key;
use jwt_simple::prelude::MACLike;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use solana_sdk::message::Message;
use solana_sdk::transaction::Transaction;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::process::Command;

pub fn new_default_test_server() -> axum_test::TestServer {
    // relative to Cargo
    let auth_secret = HS256Key::generate();
    new_test_server_with_auth_secret(&auth_secret)
}

pub fn new_test_server_with_auth_secret(auth_secret: &HS256Key) -> axum_test::TestServer {
    // relative to Cargo
    let keypair_path = "solana_program/target/deploy/solana_program-keypair.json";
    let program_keypair = solana_sdk::signer::keypair::read_keypair_file(keypair_path).unwrap();

    let cfg = app_state::Config::default();
    let app = app_state::AppState::new(cfg.clone(), auth_secret.to_bytes(), program_keypair);
    let router = server::Server::new_stateless_router().with_state(app);

    let test_server = axum_test::TestServer::new(router).unwrap();
    test_server
}

pub fn serialize_transaction(transaction: Transaction) -> Vec<u8> {
    let serialized_transaction = bincode::options()
        .with_little_endian()
        .serialize(&transaction)
        .unwrap();
    serialized_transaction
}
