pub mod handlers;
mod middleware;

use crate::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub const ACCESS_TOKEN_TYPE_BEARER: &str = "Bearer";

#[derive(Clone)]
pub struct Config {
    pub bind_address: String,
    pub scheme: http::uri::Scheme,
    pub access_token_type: String,
}

impl Config {
    pub fn default() -> Config {
        Config {
            bind_address: "127.0.0.1:3000".to_string(),
            scheme: http::uri::Scheme::HTTP,
            access_token_type: ACCESS_TOKEN_TYPE_BEARER.to_string(),
        }
    }
}

pub struct Server {
    cfg: Config,
    router: Router,
}

impl Server {
    pub fn new(cfg: Config, app_state: AppState) -> Server {
        Server {
            cfg,
            router: Server::new_stateless_router().with_state(app_state),
        }
    }

    pub fn new_stateless_router() -> Router<AppState> {
        let router = Router::new()
            .route("/", get(handlers::handler))
            .route("/api/v1/users/:pubkey", get(handlers::users::get_user))
            .route("/api/v1/auth/register", post(handlers::auth::post_register))
            .route(
                "/api/v1/auth/register/complete",
                post(handlers::auth::post_register_complete),
            );
        router
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        let listener = tokio::net::TcpListener::bind(&self.cfg.bind_address).await?;

        println!(
            "listening on {}",
            listener
                .local_addr()
                .map(|a| a.to_string())
                .unwrap_or("<NO LOCAL ADDRESS>".to_string()),
        );
        axum::serve(listener, self.router.clone()).await?;
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

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct ErrorResp {
    #[serde(skip_serializing, skip_deserializing)]
    status_code: StatusCode,
    pub error: String,
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
            crate::domain::error::Error::WalletNotFound => StatusCode::NOT_FOUND,
            crate::domain::error::Error::WalletInsufficientFounds => StatusCode::CONFLICT,
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
