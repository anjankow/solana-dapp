use axum::extract::{Json, Path, State};
use serde::{Deserialize, Serialize};

use crate::server::middleware::auth::AuthPubkey;
use crate::server::AppState;
use crate::server::ErrorResp;

use super::parse_pubkey;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUserResp {
    pub owned: bool,
    pub pubkey: String,
    pub username: String,
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
    AuthPubkey(auth_pubkey): AuthPubkey,
) -> Result<Json<GetUserResp>, ErrorResp> {
    let pubkey = parse_pubkey(&pubkey)?;
    let user = state.user_service.get_user(&pubkey)?;

    Ok(Json(GetUserResp {
        owned: auth_pubkey.is_some_and(|a| a == pubkey),
        pubkey: user.pubkey.to_string(),
        username: user.username,
    }))
}
