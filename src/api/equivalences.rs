use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;

use crate::models::*;
use crate::auth::JwtClaims;
use super::{AppError, AppState};

pub async fn list(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<Json<Vec<EquivalenceResponse>>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let items = state.pool.list_equivalences_with_signals().await?;
    Ok(Json(items))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Json(req): Json<CreateEquivalenceRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let signal_id = state.pool.get_or_create_signal(&req.internal_name).await?;
    state.pool.upsert_equivalence(signal_id, req.numeric_value, &req.display_name).await?;
    Ok(Json(serde_json::json!({"ok": true, "signal_id": signal_id})))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Json(req): Json<UpdateEquivalenceRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    state.pool.update_equivalence(req.signal_id, req.numeric_value, &req.display_name).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
pub struct DeleteEquivalenceParams {
    pub deletion_reason: Option<String>,
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Path((signal_id, numeric_value)): Path<(i64, f64)>,
    Query(params): Query<DeleteEquivalenceParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let reason = params.deletion_reason.unwrap_or_default();
    state.pool.delete_equivalence_with_log(
        signal_id,
        numeric_value,
        &claims.username,
        &reason,
    ).await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}
