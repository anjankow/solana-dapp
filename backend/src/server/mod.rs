pub mod handlers;
mod middleware;

use std::sync::Arc;

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
    pub auth_config: middleware::auth::AuthMiddlewareConfig,
}

impl Config {
    pub fn default() -> Config {
        Config {
            bind_address: "127.0.0.1:3000".to_string(),
            scheme: http::uri::Scheme::HTTP,
            access_token_type: ACCESS_TOKEN_TYPE_BEARER.to_string(),
            auth_config: AuthMiddlewareConfig::new_with_default_values(
                Arc::new(HS256Key::generate()),
                AuthMiddlewareConfig::map_allowed_issuers(vec!["anti-loneliness".to_string()]),
            ),
        }
    }

    pub fn default_with_auth_key(auth_secret: Arc<HS256Key>) -> Self {
        let mut ret = Self::default();
        let _ = ret.auth_config.with_auth_secret(auth_secret);
        ret
    }
}

pub struct Server {
    cfg: Config,
    router: Router,
}

impl Server {
    pub fn new(cfg: Config, app_state: AppState) -> Server {
        Server {
            cfg: cfg.clone(),
            router: Server::new_stateless_router(cfg).with_state(app_state),
        }
    }

    pub fn new_stateless_router(cfg: Config) -> Router<AppState> {
        let auth_routes = Router::new()
            .route("/login", post(handlers::auth::post_login_init))
            .route("/login/complete", post(handlers::auth::login_complete))
            .route("/refresh", post(handlers::auth::post_refresh))
            .route("/register", post(handlers::auth::post_register))
            .route(
                "/register/complete",
                post(handlers::auth::post_register_complete),
            );

        let user_router = Router::new().route(
            "/:pubkey",
            get(handlers::users::get_user), //.patch(handlers::users::patch_user),
        );
        // .layer(tower_http::auth::AsyncRequireAuthorizationLayer::new(
        //     middleware::auth::AppAuth::new(auth_config),
        // ));
        let router = Router::new()
            .route("/", get(handlers::handler))
            .nest("/api/v1/user", user_router)
            .nest("/api/v1/auth", auth_routes);

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

use handlers::auth;
use jwt_simple::prelude::HS256Key;
use middleware::auth::AuthMiddlewareConfig;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
#[derive(Debug, Serialize, Deserialize)]
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
            crate::domain::error::Error::UserNotConfirmed => StatusCode::CONFLICT,
            crate::domain::error::Error::InvalidAuthToken => StatusCode::FORBIDDEN,
            crate::domain::error::Error::AuthTokenExpired => StatusCode::FORBIDDEN,
            crate::domain::error::Error::InvalidSignature => StatusCode::FORBIDDEN,
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
