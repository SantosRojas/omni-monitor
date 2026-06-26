use axum::{
    extract::State,
    Extension, Json,
};

use crate::auth::{self as auth_utils, JwtClaims};
use crate::models::*;
use super::{AppError, AppState};

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = state.pool
        .find_user_by_username(&req.username)
        .await
        .map_err(|e| AppError::Database(format!("find_user_by_username: {}", e)))?
        .ok_or(AppError::Unauthorized)?;

    if !user.active {
        return Err(AppError::InactiveUser);
    }

    if !auth_utils::verify_password(&user.password, &req.password) {
        return Err(AppError::Unauthorized);
    }

    let token = auth_utils::issue_token(
        &user,
        &state.config.jwt_secret,
        state.config.jwt_expiration_hours,
        &state.config.jwt_issuer,
    )
    .map_err(|_| AppError::TokenIssuance)?;

    Ok(Json(LoginResponse {
        token,
        user: UserResponse::from(user),
    }))
}

pub async fn me(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.pool
        .find_user_by_id(claims.user_id)
        .await
        .map_err(|e| AppError::Database(format!("find_user_by_id: {}", e)))?
        .ok_or(AppError::NotFound)?;

    Ok(Json(UserResponse::from(user)))
}
