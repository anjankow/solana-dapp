mod app_state;
mod handlers;

use crate::server::app_state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub struct Server {}

impl Server {
    pub fn new() -> Server {
        Server {}
    }

    pub async fn run(&self, bind_address: &str) -> Result<(), std::io::Error> {
        let state = AppState::new();

        let router = Router::new()
            .route("/", get(handlers::handler))
            .route("/api/v1/users", post(handlers::users::post_user))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(bind_address).await?;

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
#[derive(Serialize)]
pub struct ErrorResp {
    #[serde(skip_serializing)]
    status_code: StatusCode,
    error: String,
}

impl From<crate::domain::error::Error> for ErrorResp {
    fn from(value: crate::domain::error::Error) -> Self {
        let status = match &value {
            crate::domain::error::Error::GeneralError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            crate::domain::error::Error::InvalidPubKey(_) => StatusCode::BAD_REQUEST,
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
