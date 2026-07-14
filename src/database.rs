use std::collections::HashMap;
use std::str::FromStr;
use chrono::NaiveDateTime;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    postgres::{PgConnectOptions, PgPoolOptions},
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    AssertSqlSafe, ConnectOptions, MySql, Pool, Sqlite,
};
use tracing::log::LevelFilter;

use crate::config::MonitorConfig;
use crate::models::*;

#[cfg(feature = "ssr")]
use bb8::Pool as Bb8Pool;
#[cfg(feature = "ssr")]
use bb8_tiberius::ConnectionManager;
#[cfg(feature = "ssr")]
use tiberius::{AuthMethod, Client, Config, Row};
#[cfg(feature = "ssr")]
use tokio::net::TcpStream;
#[cfg(feature = "ssr")]
use tokio_util::compat::TokioAsyncWriteCompatExt;

#[derive(Debug, Clone)]
pub enum DbPool {
    NoDb,
    Sqlite(Pool<Sqlite>),
    Postgres(Pool<sqlx::Postgres>),
    Mysql(Pool<MySql>),
    Mssql(MssqlDb),
}

#[derive(Debug, Clone)]
pub struct MssqlDb {
    pool: Bb8Pool<ConnectionManager>,
}

trait TryFromRow: Sized {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error>;
}

impl MssqlDb {
    pub fn new(pool: Bb8Pool<ConnectionManager>) -> Self {
        Self { pool }
    }

    async fn conn(&self) -> Result<bb8::PooledConnection<'_, ConnectionManager>, sqlx::Error> {
        self.pool.get().await.map_err(|e| sqlx::Error::Protocol(e.to_string()))
    }

    async fn query_one<T: TryFromRow>(&self, sql: &str, params: &[&dyn tiberius::ToSql]) -> Result<Option<T>, sqlx::Error> {
        let mut conn = self.conn().await?;
        let stream = conn.query(sql, params).await.map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let row = stream.into_row().await.map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        row.as_ref().map(|r| T::try_from_row(r)).transpose()
    }

    async fn query_all<T: TryFromRow>(&self, sql: &str, params: &[&dyn tiberius::ToSql]) -> Result<Vec<T>, sqlx::Error> {
        let mut conn = self.conn().await?;
        let stream = conn.query(sql, params).await.map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let rows = stream.into_first_result().await.map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        rows.iter().map(|r| T::try_from_row(r)).collect()
    }

    async fn execute(&self, sql: &str, params: &[&dyn tiberius::ToSql]) -> Result<u64, sqlx::Error> {
        let mut conn = self.conn().await?;
        conn.execute(sql, params).await.map_err(|e| sqlx::Error::Protocol(e.to_string())).map(|r| r.total())
    }

    async fn simple_query(&self, sql: &str) -> Result<(), sqlx::Error> {
        let mut conn = self.conn().await?;
        conn.simple_query(sql).await.map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        Ok(())
    }

    async fn query_scalar<T: for<'a> tiberius::FromSql<'a>>(&self, sql: &str, params: &[&dyn tiberius::ToSql]) -> Result<T, sqlx::Error> {
        let mut conn = self.conn().await?;
        let stream = conn.query(sql, params).await.map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let row = stream.into_row().await.map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        row.as_ref()
            .and_then(|r| r.get::<T, usize>(0))
            .ok_or_else(|| sqlx::Error::Protocol("expected a scalar value but got NULL or no rows".into()))
    }
}

// --- Tiberius row helpers ---
fn col_val<'a, T: tiberius::FromSql<'a>>(row: &'a Row, name: &str) -> Result<T, sqlx::Error> {
    match row.try_get::<T, &str>(name) {
        Ok(Some(v)) => Ok(v),
        Ok(None) => Err(sqlx::Error::Protocol(format!("null column '{}'", name))),
        Err(e) => Err(sqlx::Error::Protocol(format!("column '{}': {}", name, e))),
    }
}

fn col_opt<'a, T: tiberius::FromSql<'a>>(row: &'a Row, name: &str) -> Result<Option<T>, sqlx::Error> {
    match row.try_get::<T, &str>(name) {
        Ok(v) => Ok(v),
        Err(e) => Err(sqlx::Error::Protocol(format!("column '{}': {}", name, e))),
    }
}

fn col_str(row: &Row, name: &str) -> Result<String, sqlx::Error> {
    match row.try_get::<&str, &str>(name) {
        Ok(Some(s)) => Ok(s.to_string()),
        Ok(None) => Err(sqlx::Error::Protocol(format!("null column '{}'", name))),
        Err(e) => Err(sqlx::Error::Protocol(format!("column '{}': {}", name, e))),
    }
}

fn col_opt_str(row: &Row, name: &str) -> Result<Option<String>, sqlx::Error> {
    match row.try_get::<&str, &str>(name) {
        Ok(v) => Ok(v.map(|s| s.to_string())),
        Err(e) => Err(sqlx::Error::Protocol(format!("column '{}': {}", name, e))),
    }
}

fn col_bool<'a>(row: &'a Row, name: &str) -> Result<bool, sqlx::Error> {
    if let Ok(Some(v)) = row.try_get::<i32, &str>(name) {
        Ok(v != 0)
    } else if let Ok(Some(v)) = row.try_get::<bool, &str>(name) {
        Ok(v)
    } else {
        Err(sqlx::Error::Protocol(format!("missing column '{}'", name)))
    }
}

#[allow(dead_code)]
fn col_opt_bool<'a>(row: &'a Row, name: &str) -> Result<Option<bool>, sqlx::Error> {
    match row.try_get::<i32, &str>(name) {
        Ok(Some(v)) => Ok(Some(v != 0)),
        Ok(None) => Ok(None),
        Err(_) => match row.try_get::<bool, &str>(name) {
            Ok(v) => Ok(v),
            Err(e) => Err(sqlx::Error::Protocol(format!("column '{}': {}", name, e))),
        },
    }
}

fn col_opt_dt<'a>(row: &'a Row, name: &str) -> Result<Option<NaiveDateTime>, sqlx::Error> {
    match row.try_get::<NaiveDateTime, &str>(name) {
        Ok(v) => Ok(v),
        Err(e) => Err(sqlx::Error::Protocol(format!("column '{}': {}", name, e))),
    }
}

fn col_i64(row: &Row, name: &str) -> Result<i64, sqlx::Error> {
    if let Ok(Some(v)) = row.try_get::<i32, &str>(name) {
        Ok(v as i64)
    } else if let Ok(Some(v)) = row.try_get::<i64, &str>(name) {
        Ok(v)
    } else {
        Err(sqlx::Error::Protocol(format!("missing column '{}'", name)))
    }
}

fn col_opt_i64(row: &Row, name: &str) -> Result<Option<i64>, sqlx::Error> {
    if let Ok(Some(v)) = row.try_get::<i32, &str>(name) {
        Ok(Some(v as i64))
    } else if let Ok(Some(v)) = row.try_get::<i64, &str>(name) {
        Ok(Some(v))
    } else {
        Ok(None)
    }
}

// --- TryFromRow implementations for tiberius ---
impl TryFromRow for User {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: col_i64(row, "id")?,
            username: col_str(row, "username")?,
            password: col_str(row, "password")?,
            full_name: col_str(row, "full_name")?,
            email: col_str(row, "email")?,
            role: col_str(row, "role")?,
            active: col_bool(row, "active").unwrap_or(false),
            created_at: col_opt::<NaiveDateTime>(row, "created_at")?,
        })
    }
}

impl TryFromRow for Patient {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: col_i64(row, "id")?,
            patient_id_str: col_str(row, "patient_id_str")?,
            created_at: col_opt::<NaiveDateTime>(row, "created_at")?,
            therapy_start: col_opt::<NaiveDateTime>(row, "therapy_start")?,
            therapy_end: col_opt::<NaiveDateTime>(row, "therapy_end")?,
            active_therapy_count: col_opt_i64(row, "active_therapy_count")?,
            completed_therapy_count: col_opt_i64(row, "completed_therapy_count")?,
        })
    }
}

impl TryFromRow for Machine {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: col_i64(row, "id")?,
            serial_number: col_str(row, "serial_number")?,
            software_version: col_str(row, "software_version")?,
            registered_at: col_opt_dt(row, "registered_at")?,
            status: col_opt_str(row, "status")?,
        })
    }
}

impl TryFromRow for MachineIp {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: col_i64(row, "id")?,
            machine_id: col_i64(row, "machine_id")?,
            ip_address: col_str(row, "ip_address")?,
            port: col_val::<i32>(row, "port").ok(),
            label: col_opt_str(row, "label")?,
            is_active: col_bool(row, "is_active").unwrap_or(true),
            created_at: col_opt_dt(row, "created_at")?,
            updated_at: col_opt_dt(row, "updated_at")?,
        })
    }
}

impl TryFromRow for Signal {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: col_i64(row, "id")?,
            internal_name: col_str(row, "internal_name")?,
            display_name: col_opt_str(row, "display_name")?,
            unit: col_opt_str(row, "unit")?,
        })
    }
}

impl TryFromRow for AttributeEquivalence {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            signal_id: col_i64(row, "signal_id")?,
            numeric_value: col_val::<f64>(row, "numeric_value")?,
            display_name: col_str(row, "display_name")?,
        })
    }
}

impl TryFromRow for EquivalenceResponse {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            signal_id: col_i64(row, "signal_id")?,
            internal_name: col_str(row, "internal_name")?,
            numeric_value: col_val::<f64>(row, "numeric_value")?,
            display_name: col_str(row, "display_name")?,
        })
    }
}

impl TryFromRow for TelemetryReading {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: col_i64(row, "id")?,
            timestamp: col_opt_dt(row, "timestamp")?,
            therapy_id: col_opt_i64(row, "therapy_id")?,
            signal_id: col_opt_i64(row, "signal_id")?,
            raw_value: col_opt_i64(row, "raw_value")?,
            physical_value: col_opt_str(row, "physical_value")?,
            unit: col_opt_str(row, "unit")?,
            signal_name: col_opt_str(row, "signal_name")?,
        })
    }
}

impl TryFromRow for TelemetryExportRow {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: col_i64(row, "id")?,
            timestamp: col_opt_dt(row, "timestamp")?,
            signal_id: col_opt_i64(row, "signal_id")?,
            physical_value: col_opt_str(row, "physical_value")?,
            unit: col_opt_str(row, "unit")?,
            signal_name: col_opt_str(row, "signal_name")?,
        })
    }
}

impl TryFromRow for TherapyComment {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: col_i64(row, "id")?,
            therapy_id: col_i64(row, "therapy_id")?,
            author_name: col_str(row, "author_name")?,
            comment: col_str(row, "comment")?,
            created_at: col_opt_dt(row, "created_at")?,
            deleted_at: col_opt_dt(row, "deleted_at")?,
            deletion_reason: col_opt_str(row, "deletion_reason")?,
        })
    }
}

impl TryFromRow for AuthorizationCode {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            code: col_str(row, "code")?,
            user_id: col_i64(row, "user_id")?,
            expires_at: col_opt_dt(row, "expires_at")?,
            used: col_bool(row, "used").unwrap_or(false),
        })
    }
}

impl TryFromRow for TherapyRaw {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: col_i64(row, "id")?,
            started_at: col_opt_dt(row, "started_at")?,
            patient_id: col_opt_i64(row, "patient_id")?,
            machine_id: col_opt_i64(row, "machine_id")?,
            status: col_opt_str(row, "status")?,
            ended_at: col_opt_dt(row, "ended_at")?,
            serial_number: col_opt_str(row, "serial_number")?,
            software_version: col_opt_str(row, "software_version")?,
            ip_address: col_opt_str(row, "ip_address")?,
            port: col_val::<i32>(row, "port").ok(),
            therapy_type: col_opt_str(row, "therapy_type")?,
            kit: col_opt_str(row, "kit")?,
            weight_initial: col_opt_str(row, "weight_initial")?,
            weight_final: col_opt_str(row, "weight_final")?,
            therapy_type_signal_id: col_opt_i64(row, "therapy_type_signal_id")?,
            kit_signal_id: col_opt_i64(row, "kit_signal_id")?,
            weight_initial_signal_id: col_opt_i64(row, "weight_initial_signal_id")?,
            weight_final_signal_id: col_opt_i64(row, "weight_final_signal_id")?,
            patient_id_str: col_opt_str(row, "patient_id_str")?,
        })
    }
}

impl TryFromRow for ActiveDeviceRaw {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            ip_address: col_str(row, "ip_address")?,
            port: col_val::<i32>(row, "port").ok(),
            serial_number: col_opt_str(row, "serial_number")?,
        })
    }
}

impl TryFromRow for MachineIpWithSerialRaw {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: col_i64(row, "id")?,
            machine_id: col_i64(row, "machine_id")?,
            ip_address: col_str(row, "ip_address")?,
            port: col_val::<i32>(row, "port").ok(),
            label: col_opt_str(row, "label")?,
            is_active: col_bool(row, "is_active").unwrap_or(true),
            created_at: col_opt_dt(row, "created_at")?,
            updated_at: col_opt_dt(row, "updated_at")?,
            serial_number: col_opt_str(row, "serial_number")?,
        })
    }
}

impl TryFromRow for DashboardSignalRaw {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            signal_id: col_i64(row, "signal_id")?,
            internal_name: col_str(row, "internal_name")?,
            display_name: col_opt_str(row, "display_name")?,
            unit: col_opt_str(row, "unit")?,
            average: col_val::<f64>(row, "average").ok(),
            minimum: col_val::<f64>(row, "minimum").ok(),
            maximum: col_val::<f64>(row, "maximum").ok(),
            count: col_i64(row, "count")?,
        })
    }
}

impl TryFromRow for DashboardValueWithSignal {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            signal_id: col_i64(row, "signal_id")?,
            timestamp: col_opt_dt(row, "timestamp")?,
            physical_value: col_opt_str(row, "physical_value")?,
        })
    }
}

impl TryFromRow for (i64,) {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok((col_i64(row, "")?,)) // single column, use index 0
    }
}

macro_rules! tp {
    ($($x:expr),* $(,)?) => {
        &[$(&$x as &dyn tiberius::ToSql),*] as &[&dyn tiberius::ToSql]
    };
}

