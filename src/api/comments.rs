use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::Deserialize;

use crate::auth::JwtClaims;
use crate::models::TherapyComment;
use super::{AppError, AppState};

#[derive(Deserialize)]
pub struct CreateCommentRequest {
    pub comment: String,
}

#[derive(Deserialize)]
pub struct DeleteCommentRequest {
    pub deletion_reason: String,
}

pub async fn list_comments(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path(therapy_id): Path<i64>,
) -> Result<Json<Vec<TherapyComment>>, AppError> {
    let comments = state.pool.list_therapy_comments(therapy_id).await?;
    Ok(Json(comments))
}

pub async fn create_comment(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Path(therapy_id): Path<i64>,
    Json(body): Json<CreateCommentRequest>,
) -> Result<Json<TherapyComment>, AppError> {
    let comment = state.pool.create_therapy_comment(therapy_id, &claims.full_name, &body.comment).await?;
    Ok(Json(comment))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path((_therapy_id, comment_id)): Path<(i64, i64)>,
    Json(body): Json<DeleteCommentRequest>,
) -> Result<Json<()>, AppError> {
    state.pool.delete_therapy_comment(comment_id, &body.deletion_reason).await?;
    Ok(Json(()))
}
