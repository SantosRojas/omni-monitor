pub mod auth;
pub mod dashboard;
pub mod equivalences;
pub mod export;
pub mod machine_ips;
pub mod patients;
pub mod signals;
pub mod users;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use crate::config::MonitorConfig;
use crate::database::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: std::sync::Arc<DbPool>,
    pub config: std::sync::Arc<MonitorConfig>,
}

#[derive(Debug)]
pub enum AppError {
    NotFound,
    Forbidden,
    Unauthorized,
    InactiveUser,
    Database(String),
    Export(String),
    TokenIssuance,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            Self::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()),
            Self::InactiveUser => (StatusCode::FORBIDDEN, "User is inactive".to_string()),
            Self::Database(e) => {
                tracing::error!(error = %e, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            Self::Export(e) => {
                tracing::error!(error = %e, "Export error");
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Export error: {}", e))
            }
            Self::TokenIssuance => (StatusCode::INTERNAL_SERVER_ERROR, "Token issuance failed".to_string()),
        };
        (status, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e.to_string())
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: middleware::Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Missing token"}))))?;

    let claims = crate::auth::decode_token(token, &state.config.jwt_secret, &state.config.jwt_issuer)
        .map_err(|_| (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid token"}))))?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

pub fn create_router(state: AppState) -> Router {
    let public = Router::new()
        .route("/auth/login", post(auth::login))
        .with_state(state.clone());

    let protected = Router::new()
        .route("/auth/me", get(auth::me))
        .route("/patients", get(patients::list_patients))
        .route("/patients/{id}", get(patients::get_patient))
        .route("/patients/{id}/therapies", get(patients::get_therapies))
        .route("/patients/{id}/history", get(patients::get_history))
        .route("/patients/{id}/active-device", get(patients::get_active_device))
        .route("/patients/{id}/export", get(export::export_patient))
        .route("/patients/{id}/dashboard", get(dashboard::patient_dashboard))
        .route("/therapies/{id}/dashboard", get(dashboard::therapy_dashboard))
        .route("/therapies/{id}/export", get(export::export_therapy))
        .route("/machines", get(machine_ips::list_machines))
        .route("/admin/machine-ips", get(machine_ips::list).post(machine_ips::create))
        .route("/admin/machine-ips/{id}", put(machine_ips::update).delete(machine_ips::delete_ip))
        .route("/admin/equivalences", get(equivalences::list).post(equivalences::create).put(equivalences::update))
        .route("/admin/equivalences/{signal_id}/{numeric_value}", delete(equivalences::delete))
        .route("/admin/signals", get(signals::list))
        .route("/admin/signals/{id}", put(signals::update))
        .route("/users", get(users::list).post(users::create))
        .route("/users/{id}", put(users::update).delete(users::delete_user))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state);

    Router::new()
        .merge(public)
        .merge(protected)
}
