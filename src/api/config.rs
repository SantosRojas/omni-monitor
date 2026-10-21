use axum::{Extension, Json, extract::State};
use serde::Serialize;

use super::AppState;
use crate::auth::JwtClaims;

#[derive(Serialize)]
pub struct PublicConfig {
    pub polling_interval_ms: u64,
}

pub async fn get_config(State(state): State<AppState>, Extension(_claims): Extension<JwtClaims>) -> Json<PublicConfig> {
    Json(PublicConfig {
        polling_interval_ms: state.config.polling_interval_ms,
    })
}
