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
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<TherapyWithMachine>>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let result = state.pool.list_therapies_by_patient_paginated(id, page, per_page).await?;
    Ok(Json(result))
}

pub async fn get_history(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<TelemetryReading>>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).clamp(1, 500);
    let signal_ids: Option<Vec<i64>> = params
        .signal_ids
        .as_deref()
        .map(|s| s.split(',').filter_map(|n| n.trim().parse().ok()).collect());
    let result = state.pool
        .list_telemetry(id, page, per_page, signal_ids.as_deref(), params.from.as_deref(), params.to.as_deref())
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

pub async fn get_therapy(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Result<Json<TherapyWithMachine>, AppError> {
    let therapy = state.pool
        .find_therapy_by_id(id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(therapy))
}

pub async fn list_active_therapies(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
) -> Result<Json<Vec<ActiveTherapy>>, AppError> {
    let therapies = state.pool.list_active_therapies().await?;
    Ok(Json(therapies))
}
