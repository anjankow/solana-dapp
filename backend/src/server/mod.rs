mod app_state;
mod handlers;
mod middleware;

use crate::{
    domain::services::{solana_service, user_service},
    server::app_state::AppState,
};
use axum::{
    routing::{get, post},
    Router,
};

const ACCESS_TOKEN_TYPE: &str = "Bearer";

#[derive(Clone)]
pub struct Config {
    pub bind_address: String,
    pub solana_service_config: solana_service::Config,
    pub user_service_config: user_service::Config,
}

impl Config {
    pub fn default() -> Config {
        Config {
            bind_address: "127.0.0.1:3000".to_string(),
            solana_service_config: solana_service::Config::default(),
            user_service_config: user_service::Config::default(),
        }
    }
}

pub struct Server {
    cfg: Config,
}

impl Server {
    pub fn new(cfg: Config) -> Server {
        Server { cfg }
    }

    pub async fn run(
        &self,
        auth_secret: Vec<u8>,
        program_keypair: Keypair,
    ) -> Result<(), std::io::Error> {
        let state = AppState::new(self.cfg.clone(), auth_secret, program_keypair);

        let router = Router::new()
            .route("/", get(handlers::handler))
            .route("/api/v1/users/:pubkey", get(handlers::users::get_user))
            .route("/api/v1/auth/register", post(handlers::auth::post_register))
            .route(
                "/api/v1/auth/register/complete",
                post(handlers::auth::post_register_complete),
            )
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(&self.cfg.bind_address).await?;

        println!(
            "listening on {}",
            listener
                .local_addr()
                .map(|a| a.to_string())
                .unwrap_or("<NO LOCAL ADDRESS>".to_string()),
        );
        axum::serve(listener, router).await?;
        Ok(())
    }
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

impl IntoResponse for ErrorResp {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}

use serde::Serialize;
use solana_sdk::signature::Keypair;
#[derive(Serialize)]
pub struct ErrorResp {
    #[serde(skip_serializing)]
    status_code: StatusCode,
    error: String,
}

impl ErrorResp {
    pub fn new(status_code: StatusCode, error: &str) -> ErrorResp {
        ErrorResp {
            status_code: status_code,
            error: error.to_string(),
        }
    }
}

impl From<crate::domain::error::Error> for ErrorResp {
    fn from(value: crate::domain::error::Error) -> Self {
        let status = match &value {
            crate::domain::error::Error::GeneralError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            crate::domain::error::Error::InvalidPubKey(_) => StatusCode::BAD_REQUEST,
            crate::domain::error::Error::UserNotFound => StatusCode::NOT_FOUND,
            crate::domain::error::Error::UserAlreadyInitialized => StatusCode::BAD_REQUEST,
            crate::domain::error::Error::TransactionNotFound => StatusCode::NOT_FOUND,
            crate::domain::error::Error::InvalidTransaction(_) => StatusCode::BAD_REQUEST,
            crate::domain::error::Error::TransactionExpired => StatusCode::FORBIDDEN,
        };

        let mut error_resp = value.to_string();
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            // don't expose the error to the client
            error_resp = "Something went wrong :(".to_string();
        }

        ErrorResp {
            status_code: status,
            error: error_resp,
        }
    }
}

impl From<&str> for ErrorResp {
    fn from(value: &str) -> Self {
        ErrorResp {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            error: value.to_string(),
        }
    }
}