impl DbPool {
    pub async fn from_config(config: &MonitorConfig) -> Result<Self, sqlx::Error> {
        match config.db_connection.to_lowercase().as_str() {
            "sqlite" => {
                let url = format!("sqlite:{}?mode=rwc", config.db_database);
                let opts = SqliteConnectOptions::from_str(&url)?
                    .create_if_missing(true)
                    .log_statements(LevelFilter::Debug);
                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect_with(opts)
                    .await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS users (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        username TEXT NOT NULL UNIQUE,
                        password TEXT NOT NULL,
                        full_name TEXT NOT NULL,
                        email TEXT NOT NULL,
                        role TEXT NOT NULL,
                        active INTEGER NOT NULL DEFAULT 1,
                        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS machines (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        serial_number TEXT NOT NULL,
                        software_version TEXT NOT NULL,
                        registered_at DATETIME,
                        status TEXT
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS signals (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        internal_name TEXT NOT NULL UNIQUE,
                        display_name TEXT,
                        unit TEXT
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS attribute_equivalences (
                        signal_id INTEGER NOT NULL,
                        numeric_value REAL NOT NULL,
                        display_name TEXT NOT NULL,
                        PRIMARY KEY (signal_id, numeric_value)
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS patients (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        patient_id_str TEXT NOT NULL,
                        created_at DATETIME
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS therapies (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        started_at DATETIME,
                        patient_id INTEGER,
                        machine_id INTEGER,
                        status TEXT,
                        ended_at DATETIME
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS telemetry (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        timestamp DATETIME,
                        therapy_id INTEGER,
                        signal_id INTEGER,
                        raw_value INTEGER,
                        physical_value TEXT,
                        unit TEXT
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS machine_ips (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        machine_id INTEGER NOT NULL,
                        ip_address TEXT NOT NULL,
                        port INTEGER DEFAULT NULL,
                        label TEXT,
                        is_active INTEGER DEFAULT 1,
                        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                        FOREIGN KEY (machine_id) REFERENCES machines(id)
                    )",
                )
                .execute(&pool)
                .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_machine_ips_machine ON machine_ips(machine_id)")
                    .execute(&pool).await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_machine_ips_active ON machine_ips(machine_id, is_active)")
                    .execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS equivalence_deletion_log (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        signal_id INTEGER NOT NULL,
                        numeric_value REAL NOT NULL,
                        deleted_by TEXT NOT NULL,
                        deletion_reason TEXT NOT NULL,
                        deleted_at DATETIME DEFAULT CURRENT_TIMESTAMP
                    )",
                )
                .execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS therapy_comments (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        therapy_id INTEGER NOT NULL,
                        author_name TEXT NOT NULL,
                        comment TEXT NOT NULL,
                        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                        deleted_at DATETIME,
                        deletion_reason TEXT
                    )",
                )
                .execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS authorization_codes (
                        code TEXT PRIMARY KEY,
                        user_id INTEGER NOT NULL,
                        expires_at DATETIME,
                        used INTEGER NOT NULL DEFAULT 0
                    )",
                )
                .execute(&pool).await?;
                Ok(Self::Sqlite(pool))
            }
            "postgres" | "pgsql" | "postgresql" => {
                let opts = PgConnectOptions::new()
                    .host(&config.db_host)
                    .port(config.db_port)
                    .username(&config.db_username)
                    .password(&config.db_password)
                    .database(&config.db_database)
                    .log_statements(LevelFilter::Debug);
                let pool = PgPoolOptions::new()
                    .max_connections(10)
                    .connect_with(opts)
                    .await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS users (
                        id BIGSERIAL PRIMARY KEY,
                        username TEXT NOT NULL UNIQUE,
                        password TEXT NOT NULL,
                        full_name TEXT NOT NULL,
                        email TEXT NOT NULL,
                        role TEXT NOT NULL,
                        active BOOLEAN NOT NULL DEFAULT TRUE,
                        created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS machines (
                        id BIGSERIAL PRIMARY KEY,
                        serial_number TEXT NOT NULL,
                        software_version TEXT NOT NULL,
                        registered_at TIMESTAMPTZ,
                        status TEXT
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS signals (
                        id BIGSERIAL PRIMARY KEY,
                        internal_name TEXT NOT NULL UNIQUE,
                        display_name TEXT,
                        unit TEXT
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS attribute_equivalences (
                        signal_id BIGINT NOT NULL,
                        numeric_value DOUBLE PRECISION NOT NULL,
                        display_name TEXT NOT NULL,
                        PRIMARY KEY (signal_id, numeric_value)
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS patients (
                        id BIGSERIAL PRIMARY KEY,
                        patient_id_str TEXT NOT NULL,
                        created_at TIMESTAMPTZ
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS therapies (
                        id BIGSERIAL PRIMARY KEY,
                        started_at TIMESTAMPTZ,
                        patient_id BIGINT,
                        machine_id BIGINT,
                        status TEXT,
                        ended_at TIMESTAMPTZ
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS telemetry (
                        id BIGSERIAL PRIMARY KEY,
                        timestamp TIMESTAMPTZ,
                        therapy_id BIGINT,
                        signal_id BIGINT,
                        raw_value BIGINT,
                        physical_value TEXT,
                        unit TEXT
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS machine_ips (
                        id BIGSERIAL PRIMARY KEY,
                        machine_id BIGINT NOT NULL REFERENCES machines(id),
                        ip_address TEXT NOT NULL,
                        port INTEGER DEFAULT NULL,
                        label TEXT,
                        is_active BOOLEAN DEFAULT TRUE,
                        created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
                    )",
                )
                .execute(&pool)
                .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_machine_ips_machine ON machine_ips(machine_id)")
                    .execute(&pool).await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_machine_ips_active ON machine_ips(machine_id, is_active)")
                    .execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS equivalence_deletion_log (
                        id BIGSERIAL PRIMARY KEY,
                        signal_id BIGINT NOT NULL,
                        numeric_value DOUBLE PRECISION NOT NULL,
                        deleted_by TEXT NOT NULL,
                        deletion_reason TEXT NOT NULL,
                        deleted_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
                    )",
                )
                .execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS therapy_comments (
                        id BIGSERIAL PRIMARY KEY,
                        therapy_id BIGINT NOT NULL,
                        author_name TEXT NOT NULL,
                        comment TEXT NOT NULL,
                        created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                        deleted_at TIMESTAMPTZ,
                        deletion_reason TEXT
                    )",
                )
                .execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS authorization_codes (
                        code TEXT PRIMARY KEY,
                        user_id BIGINT NOT NULL,
                        expires_at TIMESTAMPTZ,
                        used BOOLEAN NOT NULL DEFAULT FALSE
                    )",
                )
                .execute(&pool).await?;
                Ok(Self::Postgres(pool))
            }
            "mysql" | "mariadb" => {
                let opts = MySqlConnectOptions::new()
                    .host(&config.db_host)
                    .port(config.db_port)
                    .username(&config.db_username)
                    .password(&config.db_password)
                    .database(&config.db_database)
                    .log_statements(LevelFilter::Debug);
                let pool = MySqlPoolOptions::new()
                    .max_connections(10)
                    .connect_with(opts)
                    .await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS users (
                        id BIGINT AUTO_INCREMENT PRIMARY KEY,
                        username VARCHAR(255) NOT NULL UNIQUE,
                        password VARCHAR(255) NOT NULL,
                        full_name VARCHAR(255) NOT NULL,
                        email VARCHAR(255) NOT NULL,
                        role VARCHAR(50) NOT NULL,
                        active TINYINT(1) NOT NULL DEFAULT 1,
                        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS machines (
                        id BIGINT AUTO_INCREMENT PRIMARY KEY,
                        serial_number VARCHAR(255) NOT NULL,
                        software_version VARCHAR(255) NOT NULL,
                        registered_at DATETIME,
                        status VARCHAR(50)
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS signals (
                        id BIGINT AUTO_INCREMENT PRIMARY KEY,
                        internal_name VARCHAR(255) NOT NULL UNIQUE,
                        display_name VARCHAR(255),
                        unit VARCHAR(50)
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS attribute_equivalences (
                        signal_id BIGINT NOT NULL,
                        numeric_value DOUBLE NOT NULL,
                        display_name VARCHAR(255) NOT NULL,
                        PRIMARY KEY (signal_id, numeric_value)
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS patients (
                        id BIGINT AUTO_INCREMENT PRIMARY KEY,
                        patient_id_str VARCHAR(255) NOT NULL,
                        created_at DATETIME
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS therapies (
                        id BIGINT AUTO_INCREMENT PRIMARY KEY,
                        started_at DATETIME,
                        patient_id BIGINT,
                        machine_id BIGINT,
                        status VARCHAR(50),
                        ended_at DATETIME
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS telemetry (
                        id BIGINT AUTO_INCREMENT PRIMARY KEY,
                        timestamp DATETIME,
                        therapy_id BIGINT,
                        signal_id BIGINT,
                        raw_value BIGINT,
                        physical_value TEXT,
                        unit VARCHAR(50)
                    )",
                ).execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS machine_ips (
                        id BIGINT AUTO_INCREMENT PRIMARY KEY,
                        machine_id BIGINT NOT NULL,
                        ip_address VARCHAR(255) NOT NULL,
                        port INT DEFAULT NULL,
                        label VARCHAR(500),
                        is_active TINYINT(1) DEFAULT 1,
                        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
                        FOREIGN KEY (machine_id) REFERENCES machines(id)
                    )",
                )
                .execute(&pool)
                .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_machine_ips_machine ON machine_ips(machine_id)")
                    .execute(&pool).await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_machine_ips_active ON machine_ips(machine_id, is_active)")
                    .execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS equivalence_deletion_log (
                        id BIGINT AUTO_INCREMENT PRIMARY KEY,
                        signal_id BIGINT NOT NULL,
                        numeric_value DOUBLE NOT NULL,
                        deleted_by VARCHAR(255) NOT NULL,
                        deletion_reason TEXT NOT NULL,
                        deleted_at DATETIME DEFAULT CURRENT_TIMESTAMP
                    )",
                )
                .execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS therapy_comments (
                        id BIGINT AUTO_INCREMENT PRIMARY KEY,
                        therapy_id BIGINT NOT NULL,
                        author_name VARCHAR(255) NOT NULL,
                        comment TEXT NOT NULL,
                        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                        deleted_at DATETIME,
                        deletion_reason TEXT
                    )",
                )
                .execute(&pool).await?;
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS authorization_codes (
                        code VARCHAR(255) PRIMARY KEY,
                        user_id BIGINT NOT NULL,
                        expires_at DATETIME,
                        used TINYINT(1) NOT NULL DEFAULT 0
                    )",
                )
                .execute(&pool).await?;
                Ok(Self::Mysql(pool))
            }
            "mssql" | "sqlsrv" => {
                let mut tconfig = Config::new();
                tconfig.host(&config.db_host);
                tconfig.port(config.db_port);
                tconfig.authentication(AuthMethod::sql_server(&config.db_username, &config.db_password));
                tconfig.database(&config.db_database);
                tconfig.trust_cert();
                let tcp = TcpStream::connect(tconfig.get_addr())
                    .await
                    .map_err(|e| sqlx::Error::Configuration(format!("tiberius TCP connect failed: {}", e).into()))?;
                tcp.set_nodelay(true)
                    .map_err(|e| sqlx::Error::Configuration(format!("tiberius set_nodelay failed: {}", e).into()))?;
                let _client = Client::connect(tconfig.clone(), tcp.compat_write())
                    .await
                    .map_err(|e| sqlx::Error::Configuration(format!("tiberius connect failed: {:?}", e).into()))?;
                drop(_client);
                let mgr = ConnectionManager::new(tconfig);
                let pool = Bb8Pool::builder()
                    .max_size(10)
                    .build(mgr)
                    .await
                    .map_err(|e| sqlx::Error::Configuration(format!("bb8 pool build failed: {}", e).into()))?;
                let mssql = MssqlDb::new(pool);
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'users')
                        CREATE TABLE users (
                            id BIGINT IDENTITY(1,1) PRIMARY KEY,
                            username NVARCHAR(255) NOT NULL,
                            password NVARCHAR(MAX) NOT NULL,
                            full_name NVARCHAR(255) NOT NULL,
                            email NVARCHAR(255) NOT NULL,
                            role NVARCHAR(50) NOT NULL,
                            active BIT NOT NULL DEFAULT 1,
                            created_at DATETIME2 DEFAULT CURRENT_TIMESTAMP
                        )",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'machines')
                        CREATE TABLE machines (
                            id BIGINT IDENTITY(1,1) PRIMARY KEY,
                            serial_number NVARCHAR(255) NOT NULL,
                            software_version NVARCHAR(255) NOT NULL,
                            registered_at DATETIME2,
                            status NVARCHAR(50)
                        )",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'signals')
                        CREATE TABLE signals (
                            id BIGINT IDENTITY(1,1) PRIMARY KEY,
                            internal_name NVARCHAR(255) NOT NULL,
                            display_name NVARCHAR(255),
                            unit NVARCHAR(50)
                        )",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'attribute_equivalences')
                        CREATE TABLE attribute_equivalences (
                            signal_id BIGINT NOT NULL,
                            numeric_value FLOAT NOT NULL,
                            display_name NVARCHAR(255) NOT NULL,
                            PRIMARY KEY (signal_id, numeric_value)
                        )",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'patients')
                        CREATE TABLE patients (
                            id BIGINT IDENTITY(1,1) PRIMARY KEY,
                            patient_id_str NVARCHAR(255) NOT NULL,
                            created_at DATETIME2
                        )",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'therapies')
                        CREATE TABLE therapies (
                            id BIGINT IDENTITY(1,1) PRIMARY KEY,
                            started_at DATETIME2,
                            patient_id BIGINT,
                            machine_id BIGINT,
                            status NVARCHAR(50),
                            ended_at DATETIME2
                        )",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'telemetry')
                        CREATE TABLE telemetry (
                            id BIGINT IDENTITY(1,1) PRIMARY KEY,
                            timestamp DATETIME2,
                            therapy_id BIGINT,
                            signal_id BIGINT,
                            raw_value BIGINT,
                            physical_value NVARCHAR(MAX),
                            unit NVARCHAR(50)
                        )",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'machine_ips')
                        CREATE TABLE machine_ips (
                            id BIGINT IDENTITY(1,1) PRIMARY KEY,
                            machine_id BIGINT NOT NULL REFERENCES machines(id),
                            ip_address NVARCHAR(MAX) NOT NULL,
                            port INT DEFAULT NULL,
                            label NVARCHAR(500),
                            is_active BIT DEFAULT 1,
                            created_at DATETIME2 DEFAULT CURRENT_TIMESTAMP,
                            updated_at DATETIME2 DEFAULT CURRENT_TIMESTAMP
                        )",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM sys.indexes WHERE name = 'idx_machine_ips_machine' AND object_id = OBJECT_ID('machine_ips'))
                        CREATE INDEX idx_machine_ips_machine ON machine_ips(machine_id)",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM sys.indexes WHERE name = 'idx_machine_ips_active' AND object_id = OBJECT_ID('machine_ips'))
                        CREATE INDEX idx_machine_ips_active ON machine_ips(machine_id, is_active)",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'equivalence_deletion_log')
                        CREATE TABLE equivalence_deletion_log (
                            id BIGINT IDENTITY(1,1) PRIMARY KEY,
                            signal_id BIGINT NOT NULL,
                            numeric_value FLOAT NOT NULL,
                            deleted_by NVARCHAR(MAX) NOT NULL,
                            deletion_reason NVARCHAR(MAX) NOT NULL,
                            deleted_at DATETIME2 DEFAULT CURRENT_TIMESTAMP
                        )",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'therapy_comments')
                        CREATE TABLE therapy_comments (
                            id BIGINT IDENTITY(1,1) PRIMARY KEY,
                            therapy_id BIGINT NOT NULL,
                            author_name NVARCHAR(255) NOT NULL,
                            comment NVARCHAR(MAX) NOT NULL,
                            created_at DATETIME2 DEFAULT CURRENT_TIMESTAMP,
                            deleted_at DATETIME2,
                            deletion_reason NVARCHAR(MAX)
                        )",
                ).await?;
                mssql.simple_query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'authorization_codes')
                        CREATE TABLE authorization_codes (
                            code NVARCHAR(255) PRIMARY KEY,
                            user_id BIGINT NOT NULL,
                            expires_at DATETIME2,
                            used BIT NOT NULL DEFAULT 0
                        )",
                ).await?;
                Ok(Self::Mssql(mssql))
            }
            other => Err(sqlx::Error::Configuration(
                format!("Unsupported DB_CONNECTION: {}. Supported: sqlite, postgres, mysql, mssql", other).into(),
            )),
        }
    }

    // --- Users ---
    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        match self {
            Self::NoDb => { Err(sqlx::Error::Configuration("Database not available".into()))},
                Self::Sqlite(p) => sqlx::query_as("SELECT * FROM users WHERE username = ?").bind(username).fetch_optional(p).await,
            Self::Postgres(p) => sqlx::query_as("SELECT * FROM users WHERE username = $1").bind(username).fetch_optional(p).await,
            Self::Mysql(p) => sqlx::query_as("SELECT * FROM users WHERE username = ?").bind(username).fetch_optional(p).await,
            Self::Mssql(db) => db.query_one::<User>("SELECT * FROM users WHERE username = @P1", tp!(username)).await,
        }
    }

    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, sqlx::Error> {
        match self {
            Self::NoDb => { Err(sqlx::Error::Configuration("Database not available".into()))},
                Self::Sqlite(p) => sqlx::query_as("SELECT * FROM users WHERE id = ?").bind(id).fetch_optional(p).await,
            Self::Postgres(p) => sqlx::query_as("SELECT * FROM users WHERE id = $1").bind(id).fetch_optional(p).await,
            Self::Mysql(p) => sqlx::query_as("SELECT * FROM users WHERE id = ?").bind(id).fetch_optional(p).await,
            Self::Mssql(db) => db.query_one::<User>("SELECT * FROM users WHERE id = @P1", tp!(id)).await,
        }
    }

    pub async fn list_users(&self) -> Result<Vec<User>, sqlx::Error> {
        match self {
            Self::NoDb => { Err(sqlx::Error::Configuration("Database not available".into()))},
                Self::Sqlite(p) => sqlx::query_as("SELECT * FROM users ORDER BY id").fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as("SELECT * FROM users ORDER BY id").fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as("SELECT * FROM users ORDER BY id").fetch_all(p).await,
            Self::Mssql(db) => db.query_all::<User>("SELECT * FROM users ORDER BY id", &[]).await,
        }
    }

    pub async fn create_user(&self, req: &CreateUserRequest) -> Result<User, sqlx::Error> {
        let pw = crate::auth::hash_password(&req.password)
            .map_err(|e| sqlx::Error::Configuration(format!("Password hashing failed: {}", e).into()))?;
        match self {
            Self::NoDb => { Err(sqlx::Error::Configuration("Database not available".into()))},
                Self::Sqlite(p) => {
                sqlx::query_as::<_, User>(
                    "INSERT INTO users (username, password, full_name, email, role, active) VALUES (?, ?, ?, ?, ?, 1) RETURNING *",
                ).bind(&req.username).bind(&pw).bind(&req.full_name).bind(&req.email).bind(&req.role)
                .fetch_one(p).await
            }
            Self::Postgres(p) => {
                sqlx::query_as::<_, User>(
                    "INSERT INTO users (username, password, full_name, email, role, active) VALUES ($1, $2, $3, $4, $5, TRUE) RETURNING *",
                ).bind(&req.username).bind(&pw).bind(&req.full_name).bind(&req.email).bind(&req.role)
                .fetch_one(p).await
            }
            Self::Mysql(p) => {
                sqlx::query("INSERT INTO users (username, password, full_name, email, role, active) VALUES (?, ?, ?, ?, ?, TRUE)")
                    .bind(&req.username).bind(&pw).bind(&req.full_name).bind(&req.email).bind(&req.role)
                    .execute(p).await?;
                let id: (i64,) = sqlx::query_as("SELECT LAST_INSERT_ID()").fetch_one(p).await?;
                self.find_user_by_id(id.0).await?.ok_or_else(|| sqlx::Error::Configuration("Created user not found after insert".into()))
            }
            Self::Mssql(db) => db.query_one::<User>(
                "INSERT INTO users (username, password, full_name, email, role, active) OUTPUT INSERTED.* VALUES (@P1, @P2, @P3, @P4, @P5, 1)",
                tp!(req.username, pw, req.full_name, req.email, req.role)
            ).await?.ok_or_else(|| sqlx::Error::Protocol("INSERT OUTPUT returned no rows".into())),
        }
    }

    pub async fn update_user(&self, id: i64, req: &UpdateUserRequest) -> Result<Option<User>, sqlx::Error> {
        let pw = match &req.password {
            Some(v) => Some(crate::auth::hash_password(v)
                .map_err(|e| sqlx::Error::Configuration(format!("Password hashing failed: {}", e).into()))?),
            None => None,
        };
        let has_fields = pw.is_some()
            || req.full_name.is_some()
            || req.email.is_some()
            || req.role.is_some()
            || req.active.is_some();
        if !has_fields { return self.find_user_by_id(id).await; }

        macro_rules! build_update {
            ($p:expr, $ph:expr) => {{
                let mut sets: Vec<&str> = Vec::new();
                if pw.is_some() { sets.push(concat!("password = ", $ph)); }
                if req.full_name.is_some() { sets.push(concat!("full_name = ", $ph)); }
                if req.email.is_some() { sets.push(concat!("email = ", $ph)); }
                if req.role.is_some() { sets.push(concat!("role = ", $ph)); }
                if req.active.is_some() { sets.push(concat!("active = ", $ph)); }
                let sql = format!("UPDATE users SET {} WHERE id = {}", sets.join(", "), $ph);
                let mut q = sqlx::query(AssertSqlSafe(sql));
                if let Some(ref v) = pw { q = q.bind(v.as_str()); }
                if let Some(ref v) = req.full_name { q = q.bind(v.as_str()); }
                if let Some(ref v) = req.email { q = q.bind(v.as_str()); }
                if let Some(ref v) = req.role { q = q.bind(v.as_str()); }
                if let Some(v) = req.active { q = q.bind(if v { 1i32 } else { 0i32 }); }
                q.bind(id).execute($p).await?;
            }};
        }

        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => { build_update!(p, "?"); }
            Self::Postgres(p) => {
                let mut sets: Vec<String> = Vec::new();
                let mut idx = 0u32;
                if pw.is_some() { idx += 1; sets.push(format!("password = ${}", idx)); }
                if req.full_name.is_some() { idx += 1; sets.push(format!("full_name = ${}", idx)); }
                if req.email.is_some() { idx += 1; sets.push(format!("email = ${}", idx)); }
                if req.role.is_some() { idx += 1; sets.push(format!("role = ${}", idx)); }
                if req.active.is_some() { idx += 1; sets.push(format!("active = ${}", idx)); }
                idx += 1;
                let sql = format!("UPDATE users SET {} WHERE id = ${}", sets.join(", "), idx);
                let mut q = sqlx::query(AssertSqlSafe(sql));
                if let Some(ref v) = pw { q = q.bind(v.as_str()); }
                if let Some(ref v) = req.full_name { q = q.bind(v.as_str()); }
                if let Some(ref v) = req.email { q = q.bind(v.as_str()); }
                if let Some(ref v) = req.role { q = q.bind(v.as_str()); }
                if let Some(v) = req.active { q = q.bind(if v { 1i32 } else { 0i32 }); }
                q.bind(id).execute(p).await?;
            }
            Self::Mysql(p) => { build_update!(p, "?"); }
            Self::Mssql(db) => {
                let mut sets: Vec<String> = Vec::new();
                let mut idx = 0u32;
                if pw.is_some() { idx += 1; sets.push(format!("password = @P{}", idx)); }
                if req.full_name.is_some() { idx += 1; sets.push(format!("full_name = @P{}", idx)); }
                if req.email.is_some() { idx += 1; sets.push(format!("email = @P{}", idx)); }
                if req.role.is_some() { idx += 1; sets.push(format!("role = @P{}", idx)); }
                if req.active.is_some() { idx += 1; sets.push(format!("active = @P{}", idx)); }
                idx += 1;
                let sql = format!("UPDATE users SET {} WHERE id = @P{}", sets.join(", "), idx);
                let pw_str: &str = pw.as_deref().unwrap_or("");
                let fn_str: &str = req.full_name.as_deref().unwrap_or("");
                let email_str: &str = req.email.as_deref().unwrap_or("");
                let role_str: &str = req.role.as_deref().unwrap_or("");
                let active_val: Option<i64> = req.active.map(|v| if v { 1i64 } else { 0i64 });
                let mut params: Vec<&dyn tiberius::ToSql> = Vec::new();
                if pw.is_some() { params.push(&pw_str as &dyn tiberius::ToSql); }
                if req.full_name.is_some() { params.push(&fn_str as &dyn tiberius::ToSql); }
                if req.email.is_some() { params.push(&email_str as &dyn tiberius::ToSql); }
                if req.role.is_some() { params.push(&role_str as &dyn tiberius::ToSql); }
                if let Some(ref v) = active_val { params.push(v as &dyn tiberius::ToSql); }
                params.push(&id as &dyn tiberius::ToSql);
                db.execute(&sql, &params).await?;
            }
        }
        self.find_user_by_id(id).await
    }

    pub async fn delete_user(&self, id: i64) -> Result<bool, sqlx::Error> {
        let affected = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => {
                let r = sqlx::query("DELETE FROM users WHERE id = ?").bind(id).execute(p).await?;
                r.rows_affected()
            }
            Self::Postgres(p) => {
                let r = sqlx::query("DELETE FROM users WHERE id = $1").bind(id).execute(p).await?;
                r.rows_affected()
            }
            Self::Mysql(p) => {
                let r = sqlx::query("DELETE FROM users WHERE id = ?").bind(id).execute(p).await?;
                r.rows_affected()
            }
            Self::Mssql(db) => db.execute("DELETE FROM users WHERE id = @P1", tp!(id)).await?,
        };
        Ok(affected > 0)
    }

