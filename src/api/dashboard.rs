use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};

use crate::models::*;
use crate::auth::JwtClaims;
use super::{AppError, AppState};

pub async fn patient_dashboard(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
    Query(params): Query<DashboardParams>,
) -> Result<Json<PatientDashboard>, AppError> {
    let signal_ids: Option<Vec<i64>> = params
        .signal_ids
        .as_deref()
        .map(|s| s.split(',').filter_map(|n| n.trim().parse().ok()).collect());
    let dashboard = state.pool
        .patient_dashboard(
            id,
            signal_ids.as_deref(),
            params.from.as_deref(),
            params.to.as_deref(),
        )
        .await?;
    Ok(Json(dashboard))
}

pub async fn therapy_dashboard(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Result<Json<PatientDashboard>, AppError> {
    let dashboard = state.pool.therapy_dashboard(id).await?;
    Ok(Json(dashboard))
}
