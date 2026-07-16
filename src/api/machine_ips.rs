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
) -> Result<Json<Vec<MachineIpWithSerial>>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let items = state.pool.list_machine_ips().await?;
    Ok(Json(items))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Json(mut req): Json<CreateMachineIpRequest>,
) -> Result<Json<MachineIp>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    if req.ip_address.trim().is_empty() {
        return Err(AppError::Validation("IP address cannot be empty".into()));
    }
    if let Some(port) = req.port {
        if port < 1 || port > 65535 {
            return Err(AppError::Validation("Port must be between 1 and 65535".into()));
        }
    }
    if req.machine_id == 0 {
        if let Some(ref serial) = req.serial_number {
            let machine = state.pool.find_machine_by_serial(serial).await?;
            req.machine_id = match machine {
                Some(m) => m.id,
                None => state.pool.create_machine(serial).await?.id,
            };
        }
    }
    let item = state.pool.create_machine_ip(&req).await?;
    Ok(Json(item))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateMachineIpRequest>,
) -> Result<Json<MachineIp>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let item = state.pool
        .update_machine_ip(id, &req)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(item))
}

pub async fn list_machines(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<Json<Vec<Machine>>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let machines = state.pool.list_machines().await?;
    Ok(Json(machines))
}

pub async fn delete_ip(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role.to_lowercase() != "admin" {
        return Err(AppError::Forbidden);
    }
    let deleted = state.pool.delete_machine_ip(id).await?;
    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err(AppError::NotFound)
    }
}