    pub async fn count_users(&self) -> Result<i64, sqlx::Error> {
        match self {
            Self::NoDb => { Err(sqlx::Error::Configuration("Database not available".into()))},
                Self::Sqlite(p) => sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(p).await,
            Self::Postgres(p) => sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(p).await,
            Self::Mysql(p) => sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(p).await,
            Self::Mssql(db) => db.query_scalar::<i32>("SELECT COUNT(*) FROM users", &[]).await.map(|v| v as i64),
        }
    }

    // --- Patients ---
    pub async fn list_patients(&self, page: i64, per_page: i64, search: Option<&str>) -> Result<PaginatedResponse<Patient>, sqlx::Error> {
        let offset = (page - 1).max(0) * per_page;
        let count_total = if let Some(s) = search {
            match self {
                Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM patients WHERE patient_id_str LIKE ?").bind(format!("%{}%", s)).fetch_one(p).await?,
                Self::Postgres(p) => sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM patients WHERE patient_id_str ILIKE $1").bind(format!("%{}%", s)).fetch_one(p).await?,
                Self::Mysql(p) => sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM patients WHERE patient_id_str LIKE ?").bind(format!("%{}%", s)).fetch_one(p).await?,
                Self::Mssql(db) => db.query_scalar::<i32>("SELECT COUNT(*) FROM patients WHERE patient_id_str LIKE @P1", tp!(format!("%{}%", s))).await.map(|v| v as i64)?,
            }
        } else {
            match self {
                Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => sqlx::query_scalar("SELECT COUNT(*) FROM patients").fetch_one(p).await?,
                Self::Postgres(p) => sqlx::query_scalar("SELECT COUNT(*) FROM patients").fetch_one(p).await?,
                Self::Mysql(p) => sqlx::query_scalar("SELECT COUNT(*) FROM patients").fetch_one(p).await?,
                Self::Mssql(db) => db.query_scalar::<i32>("SELECT COUNT(*) FROM patients", &[]).await.map(|v| v as i64)?,
            }
        };

        let data = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => {
                if let Some(s) = search {
                    sqlx::query_as::<_, Patient>("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.patient_id_str LIKE ? ORDER BY active_therapy_count DESC, p.id DESC LIMIT ? OFFSET ?")
                        .bind(format!("%{}%", s)).bind(per_page).bind(offset).fetch_all(p).await?
                } else {
                    sqlx::query_as::<_, Patient>("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p ORDER BY active_therapy_count DESC, p.id DESC LIMIT ? OFFSET ?")
                        .bind(per_page).bind(offset).fetch_all(p).await?
                }
            }
            Self::Postgres(p) => {
                if let Some(s) = search {
                    sqlx::query_as::<_, Patient>("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.patient_id_str ILIKE $1 ORDER BY active_therapy_count DESC, p.id DESC LIMIT $2 OFFSET $3")
                        .bind(format!("%{}%", s)).bind(per_page).bind(offset).fetch_all(p).await?
                } else {
                    sqlx::query_as::<_, Patient>("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p ORDER BY active_therapy_count DESC, p.id DESC LIMIT $1 OFFSET $2")
                        .bind(per_page).bind(offset).fetch_all(p).await?
                }
            }
            Self::Mysql(p) => {
                if let Some(s) = search {
                    sqlx::query_as::<_, Patient>("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.patient_id_str LIKE ? ORDER BY active_therapy_count DESC, p.id DESC LIMIT ? OFFSET ?")
                        .bind(format!("%{}%", s)).bind(per_page).bind(offset).fetch_all(p).await?
                } else {
                    sqlx::query_as::<_, Patient>("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p ORDER BY active_therapy_count DESC, p.id DESC LIMIT ? OFFSET ?")
                        .bind(per_page).bind(offset).fetch_all(p).await?
                }
            }
            Self::Mssql(db) => {
                let sql = "SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p";
                if let Some(s) = search {
                    let like = format!("%{}%", s);
                    let full_sql = format!("{} WHERE p.patient_id_str LIKE @P1 ORDER BY active_therapy_count DESC, p.id DESC OFFSET @P2 ROWS FETCH NEXT @P3 ROWS ONLY", sql);
                    db.query_all::<Patient>(&full_sql, tp!(like, offset, per_page)).await?
                } else {
                    let full_sql = format!("{} ORDER BY active_therapy_count DESC, p.id DESC OFFSET @P1 ROWS FETCH NEXT @P2 ROWS ONLY", sql);
                    db.query_all::<Patient>(&full_sql, tp!(offset, per_page)).await?
                }
            }
        };
        Ok(PaginatedResponse { total: count_total, page, per_page, total_pages: (count_total as f64 / per_page as f64).ceil() as i64, data })
    }

    pub async fn find_patient_by_id(&self, id: i64) -> Result<Option<Patient>, sqlx::Error> {
        match self {
            Self::NoDb => { Err(sqlx::Error::Configuration("Database not available".into()))},
            Self::Sqlite(p) => sqlx::query_as("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.id = ?").bind(id).fetch_optional(p).await,
            Self::Postgres(p) => sqlx::query_as("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.id = $1").bind(id).fetch_optional(p).await,
            Self::Mysql(p) => sqlx::query_as("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.id = ?").bind(id).fetch_optional(p).await,
            Self::Mssql(db) => db.query_one::<Patient>("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.id = @P1", tp!(id)).await,
        }
    }

    // --- Therapies ---
    pub async fn list_therapies_by_patient_paginated(&self, patient_id: i64, page: i64, per_page: i64) -> Result<PaginatedResponse<TherapyWithMachine>, sqlx::Error> {
        let offset = (page - 1).max(0) * per_page;
        let count_total: i64 = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => sqlx::query_scalar("SELECT COUNT(*) FROM therapies WHERE patient_id = ?").bind(patient_id).fetch_one(p).await?,
            Self::Postgres(p) => sqlx::query_scalar("SELECT COUNT(*) FROM therapies WHERE patient_id = $1").bind(patient_id).fetch_one(p).await?,
            Self::Mysql(p) => sqlx::query_scalar("SELECT COUNT(*) FROM therapies WHERE patient_id = ?").bind(patient_id).fetch_one(p).await?,
            Self::Mssql(db) => db.query_scalar::<i32>("SELECT COUNT(*) FROM therapies WHERE patient_id = @P1", tp!(patient_id)).await.map(|v| v as i64)?,
        };
        let raw: Vec<TherapyRaw> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final , p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.patient_id = ? ORDER BY t.started_at DESC LIMIT ? OFFSET ?")
                    .bind(patient_id).bind(per_page).bind(offset).fetch_all(p).await?
            }
            Self::Postgres(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final , p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.patient_id = $1 ORDER BY t.started_at DESC LIMIT $2 OFFSET $3")
                    .bind(patient_id).bind(per_page).bind(offset).fetch_all(p).await?
            }
            Self::Mysql(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final , p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.patient_id = ? ORDER BY t.started_at DESC LIMIT ? OFFSET ?")
                    .bind(patient_id).bind(per_page).bind(offset).fetch_all(p).await?
            }
            Self::Mssql(db) => db.query_all::<TherapyRaw>(
                "SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT TOP 1 mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id) as ip_address, (SELECT TOP 1 mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id) as port, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC) as therapy_type_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC) as therapy_type, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC) as kit_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC) as kit, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC) as weight_initial_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC) as weight_initial, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC) as weight_final_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC) as weight_final , p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.patient_id = @P1 ORDER BY t.started_at DESC OFFSET @P2 ROWS FETCH NEXT @P3 ROWS ONLY",
                tp!(patient_id, offset, per_page)
            ).await?,
        };
        let equivalences = self.load_equivalences().await.unwrap_or_default();
        let therapies: Vec<TherapyWithMachine> = raw.into_iter().map(|r| {
            let mut t = TherapyWithMachine::from(r.clone());
            if let (Some(sig_id), Some(ref val)) = (r.therapy_type_signal_id, t.therapy_type.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, &val, &equivalences) {
                    t.therapy_type = Some(display.to_string());
                }
            }
            if let (Some(sig_id), Some(ref val)) = (r.kit_signal_id, t.kit.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, &val, &equivalences) {
                    t.kit = Some(display.to_string());
                }
            }
            if let (Some(sig_id), Some(ref val)) = (r.weight_initial_signal_id, t.weight_initial.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, val, &equivalences) {
                    t.weight_initial = Some(display.to_string());
                }
            }
            if let (Some(sig_id), Some(ref val)) = (r.weight_final_signal_id, t.weight_final.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, val, &equivalences) {
                    t.weight_final = Some(display.to_string());
                }
            }
            t
        }).collect();
        Ok(PaginatedResponse { total: count_total, page, per_page, total_pages: (count_total as f64 / per_page as f64).ceil() as i64, data: therapies })
    }

    pub async fn list_therapies_by_patient(&self, patient_id: i64) -> Result<Vec<TherapyWithMachine>, sqlx::Error> {
        let raw: Vec<TherapyRaw> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final , p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.patient_id = ? ORDER BY t.started_at DESC")
                    .bind(patient_id).fetch_all(p).await?
            }
            Self::Postgres(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final , p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.patient_id = $1 ORDER BY t.started_at DESC")
                    .bind(patient_id).fetch_all(p).await?
            }
            Self::Mysql(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final , p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.patient_id = ? ORDER BY t.started_at DESC")
                    .bind(patient_id).fetch_all(p).await?
            }
            Self::Mssql(db) => db.query_all::<TherapyRaw>(
                "SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT TOP 1 mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id) as ip_address, (SELECT TOP 1 mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id) as port, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC) as therapy_type_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC) as therapy_type, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC) as kit_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC) as kit, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC) as weight_initial_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC) as weight_initial, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC) as weight_final_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC) as weight_final , p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.patient_id = @P1 ORDER BY t.started_at DESC",
                tp!(patient_id)
            ).await?,
        };
        let equivalences = self.load_equivalences().await.unwrap_or_default();
        let therapies: Vec<TherapyWithMachine> = raw.into_iter().map(|r| {
            let mut t = TherapyWithMachine::from(r.clone());
            if let (Some(sig_id), Some(ref val)) = (r.therapy_type_signal_id, t.therapy_type.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, &val, &equivalences) {
                    t.therapy_type = Some(display.to_string());
                }
            }
            if let (Some(sig_id), Some(ref val)) = (r.kit_signal_id, t.kit.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, &val, &equivalences) {
                    t.kit = Some(display.to_string());
                }
            }
            if let (Some(sig_id), Some(ref val)) = (r.weight_initial_signal_id, t.weight_initial.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, val, &equivalences) {
                    t.weight_initial = Some(display.to_string());
                }
            }
            if let (Some(sig_id), Some(ref val)) = (r.weight_final_signal_id, t.weight_final.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, val, &equivalences) {
                    t.weight_final = Some(display.to_string());
                }
            }
            t
        }).collect();
        Ok(therapies)
    }

    pub async fn find_therapy_by_id(&self, therapy_id: i64) -> Result<Option<TherapyWithMachine>, sqlx::Error> {
        let raw: Option<TherapyRaw> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final, p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.id = ?")
                    .bind(therapy_id).fetch_optional(p).await?
            }
            Self::Postgres(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final, p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.id = $1")
                    .bind(therapy_id).fetch_optional(p).await?
            }
            Self::Mysql(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC LIMIT 1) as therapy_type, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC LIMIT 1) as kit, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final, p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.id = ?")
                    .bind(therapy_id).fetch_optional(p).await?
            }
            Self::Mssql(db) => db.query_one::<TherapyRaw>(
                "SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT TOP 1 mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id) as ip_address, (SELECT TOP 1 mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id) as port, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC) as therapy_type_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_therapy_mode_set' ORDER BY te.timestamp DESC) as therapy_type, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC) as kit_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'd_kit_type_str' ORDER BY te.timestamp DESC) as kit, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC) as weight_initial_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC) as weight_initial, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC) as weight_final_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC) as weight_final, p.patient_id_str FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id LEFT JOIN patients p ON t.patient_id = p.id WHERE t.id = @P1",
                tp!(therapy_id)
            ).await?,
        };
        let equivalences = self.load_equivalences().await.unwrap_or_default();
        Ok(raw.map(|r| {
            let mut t = TherapyWithMachine::from(r.clone());
            if let (Some(sig_id), Some(ref val)) = (r.therapy_type_signal_id, t.therapy_type.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, &val, &equivalences) {
                    t.therapy_type = Some(display.to_string());
                }
            }
            if let (Some(sig_id), Some(ref val)) = (r.kit_signal_id, t.kit.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, &val, &equivalences) {
                    t.kit = Some(display.to_string());
                }
            }
            if let (Some(sig_id), Some(ref val)) = (r.weight_initial_signal_id, t.weight_initial.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, val, &equivalences) {
                    t.weight_initial = Some(display.to_string());
                }
            }
            if let (Some(sig_id), Some(ref val)) = (r.weight_final_signal_id, t.weight_final.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, val, &equivalences) {
                    t.weight_final = Some(display.to_string());
                }
            }
            t
        }))
    }

    pub async fn list_active_therapies(&self) -> Result<Vec<ActiveTherapy>, sqlx::Error> {
        let raw: Vec<ActiveTherapyRaw> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => {
                sqlx::query_as("SELECT t.id as therapy_id, t.patient_id, p.patient_id_str, t.started_at, m.serial_number, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id AND mi.is_active = 1 LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id AND mi.is_active = 1 LIMIT 1) as port, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_press_ap_act' ORDER BY te.timestamp DESC LIMIT 1) as arterial_pressure, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_press_vp_act' ORDER BY te.timestamp DESC LIMIT 1) as venous_pressure, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_pump_bs_bl_flow_act' ORDER BY te.timestamp DESC LIMIT 1) as blood_flow, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final FROM therapies t JOIN patients p ON t.patient_id = p.id LEFT JOIN machines m ON t.machine_id = m.id WHERE t.status = 'active' ORDER BY t.started_at DESC")
                    .fetch_all(p).await?
            }
            Self::Postgres(p) => {
                sqlx::query_as("SELECT t.id as therapy_id, t.patient_id, p.patient_id_str, t.started_at, m.serial_number, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id AND mi.is_active = true LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id AND mi.is_active = true LIMIT 1) as port, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_press_ap_act' ORDER BY te.timestamp DESC LIMIT 1) as arterial_pressure, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_press_vp_act' ORDER BY te.timestamp DESC LIMIT 1) as venous_pressure, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_pump_bs_bl_flow_act' ORDER BY te.timestamp DESC LIMIT 1) as blood_flow, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final FROM therapies t JOIN patients p ON t.patient_id = p.id LEFT JOIN machines m ON t.machine_id = m.id WHERE t.status = 'active' ORDER BY t.started_at DESC")
                    .fetch_all(p).await?
            }
            Self::Mysql(p) => {
                sqlx::query_as("SELECT t.id as therapy_id, t.patient_id, p.patient_id_str, t.started_at, m.serial_number, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id AND mi.is_active = 1 LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id AND mi.is_active = 1 LIMIT 1) as port, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_press_ap_act' ORDER BY te.timestamp DESC LIMIT 1) as arterial_pressure, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_press_vp_act' ORDER BY te.timestamp DESC LIMIT 1) as venous_pressure, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_pump_bs_bl_flow_act' ORDER BY te.timestamp DESC LIMIT 1) as blood_flow, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC LIMIT 1) as weight_initial, (SELECT s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final_signal_id, (SELECT te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC LIMIT 1) as weight_final FROM therapies t JOIN patients p ON t.patient_id = p.id LEFT JOIN machines m ON t.machine_id = m.id WHERE t.status = 'active' ORDER BY t.started_at DESC")
                    .fetch_all(p).await?
            }
            Self::Mssql(db) => db.query_all::<ActiveTherapyRaw>(
                "SELECT t.id as therapy_id, t.patient_id, p.patient_id_str, t.started_at, m.serial_number, (SELECT TOP 1 mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id AND mi.is_active = 1) as ip_address, (SELECT TOP 1 mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id AND mi.is_active = 1) as port, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_press_ap_act' ORDER BY te.timestamp DESC) as arterial_pressure, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_press_vp_act' ORDER BY te.timestamp DESC) as venous_pressure, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'c_pump_bs_bl_flow_act' ORDER BY te.timestamp DESC) as blood_flow, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC) as weight_initial_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp ASC) as weight_initial, (SELECT TOP 1 s.id FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC) as weight_final_signal_id, (SELECT TOP 1 te.physical_value FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = t.id AND s.internal_name = 'g_patient_data_weight_set' ORDER BY te.timestamp DESC) as weight_final FROM therapies t JOIN patients p ON t.patient_id = p.id LEFT JOIN machines m ON t.machine_id = m.id WHERE t.status = 'active' ORDER BY t.started_at DESC",
                tp!()
            ).await?,
        };
        let equivalences = self.load_equivalences().await.unwrap_or_default();
        let therapy_ids: Vec<i64> = raw.iter().map(|r| r.therapy_id).collect();
        let mut comments_by_therapy: std::collections::HashMap<i64, Vec<TherapyComment>> = std::collections::HashMap::new();
        if !therapy_ids.is_empty() {
            let all_comments = self.list_therapy_comments_bulk(&therapy_ids).await.unwrap_or_default();
            for c in all_comments {
                comments_by_therapy.entry(c.therapy_id).or_default().push(c);
            }
        }
        let therapies: Vec<ActiveTherapy> = raw.into_iter().map(|r| {
            let mut weight_initial = r.weight_initial.clone();
            let mut weight_final = r.weight_final.clone();
            if let (Some(sig_id), Some(ref val)) = (r.weight_initial_signal_id, weight_initial.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, &val, &equivalences) {
                    weight_initial = Some(display.to_string());
                }
            }
            if let (Some(sig_id), Some(ref val)) = (r.weight_final_signal_id, weight_final.clone()) {
                if let Some(display) = lookup_equivalence(sig_id, &val, &equivalences) {
                    weight_final = Some(display.to_string());
                }
            }
            ActiveTherapy {
                therapy_id: r.therapy_id,
                patient_id: r.patient_id,
                patient_id_str: r.patient_id_str,
                started_at: r.started_at,
                serial_number: r.serial_number,
                ip_address: r.ip_address,
                port: r.port,
                arterial_pressure: r.arterial_pressure,
                venous_pressure: r.venous_pressure,
                blood_flow: r.blood_flow,
                weight_initial,
                weight_final,
                comments: comments_by_therapy.remove(&r.therapy_id).unwrap_or_default(),
            }
        }).collect();
        Ok(therapies)
    }

    pub async fn list_therapy_comments(&self, therapy_id: i64) -> Result<Vec<TherapyComment>, sqlx::Error> {
        let sql = "SELECT id, therapy_id, author_name, comment, created_at, deleted_at, deletion_reason FROM therapy_comments WHERE therapy_id = ? AND deleted_at IS NULL ORDER BY created_at ASC";
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => sqlx::query_as(sql).bind(therapy_id).fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as(AssertSqlSafe(sql.replace('?', "$1"))).bind(therapy_id).fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as(sql).bind(therapy_id).fetch_all(p).await,
            Self::Mssql(db) => db.query_all::<TherapyComment>(&sql.replace('?', "@P1"), tp!(therapy_id)).await,
        }
    }

    async fn list_therapy_comments_bulk(&self, therapy_ids: &[i64]) -> Result<Vec<TherapyComment>, sqlx::Error> {
        if therapy_ids.is_empty() {
            return Ok(vec![]);
        }
        let placeholders: Vec<String> = therapy_ids.iter().enumerate().map(|(i, _)| {
            if matches!(self, Self::Postgres(_)) {
                format!("${}", i + 1)
            } else if matches!(self, Self::Mssql(_)) {
                // For MSSQL we fetch individually below
                String::new()
            } else {
                "?".to_string()
            }
        }).collect();
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => {
                let sql = format!("SELECT id, therapy_id, author_name, comment, created_at, deleted_at, deletion_reason FROM therapy_comments WHERE therapy_id IN ({}) AND deleted_at IS NULL ORDER BY created_at ASC", placeholders.join(","));
                let mut q = sqlx::query_as::<_, TherapyComment>(AssertSqlSafe(sql));
                for &id in therapy_ids { q = q.bind(id); }
                q.fetch_all(p).await
            }
            Self::Postgres(p) => {
                let sql = format!("SELECT id, therapy_id, author_name, comment, created_at, deleted_at, deletion_reason FROM therapy_comments WHERE therapy_id IN ({}) AND deleted_at IS NULL ORDER BY created_at ASC", placeholders.join(","));
                let mut q = sqlx::query_as::<_, TherapyComment>(AssertSqlSafe(sql));
                for &id in therapy_ids { q = q.bind(id); }
                q.fetch_all(p).await
            }
            Self::Mysql(p) => {
                let sql = format!("SELECT id, therapy_id, author_name, comment, created_at, deleted_at, deletion_reason FROM therapy_comments WHERE therapy_id IN ({}) AND deleted_at IS NULL ORDER BY created_at ASC", placeholders.join(","));
                let mut q = sqlx::query_as::<_, TherapyComment>(AssertSqlSafe(sql));
                for &id in therapy_ids { q = q.bind(id); }
                q.fetch_all(p).await
            }
            Self::Mssql(_db) => {
                let mut all = Vec::new();
                for &tid in therapy_ids {
                    if let Ok(mut comments) = self.list_therapy_comments(tid).await {
                        all.append(&mut comments);
                    }
                }
                Ok(all)
            }
        }
    }

    pub async fn create_therapy_comment(&self, therapy_id: i64, author_name: &str, comment: &str) -> Result<TherapyComment, sqlx::Error> {
        let insert_sql = match self {
            Self::Mssql(_) => "INSERT INTO therapy_comments (therapy_id, author_name, comment) OUTPUT INSERTED.* VALUES (@P1, @P2, @P3)".to_string(),
            _ => "INSERT INTO therapy_comments (therapy_id, author_name, comment) VALUES (?, ?, ?)".to_string(),
        };
        let last_id = match self {
            Self::NoDb => return Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => {
                sqlx::query(AssertSqlSafe(insert_sql)).bind(therapy_id).bind(author_name).bind(comment).execute(p).await?.last_insert_rowid()
            }
            Self::Postgres(p) => {
                sqlx::query_scalar::<_, i64>("INSERT INTO therapy_comments (therapy_id, author_name, comment) VALUES ($1, $2, $3) RETURNING id")
                    .bind(therapy_id).bind(author_name).bind(comment).fetch_one(p).await?
            }
            Self::Mysql(p) => {
                sqlx::query(AssertSqlSafe(insert_sql)).bind(therapy_id).bind(author_name).bind(comment).execute(p).await?.last_insert_id() as i64
            }
            Self::Mssql(db) => {
                return db.query_one::<TherapyComment>(&insert_sql, tp!(therapy_id, author_name, comment)).await?.ok_or_else(|| sqlx::Error::Protocol("No comment returned".into()));
            }
        };
        self.find_therapy_comment_by_id(last_id).await
    }

    async fn find_therapy_comment_by_id(&self, id: i64) -> Result<TherapyComment, sqlx::Error> {
        let sql = "SELECT id, therapy_id, author_name, comment, created_at, deleted_at, deletion_reason FROM therapy_comments WHERE id = ?";
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => sqlx::query_as(sql).bind(id).fetch_one(p).await,
            Self::Postgres(p) => sqlx::query_as(AssertSqlSafe(sql.replace('?', "$1"))).bind(id).fetch_one(p).await,
            Self::Mysql(p) => sqlx::query_as(sql).bind(id).fetch_one(p).await,
            Self::Mssql(db) => db.query_one::<TherapyComment>(&sql.replace('?', "@P1"), tp!(id)).await?.ok_or_else(|| sqlx::Error::Protocol("Comment not found".into())),
        }
    }

    pub async fn delete_therapy_comment(&self, comment_id: i64, deletion_reason: &str) -> Result<(), sqlx::Error> {
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => sqlx::query("UPDATE therapy_comments SET deleted_at = CURRENT_TIMESTAMP, deletion_reason = ? WHERE id = ?")
                .bind(deletion_reason).bind(comment_id).execute(p).await.map(|_| ()),
            Self::Postgres(p) => sqlx::query("UPDATE therapy_comments SET deleted_at = CURRENT_TIMESTAMP, deletion_reason = $1 WHERE id = $2")
                .bind(deletion_reason).bind(comment_id).execute(p).await.map(|_| ()),
            Self::Mysql(p) => sqlx::query("UPDATE therapy_comments SET deleted_at = CURRENT_TIMESTAMP, deletion_reason = ? WHERE id = ?")
                .bind(deletion_reason).bind(comment_id).execute(p).await.map(|_| ()),
            Self::Mssql(db) => db.execute("UPDATE therapy_comments SET deleted_at = CURRENT_TIMESTAMP, deletion_reason = @P1 WHERE id = @P2", tp!(deletion_reason, comment_id)).await.map(|_| ()),
        }
    }

    // --- Telemetry ---
    pub async fn list_telemetry(&self, patient_id: i64, page: i64, per_page: i64, signal_ids: Option<&[i64]>, date_from: Option<&str>, date_to: Option<&str>) -> Result<PaginatedResponse<TelemetryReading>, sqlx::Error> {
        let offset = (page - 1).max(0) * per_page;

        let mut extra_where = String::new();
        let sig_count = signal_ids.map(|ids| ids.len()).unwrap_or(0);
        if sig_count > 0 {
            let ph: Vec<&str> = vec!["?"; sig_count];
            extra_where = format!(" AND te.signal_id IN ({})", ph.join(", "));
        }
        if date_from.is_some() { extra_where.push_str(" AND te.timestamp >= ?"); }
        if date_to.is_some() { extra_where.push_str(" AND te.timestamp <= ?"); }

        let where_ext = &extra_where;

        macro_rules! bind_extras {
            ($q:expr, $ids:expr, $from:expr, $to:expr) => {{
                let mut q = $q;
                if let Some(ids) = $ids { for id in ids { q = q.bind(id); } }
                if let Some(from) = $from { q = q.bind(from); }
                if let Some(to) = $to { q = q.bind(to); }
                q
            }};
        }

        let total: i64 = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => {
                let sql = format!("SELECT COUNT(*) FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = ?{}", where_ext);
                let mut q = sqlx::query_scalar::<_, i64>(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras!(q, signal_ids, date_from, date_to);
                q.fetch_one(p).await?
            }
            Self::Postgres(p) => {
                let mut pg_where = where_ext.clone();
                let mut ph_idx = 2u32;
                while pg_where.contains('?') {
                    pg_where = pg_where.replacen('?', &format!("${}", ph_idx), 1);
                    ph_idx += 1;
                }
                let sql = format!("SELECT COUNT(*) FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = $1{}", pg_where);
                let mut q = sqlx::query_scalar::<_, i64>(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras!(q, signal_ids, date_from, date_to);
                q.fetch_one(p).await?
            }
            Self::Mysql(p) => {
                let mut q = sqlx::query_scalar::<_, i64>(
                    AssertSqlSafe(format!("SELECT COUNT(*) FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = ?{}", where_ext))
                ).bind(patient_id);
                q = bind_extras!(q, signal_ids, date_from, date_to);
                q.fetch_one(p).await?
            }
            Self::Mssql(db) => {
                let mut ms_where = where_ext.clone();
                let mut ph_idx = 2u32;
                while ms_where.contains('?') {
                    ms_where = ms_where.replacen('?', &format!("@P{}", ph_idx), 1);
                    ph_idx += 1;
                }
                let sql = format!("SELECT COUNT(*) FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = @P1{}", ms_where);
                let signal_ids_local: &[i64] = signal_ids.unwrap_or(&[]);
                let date_from_str: &str = date_from.unwrap_or("");
                let date_to_str: &str = date_to.unwrap_or("");
                let mut params: Vec<&dyn tiberius::ToSql> = vec![&patient_id];
                for id in signal_ids_local { params.push(id as &dyn tiberius::ToSql); }
                if date_from.is_some() { params.push(&date_from_str as &dyn tiberius::ToSql); }
                if date_to.is_some() { params.push(&date_to_str as &dyn tiberius::ToSql); }
                db.query_scalar::<i32>(&sql, &params).await.map(|v| v as i64)?
            }
        };

        let data: Vec<TelemetryReading> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => {
                let sql = format!("SELECT te.*, COALESCE(s.display_name, s.internal_name) AS signal_name FROM telemetry te JOIN therapies t ON te.therapy_id = t.id LEFT JOIN signals s ON te.signal_id = s.id WHERE t.patient_id = ?{} ORDER BY te.timestamp DESC LIMIT ? OFFSET ?", where_ext);
                let mut q = sqlx::query_as::<_, TelemetryReading>(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras!(q, signal_ids, date_from, date_to);
                q.bind(per_page).bind(offset).fetch_all(p).await?
            }
            Self::Postgres(p) => {
                let mut pg_where = where_ext.clone();
                let mut ph_idx = 2u32;
                while pg_where.contains('?') {
                    pg_where = pg_where.replacen('?', &format!("${}", ph_idx), 1);
                    ph_idx += 1;
                }
                let limit_ph = ph_idx;
                let offset_ph = ph_idx + 1;
                let sql = format!("SELECT te.*, COALESCE(s.display_name, s.internal_name) AS signal_name FROM telemetry te JOIN therapies t ON te.therapy_id = t.id LEFT JOIN signals s ON te.signal_id = s.id WHERE t.patient_id = $1{} ORDER BY te.timestamp DESC LIMIT ${} OFFSET ${}", pg_where, limit_ph, offset_ph);
                let mut q = sqlx::query_as::<_, TelemetryReading>(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras!(q, signal_ids, date_from, date_to);
                q.bind(per_page).bind(offset).fetch_all(p).await?
            }
            Self::Mysql(p) => {
                let sql = format!("SELECT te.*, COALESCE(s.display_name, s.internal_name) AS signal_name FROM telemetry te JOIN therapies t ON te.therapy_id = t.id LEFT JOIN signals s ON te.signal_id = s.id WHERE t.patient_id = ?{} ORDER BY te.timestamp DESC LIMIT ? OFFSET ?", where_ext);
                let mut q = sqlx::query_as::<_, TelemetryReading>(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras!(q, signal_ids, date_from, date_to);
                q.bind(per_page).bind(offset).fetch_all(p).await?
            }
            Self::Mssql(db) => {
                let mut ms_where = where_ext.clone();
                let mut ph_idx = 2u32;
                while ms_where.contains('?') {
                    ms_where = ms_where.replacen('?', &format!("@P{}", ph_idx), 1);
                    ph_idx += 1;
                }
                let offset_ph = ph_idx;
                let limit_ph = ph_idx + 1;
                let sql = format!("SELECT te.*, COALESCE(s.display_name, s.internal_name) AS signal_name FROM telemetry te JOIN therapies t ON te.therapy_id = t.id LEFT JOIN signals s ON te.signal_id = s.id WHERE t.patient_id = @P1{} ORDER BY te.timestamp DESC OFFSET @P{} ROWS FETCH NEXT @P{} ROWS ONLY", ms_where, offset_ph, limit_ph);
                let signal_ids_local: &[i64] = signal_ids.unwrap_or(&[]);
                let date_from_str: &str = date_from.unwrap_or("");
                let date_to_str: &str = date_to.unwrap_or("");
                let mut params: Vec<&dyn tiberius::ToSql> = vec![&patient_id as &dyn tiberius::ToSql];
                for id in signal_ids_local { params.push(id as &dyn tiberius::ToSql); }
                if date_from.is_some() { params.push(&date_from_str as &dyn tiberius::ToSql); }
                if date_to.is_some() { params.push(&date_to_str as &dyn tiberius::ToSql); }
                params.push(&offset as &dyn tiberius::ToSql);
                params.push(&per_page as &dyn tiberius::ToSql);
                db.query_all::<TelemetryReading>(&sql, &params).await?
            }
        };

        Ok(PaginatedResponse { total, page, per_page, total_pages: (total as f64 / per_page as f64).ceil() as i64, data })
    }

    // --- Active Device ---
    pub async fn find_active_device(&self, patient_id: i64) -> Result<Option<ActiveDevice>, sqlx::Error> {
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => {
                sqlx::query_as::<_, ActiveDeviceRaw>(
                    "SELECT mi.ip_address, mi.port, m.serial_number FROM therapies t JOIN machines m ON t.machine_id = m.id JOIN machine_ips mi ON mi.machine_id = m.id AND mi.is_active = 1 WHERE t.patient_id = ? AND t.status = 'active' ORDER BY t.started_at DESC LIMIT 1"
                ).bind(patient_id).fetch_optional(p).await
            }
            Self::Postgres(p) => {
                sqlx::query_as::<_, ActiveDeviceRaw>(
                    "SELECT mi.ip_address, mi.port, m.serial_number FROM therapies t JOIN machines m ON t.machine_id = m.id JOIN machine_ips mi ON mi.machine_id = m.id AND mi.is_active = TRUE WHERE t.patient_id = $1 AND t.status = 'active' ORDER BY t.started_at DESC LIMIT 1"
                ).bind(patient_id).fetch_optional(p).await
            }
            Self::Mysql(p) => {
                sqlx::query_as::<_, ActiveDeviceRaw>(
                    "SELECT mi.ip_address, mi.port, m.serial_number FROM therapies t JOIN machines m ON t.machine_id = m.id JOIN machine_ips mi ON mi.machine_id = m.id AND mi.is_active = TRUE WHERE t.patient_id = ? AND t.status = 'active' ORDER BY t.started_at DESC LIMIT 1"
                ).bind(patient_id).fetch_optional(p).await
            }
            Self::Mssql(db) => db.query_one::<ActiveDeviceRaw>(
                "SELECT mi.ip_address, mi.port, m.serial_number FROM therapies t JOIN machines m ON t.machine_id = m.id JOIN machine_ips mi ON mi.machine_id = m.id AND mi.is_active = 1 WHERE t.patient_id = @P1 AND t.status = 'active' ORDER BY t.started_at DESC OFFSET 0 ROWS FETCH NEXT 1 ROWS ONLY",
                tp!(patient_id)
            ).await
        }.map(|o| o.map(ActiveDevice::from))
    }

    // --- Machine IPs ---
    pub async fn list_machine_ips(&self) -> Result<Vec<MachineIpWithSerial>, sqlx::Error> {
        let sql = "SELECT mi.*, m.serial_number FROM machine_ips mi LEFT JOIN machines m ON mi.machine_id = m.id ORDER BY mi.id DESC";
        let raw: Vec<MachineIpWithSerialRaw> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => sqlx::query_as(sql).fetch_all(p).await?,
            Self::Postgres(p) => sqlx::query_as(sql).fetch_all(p).await?,
            Self::Mysql(p) => sqlx::query_as(sql).fetch_all(p).await?,
            Self::Mssql(db) => db.query_all::<MachineIpWithSerialRaw>(sql, &[]).await?,
        };
        Ok(raw.into_iter().map(MachineIpWithSerial::from).collect())
    }

    pub async fn create_machine_ip(&self, req: &CreateMachineIpRequest) -> Result<MachineIp, sqlx::Error> {
        match self {
            Self::NoDb => { Err(sqlx::Error::Configuration("Database not available".into()))},
                Self::Sqlite(p) => {
                sqlx::query_as::<_, MachineIp>(
                    "INSERT INTO machine_ips (machine_id, ip_address, port, label) VALUES (?, ?, ?, ?) RETURNING *"
                ).bind(req.machine_id).bind(&req.ip_address).bind(req.port).bind(&req.label)
                .fetch_one(p).await
            }
            Self::Postgres(p) => {
                sqlx::query_as::<_, MachineIp>(
                    "INSERT INTO machine_ips (machine_id, ip_address, port, label) VALUES ($1, $2, $3, $4) RETURNING *"
                ).bind(req.machine_id).bind(&req.ip_address).bind(req.port).bind(&req.label)
                .fetch_one(p).await
            }
            Self::Mysql(p) => {
                sqlx::query("INSERT INTO machine_ips (machine_id, ip_address, port, label) VALUES (?, ?, ?, ?)")
                    .bind(req.machine_id).bind(&req.ip_address).bind(req.port).bind(&req.label)
                    .execute(p).await?;
                let id: (i64,) = sqlx::query_as("SELECT LAST_INSERT_ID()").fetch_one(p).await?;
                self.find_machine_ip_by_id(id.0).await?.ok_or_else(|| sqlx::Error::Configuration("Created machine IP not found after insert".into()))
            }
            Self::Mssql(db) => {
                let port = req.port;
                db.query_one::<MachineIp>(
                    "INSERT INTO machine_ips (machine_id, ip_address, port, label) OUTPUT INSERTED.* VALUES (@P1, @P2, @P3, @P4)",
                    tp!(req.machine_id, req.ip_address, port, req.label)
                ).await?.ok_or_else(|| sqlx::Error::Protocol("INSERT OUTPUT returned no rows".into()))
            }
        }
    }

    pub async fn find_machine_ip_by_id(&self, id: i64) -> Result<Option<MachineIp>, sqlx::Error> {
        match self {
            Self::NoDb => { Err(sqlx::Error::Configuration("Database not available".into()))},
                Self::Sqlite(p) => sqlx::query_as("SELECT * FROM machine_ips WHERE id = ?").bind(id).fetch_optional(p).await,
            Self::Postgres(p) => sqlx::query_as("SELECT * FROM machine_ips WHERE id = $1").bind(id).fetch_optional(p).await,
            Self::Mysql(p) => sqlx::query_as("SELECT * FROM machine_ips WHERE id = ?").bind(id).fetch_optional(p).await,
            Self::Mssql(db) => db.query_one::<MachineIp>("SELECT * FROM machine_ips WHERE id = @P1", tp!(id)).await,
        }
    }

    pub async fn update_machine_ip(&self, id: i64, req: &UpdateMachineIpRequest) -> Result<Option<MachineIp>, sqlx::Error> {
        let has_fields = req.ip_address.is_some()
            || req.port.is_some()
            || req.label.is_some()
            || req.is_active.is_some();
        if !has_fields { return self.find_machine_ip_by_id(id).await; }

        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => {
                let mut sets: Vec<&str> = Vec::new();
                if req.ip_address.is_some() { sets.push("ip_address = ?"); }
                if req.port.is_some() { sets.push("port = ?"); }
                if req.label.is_some() { sets.push("label = ?"); }
                if req.is_active.is_some() { sets.push("is_active = ?"); }
                sets.push("updated_at = CURRENT_TIMESTAMP");
                let sql = format!("UPDATE machine_ips SET {} WHERE id = ?", sets.join(", "));
                let mut q = sqlx::query(AssertSqlSafe(sql));
                if let Some(ref v) = req.ip_address { q = q.bind(v.as_str()); }
                if let Some(v) = req.port { q = q.bind(v); }
                if let Some(ref v) = req.label { q = q.bind(v.as_str()); }
                if let Some(v) = req.is_active { q = q.bind(if v { 1i32 } else { 0i32 }); }
                q.bind(id).execute(p).await?;
            }
            Self::Postgres(p) => {
                let mut sets: Vec<String> = Vec::new();
                let mut idx = 0u32;
                if req.ip_address.is_some() { idx += 1; sets.push(format!("ip_address = ${}", idx)); }
                if req.port.is_some() { idx += 1; sets.push(format!("port = ${}", idx)); }
                if req.label.is_some() { idx += 1; sets.push(format!("label = ${}", idx)); }
                if req.is_active.is_some() { idx += 1; sets.push(format!("is_active = ${}", idx)); }
                idx += 1;
                let sql = format!("UPDATE machine_ips SET {}, updated_at = CURRENT_TIMESTAMP WHERE id = ${}", sets.join(", "), idx);
                let mut q = sqlx::query(AssertSqlSafe(sql));
                if let Some(ref v) = req.ip_address { q = q.bind(v.as_str()); }
                if let Some(v) = req.port { q = q.bind(v); }
                if let Some(ref v) = req.label { q = q.bind(v.as_str()); }
                if let Some(v) = req.is_active { q = q.bind(if v { 1i32 } else { 0i32 }); }
                q.bind(id).execute(p).await?;
            }
            Self::Mysql(p) => {
                let mut sets: Vec<&str> = Vec::new();
                if req.ip_address.is_some() { sets.push("ip_address = ?"); }
                if req.port.is_some() { sets.push("port = ?"); }
                if req.label.is_some() { sets.push("label = ?"); }
                if req.is_active.is_some() { sets.push("is_active = ?"); }
                sets.push("updated_at = CURRENT_TIMESTAMP");
                let sql = format!("UPDATE machine_ips SET {} WHERE id = ?", sets.join(", "));
                let mut q = sqlx::query(AssertSqlSafe(sql));
                if let Some(ref v) = req.ip_address { q = q.bind(v.as_str()); }
                if let Some(v) = req.port { q = q.bind(v); }
                if let Some(ref v) = req.label { q = q.bind(v.as_str()); }
                if let Some(v) = req.is_active { q = q.bind(if v { 1i32 } else { 0i32 }); }
                q.bind(id).execute(p).await?;
            }
            Self::Mssql(db) => {
                let mut sets: Vec<String> = Vec::new();
                let mut idx = 0u32;
                if req.ip_address.is_some() { idx += 1; sets.push(format!("ip_address = @P{}", idx)); }
                if req.port.is_some() { idx += 1; sets.push(format!("port = @P{}", idx)); }
                if req.label.is_some() { idx += 1; sets.push(format!("label = @P{}", idx)); }
                if req.is_active.is_some() { idx += 1; sets.push(format!("is_active = @P{}", idx)); }
                idx += 1;
                let sql = format!("UPDATE machine_ips SET {}, updated_at = GETUTCDATE() WHERE id = @P{}", sets.join(", "), idx);
                let ip_str: &str = req.ip_address.as_deref().unwrap_or("");
                let label_str: &str = req.label.as_deref().unwrap_or("");
                let port_val: Option<i64> = req.port.map(|p| p as i64);
                let active_val: Option<i64> = req.is_active.map(|v| if v { 1i64 } else { 0i64 });
                let mut params: Vec<&dyn tiberius::ToSql> = Vec::new();
                if req.ip_address.is_some() { params.push(&ip_str as &dyn tiberius::ToSql); }
                if let Some(ref v) = port_val { params.push(v as &dyn tiberius::ToSql); }
                if req.label.is_some() { params.push(&label_str as &dyn tiberius::ToSql); }
                if let Some(ref v) = active_val { params.push(v as &dyn tiberius::ToSql); }
                params.push(&id as &dyn tiberius::ToSql);
                db.execute(&sql, &params).await?;
            }
        }
        self.find_machine_ip_by_id(id).await
    }

    pub async fn delete_machine_ip(&self, id: i64) -> Result<bool, sqlx::Error> {
        let affected = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => {
                let r = sqlx::query("DELETE FROM machine_ips WHERE id = ?").bind(id).execute(p).await?;
                r.rows_affected()
            }
            Self::Postgres(p) => {
                let r = sqlx::query("DELETE FROM machine_ips WHERE id = $1").bind(id).execute(p).await?;
                r.rows_affected()
            }
            Self::Mysql(p) => {
                let r = sqlx::query("DELETE FROM machine_ips WHERE id = ?").bind(id).execute(p).await?;
                r.rows_affected()
            }
            Self::Mssql(db) => db.execute("DELETE FROM machine_ips WHERE id = @P1", tp!(id)).await?
        };
        Ok(affected > 0)
    }

    // --- Dashboard ---
    pub async fn patient_dashboard(&self, patient_id: i64, signal_ids: Option<&[i64]>, date_from: Option<&str>, date_to: Option<&str>) -> Result<PatientDashboard, sqlx::Error> {
        let mut extra_where = String::new();
        let sig_count = signal_ids.map(|ids| ids.len()).unwrap_or(0);
        if sig_count > 0 {
            let ph: Vec<&str> = vec!["?"; sig_count];
            extra_where = format!(" AND te.signal_id IN ({})", ph.join(", "));
        }
        if date_from.is_some() { extra_where.push_str(" AND te.timestamp >= ?"); }
        if date_to.is_some() { extra_where.push_str(" AND te.timestamp <= ?"); }

        let agg_base = "SELECT te.signal_id, s.internal_name, s.display_name, s.unit, AVG(CAST(te.physical_value AS REAL)) as average, MIN(CAST(te.physical_value AS REAL)) as minimum, MAX(CAST(te.physical_value AS REAL)) as maximum, COUNT(*) as count FROM telemetry te JOIN therapies t ON te.therapy_id = t.id JOIN signals s ON te.signal_id = s.id WHERE t.patient_id = ? AND te.physical_value IS NOT NULL AND te.physical_value != ''";
        let batch_base = "SELECT te.signal_id, te.timestamp, te.physical_value FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = ? AND te.physical_value IS NOT NULL AND te.physical_value != ''";
        let order = " GROUP BY te.signal_id, s.internal_name, s.display_name, s.unit ORDER BY te.signal_id";
        let order_batch = " ORDER BY te.signal_id, te.timestamp ASC";

        macro_rules! bind_extras_dash {
            ($q:expr, $ids:expr, $from:expr, $to:expr) => {{
                let mut q = $q;
                if let Some(ids) = $ids { for id in ids { q = q.bind(id); } }
                if let Some(from) = $from { q = q.bind(from); }
                if let Some(to) = $to { q = q.bind(to); }
                q
            }};
        }

        let raw_signals: Vec<DashboardSignalRaw> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => {
                let sql = format!("{}{}{}", agg_base, extra_where, order);
                let mut q = sqlx::query_as(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras_dash!(q, signal_ids, date_from, date_to);
                q.fetch_all(p).await?
            }
            Self::Postgres(p) => {
                let safe_cast = "AVG(CASE WHEN te.physical_value ~ '^[0-9]+(\\.[0-9]+)?$' THEN te.physical_value::double precision END) as average, MIN(CASE WHEN te.physical_value ~ '^[0-9]+(\\.[0-9]+)?$' THEN te.physical_value::double precision END) as minimum, MAX(CASE WHEN te.physical_value ~ '^[0-9]+(\\.[0-9]+)?$' THEN te.physical_value::double precision END) as maximum";
                let agg_pg = agg_base.replace("AVG(CAST(te.physical_value AS REAL)) as average, MIN(CAST(te.physical_value AS REAL)) as minimum, MAX(CAST(te.physical_value AS REAL)) as maximum", safe_cast);
                let mut pg_where = extra_where.clone();
                let mut ph_idx = 2u32;
                while pg_where.contains('?') {
                    pg_where = pg_where.replacen('?', &format!("${}", ph_idx), 1);
                    ph_idx += 1;
                }
                let sql = format!("{}{}{}", agg_pg.replace('?', "$1"), pg_where, order);
                let mut q = sqlx::query_as(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras_dash!(q, signal_ids, date_from, date_to);
                q.fetch_all(p).await?
            }
            Self::Mysql(p) => {
                let sql = format!("{}{}{}", agg_base, extra_where, order);
                let mut q = sqlx::query_as(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras_dash!(q, signal_ids, date_from, date_to);
                q.fetch_all(p).await?
            }
            Self::Mssql(db) => {
                let safe_cast = agg_base.replace("CAST(te.physical_value AS REAL)", "TRY_CAST(te.physical_value AS REAL)");
                let mut ms_where = extra_where.clone();
                let mut ph_idx = 2u32;
                while ms_where.contains('?') {
                    ms_where = ms_where.replacen('?', &format!("@P{}", ph_idx), 1);
                    ph_idx += 1;
                }
                let sql = format!("{}{}{}", safe_cast.replace('?', "@P1"), ms_where, order);
                let signal_ids_local: &[i64] = signal_ids.unwrap_or(&[]);
                let date_from_str: &str = date_from.unwrap_or("");
                let date_to_str: &str = date_to.unwrap_or("");
                let mut params: Vec<&dyn tiberius::ToSql> = vec![&patient_id as &dyn tiberius::ToSql];
                for id in signal_ids_local { params.push(id as &dyn tiberius::ToSql); }
                if date_from.is_some() { params.push(&date_from_str as &dyn tiberius::ToSql); }
                if date_to.is_some() { params.push(&date_to_str as &dyn tiberius::ToSql); }
                db.query_all::<DashboardSignalRaw>(&sql, &params).await?
            }
        };

        let all_values: Vec<DashboardValueWithSignal> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => {
                let sql = format!("{}{}{}", batch_base, extra_where, order_batch);
                let mut q = sqlx::query_as(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras_dash!(q, signal_ids, date_from, date_to);
                q.fetch_all(p).await?
            }
            Self::Postgres(p) => {
                let mut pg_where = extra_where.clone();
                let mut ph_idx = 2u32;
                while pg_where.contains('?') {
                    pg_where = pg_where.replacen('?', &format!("${}", ph_idx), 1);
                    ph_idx += 1;
                }
                let sql = format!("{}{}{}", batch_base.replace('?', "$1"), pg_where, order_batch);
                let mut q = sqlx::query_as(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras_dash!(q, signal_ids, date_from, date_to);
                q.fetch_all(p).await?
            }
            Self::Mysql(p) => {
                let sql = format!("{}{}{}", batch_base, extra_where, order_batch);
                let mut q = sqlx::query_as(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras_dash!(q, signal_ids, date_from, date_to);
                q.fetch_all(p).await?
            }
            Self::Mssql(db) => {
                let mut ms_where = extra_where.clone();
                let mut ph_idx = 2u32;
                while ms_where.contains('?') {
                    ms_where = ms_where.replacen('?', &format!("@P{}", ph_idx), 1);
                    ph_idx += 1;
                }
                let sql = format!("{}{}{}", batch_base.replace('?', "@P1"), ms_where, order_batch);
                let signal_ids_local: &[i64] = signal_ids.unwrap_or(&[]);
                let date_from_str: &str = date_from.unwrap_or("");
                let date_to_str: &str = date_to.unwrap_or("");
                let mut params: Vec<&dyn tiberius::ToSql> = vec![&patient_id as &dyn tiberius::ToSql];
                for id in signal_ids_local { params.push(id as &dyn tiberius::ToSql); }
                if date_from.is_some() { params.push(&date_from_str as &dyn tiberius::ToSql); }
                if date_to.is_some() { params.push(&date_to_str as &dyn tiberius::ToSql); }
                db.query_all::<DashboardValueWithSignal>(&sql, &params).await?
            }
        };

        let mut values_by_signal: HashMap<i64, Vec<DashboardValue>> = HashMap::new();
        for v in &all_values {
            if let Some(ts) = v.timestamp
                && let Some(val) = v.physical_value.as_deref().and_then(|s| s.parse::<f64>().ok()) {
                    values_by_signal.entry(v.signal_id).or_default().push(DashboardValue { timestamp: ts, value: val });
                }
        }
        let mut signals = Vec::with_capacity(raw_signals.len());
        for sig in raw_signals {
            let values = values_by_signal.remove(&sig.signal_id).unwrap_or_default();
            signals.push(DashboardSignal {
                signal_id: sig.signal_id, internal_name: sig.internal_name,
                display_name: sig.display_name, unit: sig.unit,
                average: sig.average, minimum: sig.minimum, maximum: sig.maximum,
                count: sig.count, values,
            });
        }
        Ok(PatientDashboard { signals })
    }

    pub async fn therapy_dashboard(&self, therapy_id: i64) -> Result<PatientDashboard, sqlx::Error> {
        let agg_sql = "SELECT te.signal_id, s.internal_name, s.display_name, s.unit, AVG(CAST(te.physical_value AS REAL)) as average, MIN(CAST(te.physical_value AS REAL)) as minimum, MAX(CAST(te.physical_value AS REAL)) as maximum, COUNT(*) as count FROM telemetry te JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = ? AND te.physical_value IS NOT NULL AND te.physical_value != '' GROUP BY te.signal_id, s.internal_name, s.display_name, s.unit ORDER BY te.signal_id";
        let batch_vals_sql = "SELECT te.signal_id, te.timestamp, te.physical_value FROM telemetry te WHERE te.therapy_id = ? AND te.physical_value IS NOT NULL AND te.physical_value != '' ORDER BY te.signal_id, te.timestamp ASC";

        let raw_signals: Vec<DashboardSignalRaw> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => sqlx::query_as(agg_sql).bind(therapy_id).fetch_all(p).await?,
            Self::Postgres(p) => {
                let pg_sql = agg_sql.replace('?', "$1");
                sqlx::query_as(AssertSqlSafe(pg_sql)).bind(therapy_id).fetch_all(p).await?
            }
            Self::Mysql(p) => sqlx::query_as(agg_sql).bind(therapy_id).fetch_all(p).await?,
            Self::Mssql(db) => {
                let mssql_sql = agg_sql.replace('?', "@P1").replace("CAST(te.physical_value AS REAL)", "TRY_CAST(te.physical_value AS REAL)");
                db.query_all::<DashboardSignalRaw>(&mssql_sql, tp!(therapy_id)).await?
            }
        };

        let all_values: Vec<DashboardValueWithSignal> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => sqlx::query_as(batch_vals_sql).bind(therapy_id).fetch_all(p).await?,
            Self::Postgres(p) => {
                let pg_sql = batch_vals_sql.replace('?', "$1");
                sqlx::query_as(AssertSqlSafe(pg_sql)).bind(therapy_id).fetch_all(p).await?
            }
            Self::Mysql(p) => sqlx::query_as(batch_vals_sql).bind(therapy_id).fetch_all(p).await?,
            Self::Mssql(db) => db.query_all::<DashboardValueWithSignal>(
                "SELECT te.signal_id, te.timestamp, te.physical_value FROM telemetry te WHERE te.therapy_id = @P1 AND te.physical_value IS NOT NULL AND te.physical_value != '' ORDER BY te.signal_id, te.timestamp ASC",
                tp!(therapy_id)
            ).await?,
        };

        let mut values_by_signal: HashMap<i64, Vec<DashboardValue>> = HashMap::new();
        for v in &all_values {
            if let Some(ts) = v.timestamp
                && let Some(val) = v.physical_value.as_deref().and_then(|s| s.parse::<f64>().ok()) {
                    values_by_signal.entry(v.signal_id).or_default().push(DashboardValue { timestamp: ts, value: val });
                }
        }
        let mut signals = Vec::with_capacity(raw_signals.len());
        for sig in raw_signals {
            let values = values_by_signal.remove(&sig.signal_id).unwrap_or_default();
            signals.push(DashboardSignal {
                signal_id: sig.signal_id, internal_name: sig.internal_name,
                display_name: sig.display_name, unit: sig.unit,
                average: sig.average, minimum: sig.minimum, maximum: sig.maximum,
                count: sig.count, values,
            });
        }
        Ok(PatientDashboard { signals })
    }

    // --- Export (limited to 100k rows to prevent OOM) ---
    pub async fn export_patient_telemetry(&self, patient_id: i64) -> Result<Vec<TelemetryExportRow>, sqlx::Error> {
        const EXPORT_LIMIT: i64 = 100_000;
        let sql = "SELECT te.id, te.timestamp, te.signal_id, te.physical_value, COALESCE(s.unit, te.unit) as unit, COALESCE(s.display_name, s.internal_name) as signal_name FROM telemetry te JOIN therapies t ON te.therapy_id = t.id LEFT JOIN signals s ON te.signal_id = s.id WHERE t.patient_id = ? ORDER BY te.timestamp";
        let limited_sql = match self {
            Self::Sqlite(_) | Self::Mysql(_) => format!("{} LIMIT {}", sql, EXPORT_LIMIT),
            Self::Postgres(_) => format!("{} LIMIT ${}", sql.replace('?', "$1"), EXPORT_LIMIT),
            Self::Mssql(_) => format!("{} OFFSET 0 ROWS FETCH NEXT {} ROWS ONLY", sql.replace('?', "@P1"), EXPORT_LIMIT),
            Self::NoDb => String::new(),
        };
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql)).bind(patient_id).fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql)).bind(patient_id).fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql)).bind(patient_id).fetch_all(p).await,
            Self::Mssql(db) => db.query_all::<TelemetryExportRow>(&limited_sql, tp!(patient_id)).await,
        }
    }

    pub async fn export_therapy_telemetry(&self, therapy_id: i64) -> Result<Vec<TelemetryExportRow>, sqlx::Error> {
        const EXPORT_LIMIT: i64 = 100_000;
        let sql = "SELECT te.id, te.timestamp, te.signal_id, te.physical_value, COALESCE(s.unit, te.unit) as unit, COALESCE(s.display_name, s.internal_name) as signal_name FROM telemetry te LEFT JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = ? ORDER BY te.timestamp";
        let limited_sql = match self {
            Self::Sqlite(_) | Self::Mysql(_) => format!("{} LIMIT {}", sql, EXPORT_LIMIT),
            Self::Postgres(_) => format!("{} LIMIT ${}", sql.replace('?', "$1"), EXPORT_LIMIT),
            Self::Mssql(_) => format!("{} OFFSET 0 ROWS FETCH NEXT {} ROWS ONLY", sql.replace('?', "@P1"), EXPORT_LIMIT),
            Self::NoDb => String::new(),
        };
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql)).bind(therapy_id).fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql)).bind(therapy_id).fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql)).bind(therapy_id).fetch_all(p).await,
            Self::Mssql(db) => db.query_all::<TelemetryExportRow>(&limited_sql, tp!(therapy_id)).await,
        }
    }

    pub async fn load_equivalences(&self) -> Result<Vec<AttributeEquivalence>, sqlx::Error> {
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => sqlx::query_as::<_, AttributeEquivalence>("SELECT signal_id, numeric_value, display_name FROM attribute_equivalences").fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as::<_, AttributeEquivalence>("SELECT signal_id, numeric_value, display_name FROM attribute_equivalences").fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as::<_, AttributeEquivalence>("SELECT signal_id, numeric_value, display_name FROM attribute_equivalences").fetch_all(p).await,
            Self::Mssql(db) => db.query_all::<AttributeEquivalence>("SELECT signal_id, numeric_value, display_name FROM attribute_equivalences", &[]).await,
        }
    }

    // --- Equivalences CRUD ---
    pub async fn list_equivalences_with_signals(&self) -> Result<Vec<EquivalenceResponse>, sqlx::Error> {
        let sql = "SELECT ae.signal_id, s.internal_name, ae.numeric_value, ae.display_name FROM attribute_equivalences ae JOIN signals s ON ae.signal_id = s.id ORDER BY s.internal_name, ae.numeric_value";
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => sqlx::query_as::<_, EquivalenceResponse>(sql).fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as::<_, EquivalenceResponse>(sql).fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as::<_, EquivalenceResponse>(sql).fetch_all(p).await,
            Self::Mssql(db) => db.query_all::<EquivalenceResponse>(sql, &[]).await,
        }
    }

    pub async fn get_or_create_signal(&self, internal_name: &str) -> Result<i64, sqlx::Error> {
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => {
                if let Some((id,)) = sqlx::query_as::<_, (i64,)>("SELECT id FROM signals WHERE internal_name = ?")
                    .bind(internal_name).fetch_optional(p).await? {
                    return Ok(id);
                }
                sqlx::query("INSERT INTO signals (internal_name) VALUES (?)")
                    .bind(internal_name).execute(p).await?;
                let (id,): (i64,) = sqlx::query_as("SELECT last_insert_rowid()").fetch_one(p).await?;
                Ok(id)
            }
            Self::Postgres(p) => {
                let (id,): (i64,) = sqlx::query_as(
                    "INSERT INTO signals (internal_name) VALUES ($1) ON CONFLICT (internal_name) DO UPDATE SET internal_name = EXCLUDED.internal_name RETURNING id"
                ).bind(internal_name).fetch_one(p).await?;
                Ok(id)
            }
            Self::Mysql(p) => {
                if let Some((id,)) = sqlx::query_as::<_, (i64,)>("SELECT id FROM signals WHERE internal_name = ?")
                    .bind(internal_name).fetch_optional(p).await? {
                    return Ok(id);
                }
                sqlx::query("INSERT INTO signals (internal_name) VALUES (?)")
                    .bind(internal_name).execute(p).await?;
                let (id,): (i64,) = sqlx::query_as("SELECT LAST_INSERT_ID()").fetch_one(p).await?;
                Ok(id)
            }
            Self::Mssql(db) => {
                if let Some(id) = db.query_scalar::<i64>("SELECT id FROM signals WHERE internal_name = @P1", tp!(internal_name)).await.ok() {
                    return Ok(id);
                }
                db.query_scalar::<i64>("INSERT INTO signals (internal_name) OUTPUT INSERTED.id VALUES (@P1)", tp!(internal_name)).await
            }
        }
    }

    pub async fn upsert_equivalence(&self, signal_id: i64, numeric_value: f64, display_name: &str) -> Result<(), sqlx::Error> {
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => {
                sqlx::query(
                    "INSERT INTO attribute_equivalences (signal_id, numeric_value, display_name) VALUES (?, ?, ?) ON CONFLICT(signal_id, numeric_value) DO UPDATE SET display_name = excluded.display_name"
                ).bind(signal_id).bind(numeric_value).bind(display_name).execute(p).await?;
                Ok(())
            }
            Self::Postgres(p) => {
                sqlx::query(
                    "INSERT INTO attribute_equivalences (signal_id, numeric_value, display_name) VALUES ($1, $2, $3) ON CONFLICT(signal_id, numeric_value) DO UPDATE SET display_name = EXCLUDED.display_name"
                ).bind(signal_id).bind(numeric_value).bind(display_name).execute(p).await?;
                Ok(())
            }
            Self::Mysql(p) => {
                sqlx::query(
                    "INSERT INTO attribute_equivalences (signal_id, numeric_value, display_name) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE display_name = VALUES(display_name)"
                ).bind(signal_id).bind(numeric_value).bind(display_name).execute(p).await?;
                Ok(())
            }
            Self::Mssql(db) => {
                db.execute(
                    "MERGE attribute_equivalences AS target USING (SELECT @P1 AS signal_id, @P2 AS numeric_value, @P3 AS display_name) AS source ON (target.signal_id = source.signal_id AND target.numeric_value = source.numeric_value) WHEN MATCHED THEN UPDATE SET display_name = source.display_name WHEN NOT MATCHED THEN INSERT (signal_id, numeric_value, display_name) VALUES (source.signal_id, source.numeric_value, source.display_name);",
                    tp!(signal_id, numeric_value, display_name)
                ).await?;
                Ok(())
            }
        }
    }

    pub async fn update_equivalence(&self, signal_id: i64, numeric_value: f64, display_name: &str) -> Result<(), sqlx::Error> {
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => {
                sqlx::query("UPDATE attribute_equivalences SET display_name = ? WHERE signal_id = ? AND numeric_value = ?")
                    .bind(display_name).bind(signal_id).bind(numeric_value).execute(p).await?;
                Ok(())
            }
            Self::Postgres(p) => {
                sqlx::query("UPDATE attribute_equivalences SET display_name = $1 WHERE signal_id = $2 AND numeric_value = $3")
                    .bind(display_name).bind(signal_id).bind(numeric_value).execute(p).await?;
                Ok(())
            }
            Self::Mysql(p) => {
                sqlx::query("UPDATE attribute_equivalences SET display_name = ? WHERE signal_id = ? AND numeric_value = ?")
                    .bind(display_name).bind(signal_id).bind(numeric_value).execute(p).await?;
                Ok(())
            }
            Self::Mssql(db) => {
                db.execute("UPDATE attribute_equivalences SET display_name = @P1 WHERE signal_id = @P2 AND numeric_value = @P3", tp!(display_name, signal_id, numeric_value)).await?;
                Ok(())
            }
        }
    }

    pub async fn delete_equivalence_with_log(&self, signal_id: i64, numeric_value: f64, deleted_by: &str, deletion_reason: &str) -> Result<(), sqlx::Error> {
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => {
                sqlx::query("INSERT INTO equivalence_deletion_log (signal_id, numeric_value, deleted_by, deletion_reason) VALUES (?, ?, ?, ?)")
                    .bind(signal_id).bind(numeric_value).bind(deleted_by).bind(deletion_reason).execute(p).await?;
                sqlx::query("DELETE FROM attribute_equivalences WHERE signal_id = ? AND numeric_value = ?")
                    .bind(signal_id).bind(numeric_value).execute(p).await?;
                Ok(())
            }
            Self::Postgres(p) => {
                sqlx::query("INSERT INTO equivalence_deletion_log (signal_id, numeric_value, deleted_by, deletion_reason) VALUES ($1, $2, $3, $4)")
                    .bind(signal_id).bind(numeric_value).bind(deleted_by).bind(deletion_reason).execute(p).await?;
                sqlx::query("DELETE FROM attribute_equivalences WHERE signal_id = $1 AND numeric_value = $2")
                    .bind(signal_id).bind(numeric_value).execute(p).await?;
                Ok(())
            }
            Self::Mysql(p) => {
                sqlx::query("INSERT INTO equivalence_deletion_log (signal_id, numeric_value, deleted_by, deletion_reason) VALUES (?, ?, ?, ?)")
                    .bind(signal_id).bind(numeric_value).bind(deleted_by).bind(deletion_reason).execute(p).await?;
                sqlx::query("DELETE FROM attribute_equivalences WHERE signal_id = ? AND numeric_value = ?")
                    .bind(signal_id).bind(numeric_value).execute(p).await?;
                Ok(())
            }
            Self::Mssql(db) => {
                db.execute("INSERT INTO equivalence_deletion_log (signal_id, numeric_value, deleted_by, deletion_reason) VALUES (@P1, @P2, @P3, @P4)", tp!(signal_id, numeric_value, deleted_by, deletion_reason)).await?;
                db.execute("DELETE FROM attribute_equivalences WHERE signal_id = @P1 AND numeric_value = @P2", tp!(signal_id, numeric_value)).await?;
                Ok(())
            }
        }
    }

    // --- Signals CRUD ---
    pub async fn list_signals(&self) -> Result<Vec<Signal>, sqlx::Error> {
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => sqlx::query_as::<_, Signal>("SELECT id, internal_name, display_name, unit FROM signals ORDER BY internal_name").fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as::<_, Signal>("SELECT id, internal_name, display_name, unit FROM signals ORDER BY internal_name").fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as::<_, Signal>("SELECT id, internal_name, display_name, unit FROM signals ORDER BY internal_name").fetch_all(p).await,
            Self::Mssql(db) => db.query_all::<Signal>("SELECT id, internal_name, display_name, unit FROM signals ORDER BY internal_name", &[]).await,
        }
    }

    pub async fn update_signal(&self, id: i64, display_name: Option<&str>, unit: Option<&str>) -> Result<(), sqlx::Error> {
        let has_display = display_name.is_some();
        let has_unit = unit.is_some();
        if !has_display && !has_unit {
            return Ok(());
        }
        let sql = {
            let mut sets: Vec<&str> = Vec::new();
            if has_display { sets.push("display_name = ?"); }
            if has_unit { sets.push("unit = ?"); }
            format!("UPDATE signals SET {} WHERE id = ?", sets.join(", "))
        };
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => {
                let mut q = sqlx::query(AssertSqlSafe(sql.clone()));
                if let Some(v) = display_name { q = q.bind(v); }
                if let Some(v) = unit { q = q.bind(v); }
                q.bind(id).execute(p).await?;
                Ok(())
            }
            Self::Postgres(p) => {
                let mut sets: Vec<String> = Vec::new();
                let mut ph = 1u32;
                if has_display { sets.push(format!("display_name = ${}", ph)); ph += 1; }
                if has_unit { sets.push(format!("unit = ${}", ph)); ph += 1; }
                let pg_sql = format!("UPDATE signals SET {} WHERE id = ${}", sets.join(", "), ph);
                let mut q = sqlx::query(AssertSqlSafe(pg_sql));
                if let Some(v) = display_name { q = q.bind(v); }
                if let Some(v) = unit { q = q.bind(v); }
                q.bind(id).execute(p).await?;
                Ok(())
            }
            Self::Mysql(p) => {
                let mut q = sqlx::query(AssertSqlSafe(sql.clone()));
                if let Some(v) = display_name { q = q.bind(v); }
                if let Some(v) = unit { q = q.bind(v); }
                q.bind(id).execute(p).await?;
                Ok(())
            }
            Self::Mssql(db) => {
                let mut sets: Vec<String> = Vec::new();
                let mut ph = 1u32;
                if has_display { sets.push(format!("display_name = @P{}", ph)); ph += 1; }
                if has_unit { sets.push(format!("unit = @P{}", ph)); ph += 1; }
                let mssql_sql = format!("UPDATE signals SET {} WHERE id = @P{}", sets.join(", "), ph);
                let display_str: &str = display_name.unwrap_or("");
                let unit_str: &str = unit.unwrap_or("");
                let mut params: Vec<&dyn tiberius::ToSql> = Vec::new();
                if has_display { params.push(&display_str as &dyn tiberius::ToSql); }
                if has_unit { params.push(&unit_str as &dyn tiberius::ToSql); }
                params.push(&id as &dyn tiberius::ToSql);
                db.execute(&mssql_sql, &params).await?;
                Ok(())
            }
        }
    }

    pub async fn list_machines(&self) -> Result<Vec<Machine>, sqlx::Error> {
        match self {
            Self::NoDb => { Err(sqlx::Error::Configuration("Database not available".into()))},
                Self::Sqlite(p) => sqlx::query_as::<_, Machine>("SELECT * FROM machines ORDER BY serial_number").fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as::<_, Machine>("SELECT * FROM machines ORDER BY serial_number").fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as::<_, Machine>("SELECT * FROM machines ORDER BY serial_number").fetch_all(p).await,
            Self::Mssql(db) => db.query_all::<Machine>("SELECT * FROM machines ORDER BY serial_number", &[]).await,
        }
    }

    pub async fn seed_admin(&self, password: &str) -> Result<(), sqlx::Error> {
        let count = self.count_users().await?;
        if count == 0 {
            let pw = crate::auth::hash_password(password)
                .map_err(|e| sqlx::Error::Configuration(format!("Password hashing failed: {}", e).into()))?;
            match self {
                Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => {
                    sqlx::query("INSERT INTO users (username, password, full_name, email, role, active) VALUES ('admin', ?, 'Administrator', 'admin@monitor.local', 'admin', 1)")
                        .bind(&pw).execute(p).await?;
                }
                Self::Postgres(p) => {
                    sqlx::query("INSERT INTO users (username, password, full_name, email, role, active) VALUES ('admin', $1, 'Administrator', 'admin@monitor.local', 'admin', TRUE)")
                        .bind(&pw).execute(p).await?;
                }
                Self::Mysql(p) => {
                    sqlx::query("INSERT INTO users (username, password, full_name, email, role, active) VALUES ('admin', ?, 'Administrator', 'admin@monitor.local', 'admin', TRUE)")
                        .bind(&pw).execute(p).await?;
                }
            Self::Mssql(db) => {
                    db.execute("INSERT INTO users (username, password, full_name, email, role, active) VALUES ('admin', @P1, 'Administrator', 'admin@monitor.local', 'admin', 1)", tp!(pw)).await?;
                }
            }
        }
        Ok(())
    }

    // --- Authorization Codes ---
    pub async fn create_authorization_code(&self, code: &str, user_id: i64, expires_at: Option<NaiveDateTime>) -> Result<(), sqlx::Error> {
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => {
                sqlx::query("INSERT INTO authorization_codes (code, user_id, expires_at, used) VALUES (?, ?, ?, 0)")
                    .bind(code).bind(user_id).bind(expires_at).execute(p).await?;
                Ok(())
            }
            Self::Postgres(p) => {
                sqlx::query("INSERT INTO authorization_codes (code, user_id, expires_at, used) VALUES ($1, $2, $3, FALSE)")
                    .bind(code).bind(user_id).bind(expires_at).execute(p).await?;
                Ok(())
            }
            Self::Mysql(p) => {
                sqlx::query("INSERT INTO authorization_codes (code, user_id, expires_at, used) VALUES (?, ?, ?, 0)")
                    .bind(code).bind(user_id).bind(expires_at).execute(p).await?;
                Ok(())
            }
            Self::Mssql(db) => {
                db.execute(
                    "INSERT INTO authorization_codes (code, user_id, expires_at, used) VALUES (@P1, @P2, @P3, 0)",
                    tp!(code, user_id, expires_at)
                ).await?;
                Ok(())
            }
        }
    }

    pub async fn find_authorization_code(&self, code: &str) -> Result<Option<AuthorizationCode>, sqlx::Error> {
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => sqlx::query_as("SELECT * FROM authorization_codes WHERE code = ?").bind(code).fetch_optional(p).await,
            Self::Postgres(p) => sqlx::query_as("SELECT * FROM authorization_codes WHERE code = $1").bind(code).fetch_optional(p).await,
            Self::Mysql(p) => sqlx::query_as("SELECT * FROM authorization_codes WHERE code = ?").bind(code).fetch_optional(p).await,
            Self::Mssql(db) => db.query_one::<AuthorizationCode>("SELECT * FROM authorization_codes WHERE code = @P1", tp!(code)).await,
        }
    }
}

