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
    if req.username.trim().is_empty() {
        return Err(AppError::Validation("Username cannot be empty".into()));
    }
    if req.password.len() < 6 {
        return Err(AppError::Validation("Password must be at least 6 characters".into()));
    }
    if req.full_name.trim().is_empty() {
        return Err(AppError::Validation("Full name cannot be empty".into()));
    }
    if !req.email.contains('@') {
        return Err(AppError::Validation("Invalid email address".into()));
    }
    let role_lower = req.role.to_lowercase();
    if role_lower != "admin" && role_lower != "user" && role_lower != "viewer" {
        return Err(AppError::Validation("Role must be admin, user, or viewer".into()));
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
