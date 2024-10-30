use anti_loneliness_solana_dapp::app_state;
use anti_loneliness_solana_dapp::domain::services;
use anti_loneliness_solana_dapp::repo;
use anti_loneliness_solana_dapp::repo::solana;
use anti_loneliness_solana_dapp::repo::user;
use anti_loneliness_solana_dapp::server;
use anti_loneliness_solana_dapp::utils;
use anti_loneliness_solana_dapp::utils::jwt;
use axum_test::TestServer;
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
use std::borrow::BorrowMut;
use std::process::Command;

pub struct TestServerBuilder {
    cfg: Option<app_state::Config>,
    auth_secret: Option<HS256Key>,
    program_keypair: Option<solana_sdk::signer::keypair::Keypair>,
    user_repo: Option<repo::user::Repo>,
    solana_repo: Option<repo::solana::Repo>,
}

impl TestServerBuilder {
    pub fn new() -> TestServerBuilder {
        TestServerBuilder {
            auth_secret: None,
            cfg: None,
            program_keypair: None,
            solana_repo: None,
            user_repo: None,
        }
    }

    pub fn with_auth_secret<'a>(&'a mut self, auth_secret: &HS256Key) -> &'a mut TestServerBuilder {
        self.auth_secret = Some(auth_secret.clone());
        self
    }

    pub fn with_program_keypair<'a>(
        &'a mut self,
        program_keypair: solana_sdk::signer::keypair::Keypair,
    ) -> &'a mut TestServerBuilder {
        self.program_keypair = Some(program_keypair);
        self
    }

    pub fn with_config<'a>(&'a mut self, cfg: app_state::Config) -> &'a mut TestServerBuilder {
        self.cfg = Some(cfg);
        self
    }

    pub fn with_user_repo<'a>(&'a mut self, repo: repo::user::Repo) -> &'a mut TestServerBuilder {
        self.user_repo = Some(repo);
        self
    }

    pub fn with_solana_repo<'a>(
        &'a mut self,
        repo: repo::solana::Repo,
    ) -> &'a mut TestServerBuilder {
        self.solana_repo = Some(repo);
        self
    }

    pub fn build(&mut self) -> TestServer {
        let auth_secret = self
            .auth_secret
            .take()
            .unwrap_or(HS256Key::generate())
            .to_bytes();
        let program_keypair = self.program_keypair.take().unwrap_or_else(|| {
            // relative to Cargo
            let keypair_path = "solana_program/target/deploy/solana_program-keypair.json";
            solana_sdk::signer::keypair::read_keypair_file(keypair_path).unwrap()
        });
        let cfg = self.cfg.take().unwrap_or(app_state::Config::default());
        let solana_repo = self.solana_repo.take().unwrap_or(repo::solana::Repo::new());
        let user_repo = self.user_repo.take().unwrap_or(repo::user::Repo::new());
        let app = app_state::AppStateBuiler::new()
            .with_solana_repo(solana_repo)
            .with_user_repo(user_repo)
            .build(cfg, auth_secret, program_keypair);

        let router = server::Server::new_stateless_router().with_state(app);

        let test_server = axum_test::TestServer::new(router).unwrap();
        test_server
    }
}
