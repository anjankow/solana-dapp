use axum::extract::{Json, Path, State};
use serde::{Deserialize, Serialize};

use crate::server::AppState;
use crate::server::ErrorResp;

#[derive(Deserialize)]
pub struct PostRegisterReq {
    pubkey: String,
    username: String,
}

#[derive(Serialize)]
pub struct PostRegisterResp {
    nonce: u64,
}

#[axum_macros::debug_handler]
pub async fn post_register(
    State(state): State<AppState>,
    Json(req): Json<PostRegisterReq>,
) -> Result<Json<PostRegisterResp>, ErrorResp> {
    let user = state
        .user_service
        .register_init(&req.pubkey, req.username)?;
    Ok(Json(PostRegisterResp { nonce: 0 })) // todo
}

#[derive(Serialize)]
pub struct LoginCompleteResp {
    // access_token used to access protected API endpoints
    access_token: String,
    // token_type e.g. Bearer
    token_type: String,
    // expires_in time in seconds
    expires_in: u64,
    // nonce is used to refresh the access token once it expires
    nonce: u64,
}

#[axum_macros::debug_handler]
pub async fn post_register_complete(
    State(state): State<AppState>,
    Json(req): Json<PostRegisterReq>,
) -> Result<Json<LoginCompleteResp>, ErrorResp> {
    // let user = user_service.create_user(&req.pubkey, req.username)?;
    Ok(Json(LoginCompleteResp {
        access_token: todo!(),
        token_type: todo!(),
        expires_in: todo!(),
        nonce: todo!(),
    }))
}
