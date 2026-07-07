use axum::{
    extract::{Path, State},
    Extension, Json,
};

use crate::models::*;
use crate::auth::JwtClaims;
use super::{AppError, AppState};

pub async fn list(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<Json<Vec<Signal>>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let items = state.pool.list_signals().await?;
    Ok(Json(items))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSignalRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    state.pool.update_signal(
        id,
        req.display_name.as_deref(),
        req.unit.as_deref(),
    ).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
