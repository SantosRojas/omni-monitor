use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub active: bool,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub active: bool,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            full_name: u.full_name,
            email: u.email,
            role: u.role,
            active: u.active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Patient {
    pub id: i64,
    pub patient_id_str: String,
    pub created_at: Option<NaiveDateTime>,
    pub therapy_start: Option<NaiveDateTime>,
    pub therapy_end: Option<NaiveDateTime>,
    pub active_therapy_count: Option<i64>,
    pub completed_therapy_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Machine {
    pub id: i64,
    pub serial_number: String,
    pub software_version: String,
    pub registered_at: Option<NaiveDateTime>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Therapy {
    pub id: i64,
    pub started_at: Option<NaiveDateTime>,
    pub patient_id: Option<i64>,
    pub machine_id: Option<i64>,
    pub status: Option<String>,
    pub ended_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TherapyWithMachine {
    pub id: i64,
    pub started_at: Option<NaiveDateTime>,
    pub ended_at: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub machine_id: Option<i64>,
    pub serial_number: Option<String>,
    pub software_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct MachineIp {
    pub id: i64,
    pub machine_id: i64,
    pub ip_address: String,
    pub port: Option<i32>,
    pub label: Option<String>,
    pub is_active: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineIpWithSerial {
    pub id: i64,
    pub machine_id: i64,
    pub ip_address: String,
    pub port: Option<i32>,
    pub label: Option<String>,
    pub is_active: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub serial_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct AttributeEquivalence {
    pub signal_id: i64,
    pub numeric_value: i64,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct TelemetryReading {
    pub id: i64,
    pub timestamp: Option<NaiveDateTime>,
    pub therapy_id: Option<i64>,
    pub signal_id: Option<i64>,
    pub raw_value: Option<i64>,
    pub physical_value: Option<String>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct TelemetryExportRow {
    pub id: i64,
    pub timestamp: Option<NaiveDateTime>,
    pub signal_id: Option<i64>,
    pub physical_value: Option<String>,
    pub unit: Option<String>,
    pub signal_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Signal {
    pub id: i64,
    pub internal_name: String,
    pub display_name: Option<String>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct TherapyComment {
    pub id: i64,
    pub therapy_id: i64,
    pub author_name: String,
    pub comment: String,
    pub created_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
    pub deletion_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMachineIpRequest {
    pub machine_id: i64,
    pub ip_address: String,
    pub port: Option<i32>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMachineIpRequest {
    pub ip_address: Option<String>,
    pub port: Option<i32>,
    pub label: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub password: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveDevice {
    pub ip_address: String,
    pub port: i32,
    pub url: String,
    pub serial_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSignal {
    pub signal_id: i64,
    pub internal_name: String,
    pub display_name: Option<String>,
    pub unit: Option<String>,
    pub average: Option<f64>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub count: i64,
    pub values: Vec<DashboardValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardValue {
    pub timestamp: NaiveDateTime,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientDashboard {
    pub signals: Vec<DashboardSignal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardParams {
    pub signal_ids: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}
