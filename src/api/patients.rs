use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};

use crate::models::*;
use crate::auth::JwtClaims;
use super::{AppError, AppState};

pub async fn list_patients(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Patient>>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let result = state.pool
        .list_patients(page, per_page, params.search.as_deref())
        .await?;
    Ok(Json(result))
}

pub async fn get_patient(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Result<Json<Patient>, AppError> {
    let patient = state.pool
        .find_patient_by_id(id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(patient))
}

pub async fn get_therapies(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<TherapyWithMachine>>, AppError> {
    let therapies = state.pool.list_therapies_by_patient(id).await?;
    Ok(Json(therapies))
}

pub async fn get_history(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<TelemetryReading>>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).clamp(1, 500);
    let result = state.pool
        .list_telemetry(id, page, per_page, None, None, None)
        .await?;
    Ok(Json(result))
}

pub async fn get_active_device(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Result<Json<ActiveDevice>, AppError> {
    let device = state.pool.find_active_device(id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(device))
}