// Raw structs for JOIN queries
#[derive(Debug, Clone, sqlx::FromRow)]
struct TherapyRaw {
    pub id: i64,
    pub started_at: Option<NaiveDateTime>,
    #[allow(dead_code)]
    pub patient_id: Option<i64>,
    pub machine_id: Option<i64>,
    pub status: Option<String>,
    pub ended_at: Option<NaiveDateTime>,
    pub serial_number: Option<String>,
    pub software_version: Option<String>,
    pub ip_address: Option<String>,
    pub port: Option<i32>,
    pub therapy_type: Option<String>,
    pub kit: Option<String>,
    pub weight_initial: Option<String>,
    pub weight_final: Option<String>,
    pub therapy_type_signal_id: Option<i64>,
    pub kit_signal_id: Option<i64>,
    pub weight_initial_signal_id: Option<i64>,
    pub weight_final_signal_id: Option<i64>,
    pub patient_id_str: Option<String>,
}

impl From<TherapyRaw> for TherapyWithMachine {
    fn from(r: TherapyRaw) -> Self {
        Self { id: r.id, started_at: r.started_at, ended_at: r.ended_at, status: r.status, machine_id: r.machine_id, serial_number: r.serial_number, software_version: r.software_version, ip_address: r.ip_address, port: r.port, therapy_type: r.therapy_type, kit: r.kit, weight_initial: r.weight_initial, weight_final: r.weight_final, patient_id_str: r.patient_id_str }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct ActiveTherapyRaw {
    pub therapy_id: i64,
    pub patient_id: i64,
    pub patient_id_str: String,
    pub started_at: Option<NaiveDateTime>,
    pub serial_number: Option<String>,
    pub ip_address: Option<String>,
    pub port: Option<i32>,
    pub arterial_pressure: Option<String>,
    pub venous_pressure: Option<String>,
    pub blood_flow: Option<String>,
    pub weight_initial: Option<String>,
    pub weight_final: Option<String>,
    pub weight_initial_signal_id: Option<i64>,
    pub weight_final_signal_id: Option<i64>,
}

impl TryFromRow for ActiveTherapyRaw {
    fn try_from_row(row: &Row) -> Result<Self, sqlx::Error> {
        Ok(Self {
            therapy_id: col_i64(row, "therapy_id")?,
            patient_id: col_i64(row, "patient_id")?,
            patient_id_str: col_str(row, "patient_id_str")?,
            started_at: col_opt_dt(row, "started_at")?,
            serial_number: col_opt_str(row, "serial_number")?,
            ip_address: col_opt_str(row, "ip_address")?,
            port: col_val::<i32>(row, "port").ok(),
            arterial_pressure: col_opt_str(row, "arterial_pressure")?,
            venous_pressure: col_opt_str(row, "venous_pressure")?,
            blood_flow: col_opt_str(row, "blood_flow")?,
            weight_initial: col_opt_str(row, "weight_initial")?,
            weight_final: col_opt_str(row, "weight_final")?,
            weight_initial_signal_id: col_opt_i64(row, "weight_initial_signal_id")?,
            weight_final_signal_id: col_opt_i64(row, "weight_final_signal_id")?,
        })
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct MachineIpWithSerialRaw {
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

impl From<MachineIpWithSerialRaw> for MachineIpWithSerial {
    fn from(r: MachineIpWithSerialRaw) -> Self {
        Self { id: r.id, machine_id: r.machine_id, ip_address: r.ip_address, port: r.port, label: r.label, is_active: r.is_active, created_at: r.created_at, updated_at: r.updated_at, serial_number: r.serial_number }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct ActiveDeviceRaw {
    pub ip_address: String,
    pub port: Option<i32>,
    pub serial_number: Option<String>,
}

impl From<ActiveDeviceRaw> for ActiveDevice {
    fn from(r: ActiveDeviceRaw) -> Self {
        let url = match r.port {
            Some(p) => format!("http://{}:{}", r.ip_address, p),
            None => format!("http://{}", r.ip_address),
        };
        Self { url, ip_address: r.ip_address, port: r.port, serial_number: r.serial_number.unwrap_or_default() }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct DashboardSignalRaw {
    pub signal_id: i64,
    pub internal_name: String,
    pub display_name: Option<String>,
    pub unit: Option<String>,
    pub average: Option<f64>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub count: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct DashboardValueWithSignal {
    pub signal_id: i64,
    pub timestamp: Option<NaiveDateTime>,
    pub physical_value: Option<String>,
}
