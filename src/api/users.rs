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
) -> Result<Json<Vec<UserResponse>>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let users = state.pool.list_users().await?;
    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let user = state.pool.create_user(&req).await?;
    Ok(Json(UserResponse::from(user)))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let user = state.pool
        .update_user(id, &req)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(UserResponse::from(user)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let deleted = state.pool.delete_user(id).await?;
    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err(AppError::NotFound)
    }
}
