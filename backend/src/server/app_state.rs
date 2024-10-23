use crate::domain::services::solana_service::{self, SolanaService};
use crate::domain::services::user_service::UserService;
use crate::{domain::model::User, repo::Repo};
use axum::extract::State;
use solana_sdk::signature::Keypair;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    // pub dbcp: Arc<DbConnPool>,
    // pub auth_mgr: AuthMgr,
    repo: Arc<Mutex<Repo>>,
    solana: solana_service::SolanaService,
}

impl AppState {
    pub fn new(
        cfg: crate::server::Config,
        program_keypair: Keypair, /*dbcp: DbConnPool*/
    ) -> Self {
        // let dbcp = Arc::new(dbcp);
        // let auth_mgr = AuthMgr::new(repo.clone());
        let repo = Arc::new(Mutex::new(Repo::new(/*dbcp.clone()*/)));
        let solana = solana_service::SolanaService::new(cfg.solana.clone(), program_keypair);

        Self { repo, solana }
    }

    pub fn get_user_service(self) -> UserService {
        UserService::new(self.repo, self.solana)
    }
}
