use crate::domain::model::User;
use crate::domain::services::solana_service::{self, SolanaService};
use crate::domain::services::user_service::{self, UserService};
use crate::repo;
use axum::extract::State;
use solana_sdk::signature::Keypair;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    // pub dbcp: Arc<DbConnPool>,
    // pub auth_mgr: AuthMgr,
    pub user_service: user_service::UserService,
    pub solana_service: solana_service::SolanaService,
}

impl AppState {
    pub fn new(
        cfg: crate::server::Config,
        program_keypair: Keypair, /*dbcp: DbConnPool*/
    ) -> Self {
        // let dbcp = Arc::new(dbcp);
        // let auth_mgr = AuthMgr::new(repo.clone());
        let user_repo = repo::user::Repo::new();
        let solana_repo = repo::solana::Repo::new();

        let solana_service =
            solana_service::SolanaService::new(cfg.solana.clone(), program_keypair, solana_repo);
        let user_service = UserService::new(user_repo, solana_service.clone());

        Self {
            user_service: user_service,
            solana_service: solana_service,
        }
    }
}
