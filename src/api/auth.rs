use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Extension, Json,
};

use crate::auth::{self as auth_utils, JwtClaims};
use crate::models::*;
use super::{AppError, AppState};

pub async fn generate_authorization_code(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Json(req): Json<GenerateTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    if req.user_id <= 0 {
        return Err(AppError::Validation("Invalid user_id".into()));
    }
    state.pool
        .find_user_by_id(claims.user_id)
        .await
        .map_err(|e| AppError::Database(format!("find_user_by_id: {}", e)))?
        .ok_or(AppError::NotFound)?;

    let code = uuid::Uuid::new_v4().to_string();

    state.pool
        .create_authorization_code(&code, req.user_id, req.expires_at)
        .await
        .map_err(|e| AppError::Database(format!("create_authorization_code: {}", e)))?;

    Ok(Json(GenerateTokenResponse { code }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
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

    let max_age = state.config.jwt_expiration_hours * 3600;
    let cookie = format!(
        "monitor_token={}; HttpOnly; SameSite=Strict; Path=/api; Max-Age={}",
        token, max_age
    );

    let mut headers = HeaderMap::new();
    headers.insert(axum::http::header::SET_COOKIE, HeaderValue::from_str(&cookie).map_err(|e| AppError::Internal(format!("Invalid cookie header: {}", e)))?);

    Ok((headers, Json(LoginResponse {
        user: UserResponse::from(user),
    })))
}

pub async fn logout() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_static("monitor_token=; HttpOnly; SameSite=Lax; Path=/api; Max-Age=0"),
    );
    (headers, StatusCode::OK)
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
