use axum::extract::{Json, Path, State};
use serde::Serialize;

use crate::server::AppState;
use crate::server::ErrorResp;

use super::parse_pubkey;

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
    let pubkey = parse_pubkey(&pubkey)?;
    let user = state.user_service.get_user(&pubkey)?;
    Ok(Json(GetUserResp {
        pubkey: user.pubkey.to_string(),
        username: user.username,
        nonce: 0, // todo
    }))
}
