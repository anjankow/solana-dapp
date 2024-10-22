use axum::extract::{Json, State};
use axum::Error;
use serde::{Deserialize, Serialize};

use crate::server::AppState;
use crate::server::ErrorResp;

#[derive(Deserialize)]
pub struct PostUserReq {
    pubkey: String,
    username: String,
}

#[derive(Serialize)]
pub struct PostUserResp {
    nonce: u64,
}

#[axum_macros::debug_handler]
pub async fn post_user(
    State(state): State<AppState>,
    Json(req): Json<PostUserReq>,
) -> Result<Json<PostUserResp>, ErrorResp> {
    let user_mgr = state.get_user_mgr();

    let user = user_mgr.create_user(&req.pubkey, req.username)?;
    Ok(Json(PostUserResp { nonce: user.nonce }))
}

struct GetUserNonceResp {
    pubkey: String,
    nonce: u64,
}

pub async fn get_user_nonce() -> Result<axum::response::Json<GetUserNonceResp>, Error> {
    todo!()
}
