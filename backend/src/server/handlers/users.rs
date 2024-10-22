use axum::extract::{Json, Path, State};
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

#[derive(Serialize)]
pub struct GetUserResp {
    pubkey: String,
    username: String,
    nonce: u64,
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
) -> Result<Json<GetUserResp>, ErrorResp> {
    let user_mgr = state.get_user_mgr();

    let user = user_mgr.get_user(&pubkey)?;
    Ok(Json(GetUserResp {
        pubkey: user.pubkey.to_string(),
        username: user.username,
        nonce: user.nonce,
    }))
}
