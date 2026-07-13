use axum::{extract::State, Extension, Json};
use serde::Serialize;

use crate::auth::JwtClaims;
use super::AppState;

#[derive(Serialize)]
pub struct PublicConfig {
    pub polling_interval_ms: u64,
}

pub async fn get_config(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
) -> Json<PublicConfig> {
    Json(PublicConfig {
        polling_interval_ms: state.config.polling_interval_ms,
    })
}
