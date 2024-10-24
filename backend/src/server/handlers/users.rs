use axum::extract::{Json, Path, State};
use serde::{Deserialize, Serialize};

use crate::server::AppState;
use crate::server::ErrorResp;

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
    let user = state.user_service.get_user(&pubkey)?;
    Ok(Json(GetUserResp {
        pubkey: user.pubkey.to_string(),
        username: user.username,
        nonce: 0, // todo
    }))
}
