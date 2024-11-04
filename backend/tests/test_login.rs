use anti_loneliness_solana_dapp::domain;
use anti_loneliness_solana_dapp::domain::model::RefreshToken;
use anti_loneliness_solana_dapp::repo;
use anti_loneliness_solana_dapp::server;
use anti_loneliness_solana_dapp::server::ErrorResp;
use anti_loneliness_solana_dapp::utils;
use bincode::Options;
use ed25519_dalek::ed25519::signature::SignerMut;
use http::StatusCode;
use jwt_simple::claims::NoCustomClaims;
use jwt_simple::prelude::HS256Key;
use jwt_simple::prelude::MACLike;
use serde_json::json;
use solana_sdk::pubkey;
use solana_sdk::transaction::Transaction;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::process::Command;
use std::time::SystemTime;
use uuid::Uuid;

mod common;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_login_success() {
    // User exists and is confirmed
    let wallet = Keypair::new();
    let user = domain::model::User {
        pubkey: wallet.pubkey(),
        username: "User1".to_string(),
        pda_pubkey: Some(wallet.pubkey()),
        refresh_token: Some(RefreshToken {
            token: uuid::Uuid::new_v4().to_string(),
            valid_until: SystemTime::now()
                .checked_add(std::time::Duration::from_secs(3600))
                .unwrap(),
        }),
    };

    let user_repo = repo::user::Repo::new();
    user_repo.add_user(user.clone()).unwrap();

    let auth_secret = HS256Key::generate();

    let test_server = common::TestServerBuilder::new()
        .with_user_repo(user_repo)
        .with_auth_secret(&auth_secret)
        .build();

    // REGISTER INIT
    let response = test_server
        .post("/api/v1/auth/login")
        .json(&json!({
            "pubkey": wallet.pubkey().to_string(),
        }))
        .await;

    // Check the response
    response.assert_status_ok();
    let login_resp: server::handlers::auth::LoginInitResp = response.json();
    let refresh_token = login_resp.refresh_token;

    // Sign the token and complete logging in
    let mut signing_key =
        ed25519_dalek::SigningKey::from_keypair_bytes(&wallet.to_bytes()).unwrap();
    let signature = signing_key.sign(refresh_token.as_bytes());

    // Pass it back to the server
    // LOGIN COMPLETE
    let response = test_server
        .post("/api/v1/auth/login/complete")
        .json(&json!({
            "refresh_token": refresh_token,
            "signature": signature.to_string(),
            "pubkey": wallet.pubkey().to_string(),
        }))
        .await;

    // Check the response
    response.assert_status_ok();
    let login_compl_resp: server::handlers::auth::LoginCompleteResp = response.json();
    assert!(login_compl_resp.access_token.len() > 0);
    assert!(
        jwt_simple::algorithms::HS256Key::verify_token::<NoCustomClaims>(
            &auth_secret,
            &login_compl_resp.access_token,
            None
        )
        .is_ok()
    );

    println!("{}", response.text());

    // check if this token works
    // GET USER
    let response = test_server
        .get(format!("/api/v1/users/{}", wallet.pubkey().to_string()).as_str())
        .add_header(
            "Authorization",
            format!("Bearer {}", login_compl_resp.access_token),
        )
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let get_user_resp: server::handlers::users::GetUserResp = response.json();
    println!("{:?}", get_user_resp);
    assert!(get_user_resp.owned);
    assert_eq!(get_user_resp.pubkey, wallet.pubkey().to_string());
}
