use crate::domain::services::solana_service;
use crate::domain::services::user_service;
use crate::repo;
use crate::repo::user;
use crate::server;
use solana_sdk::signature::Keypair;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub user_service: user_service::UserService,
    pub solana_service: solana_service::SolanaService,
}

#[derive(Clone)]
pub struct Config {
    pub server_config: server::Config,
    pub solana_service_config: solana_service::Config,
    pub user_service_config: user_service::Config,
}

impl Config {
    pub fn default() -> Config {
        Config {
            server_config: server::Config::default(),
            solana_service_config: solana_service::Config::default(),
            user_service_config: user_service::Config::default(),
        }
    }
}

#[derive(Clone)]
pub struct AppStateBuiler {
    user_repo: Option<repo::user::Repo>,
    solana_repo: Option<repo::solana::Repo>,
}

impl AppStateBuiler {
    pub fn new() -> AppStateBuiler {
        AppStateBuiler {
            solana_repo: None,
            user_repo: None,
        }
    }

    pub fn with_solana_repo<'a>(
        &'a mut self,
        solana_repo: repo::solana::Repo,
    ) -> &'a mut AppStateBuiler {
        self.solana_repo = Some(solana_repo);
        return self;
    }

    pub fn with_user_repo<'a>(&'a mut self, user_repo: repo::user::Repo) -> &'a mut AppStateBuiler {
        self.user_repo = Some(user_repo);
        return self;
    }

    pub fn build(
        &mut self,
        cfg: Config,
        auth_secret: Vec<u8>,
        program_keypair: Keypair,
    ) -> AppState {
        let cfg_clone = cfg.clone();

        let solana_service = solana_service::SolanaService::new(
            cfg.solana_service_config.clone(),
            program_keypair,
            self.solana_repo.take().unwrap_or(repo::solana::Repo::new()),
        );
        let user_service = user_service::UserService::new(
            cfg.user_service_config,
            self.user_repo.take().unwrap_or(repo::user::Repo::new()),
            auth_secret,
            solana_service.clone(),
        );

        AppState {
            cfg: cfg_clone,
            user_service,
            solana_service,
        }
    }
}

impl AppState {}
