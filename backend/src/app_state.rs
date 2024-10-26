use crate::domain::services::solana_service;
use crate::domain::services::user_service;
use crate::repo;
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

impl AppState {
    pub fn new(cfg: Config, auth_secret: Vec<u8>, program_keypair: Keypair) -> Self {
        let cfg_clone = cfg.clone();
        let user_repo = repo::user::Repo::new();
        let solana_repo = repo::solana::Repo::new();

        let solana_service = solana_service::SolanaService::new(
            cfg.solana_service_config.clone(),
            program_keypair,
            solana_repo,
        );
        let user_service = user_service::UserService::new(
            cfg.user_service_config,
            user_repo,
            auth_secret,
            solana_service.clone(),
        );

        Self {
            cfg: cfg_clone,
            user_service,
            solana_service,
        }
    }
}
