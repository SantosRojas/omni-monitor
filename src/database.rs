use chrono::NaiveDateTime;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    postgres::{PgConnectOptions, PgPoolOptions},
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    AssertSqlSafe, ConnectOptions, MySql, Pool, Sqlite,
};
use sqlx_sqlserver::{Mssql, MssqlConnectOptions, MssqlPoolOptions};
use tracing::log::LevelFilter;
use std::str::FromStr;

use crate::config::MonitorConfig;
use crate::models::*;

#[derive(Debug, Clone)]
pub enum DbPool {
    NoDb,
    Sqlite(Pool<Sqlite>),
    Postgres(Pool<sqlx::Postgres>),
    Mysql(Pool<MySql>),
    Mssql(Pool<Mssql>),
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
                    "CREATE TABLE IF NOT EXISTS machine_ips (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        machine_id INTEGER NOT NULL,
                        ip_address TEXT NOT NULL,
                        port INTEGER DEFAULT 9001,
                        label TEXT DEFAULT '',
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
                    "CREATE TABLE IF NOT EXISTS machine_ips (
                        id BIGSERIAL PRIMARY KEY,
                        machine_id BIGINT NOT NULL REFERENCES machines(id),
                        ip_address TEXT NOT NULL,
                        port INTEGER DEFAULT 9001,
                        label TEXT DEFAULT '',
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
                    "CREATE TABLE IF NOT EXISTS machine_ips (
                        id BIGINT AUTO_INCREMENT PRIMARY KEY,
                        machine_id BIGINT NOT NULL,
                        ip_address VARCHAR(255) NOT NULL,
                        port INT DEFAULT 9001,
                        label VARCHAR(500) DEFAULT '',
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
                Ok(Self::Mysql(pool))
            }
            "mssql" | "sqlsrv" => {
                let opts = MssqlConnectOptions::new()
                    .set_host(&config.db_host)
                    .set_port(config.db_port)
                    .set_username(&config.db_username)
                    .set_password(&config.db_password)
                    .set_database(&config.db_database)
                    .trust_certificate()
                    .log_statements(LevelFilter::Debug);
                let pool = MssqlPoolOptions::new()
                    .max_connections(10)
                    .connect_with(opts)
                    .await?;
                sqlx::query(
                    "IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'machine_ips')
                        CREATE TABLE machine_ips (
                            id BIGINT IDENTITY(1,1) PRIMARY KEY,
                            machine_id BIGINT NOT NULL,
                            ip_address NVARCHAR(MAX) NOT NULL,
                            port INT DEFAULT 9001,
                            label NVARCHAR(500) DEFAULT '',
                            is_active BIT DEFAULT 1,
                            created_at DATETIME2 DEFAULT CURRENT_TIMESTAMP,
                            updated_at DATETIME2 DEFAULT CURRENT_TIMESTAMP
                        )",
                )
                .execute(&pool)
                .await?;
                sqlx::query(
                    "IF NOT EXISTS (SELECT * FROM sys.indexes WHERE name = 'idx_machine_ips_machine' AND object_id = OBJECT_ID('machine_ips'))
                        CREATE INDEX idx_machine_ips_machine ON machine_ips(machine_id)",
                )
                .execute(&pool).await?;
                sqlx::query(
                    "IF NOT EXISTS (SELECT * FROM sys.indexes WHERE name = 'idx_machine_ips_active' AND object_id = OBJECT_ID('machine_ips'))
                        CREATE INDEX idx_machine_ips_active ON machine_ips(machine_id, is_active)",
                )
                .execute(&pool).await?;
                Ok(Self::Mssql(pool))
            }
            other => Err(sqlx::Error::Configuration(
                format!("Unsupported DB_CONNECTION: {}. Supported: sqlite, postgres, mysql, mssql", other).into(),
            )),
        }
    }

    // --- Users ---
    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => sqlx::query_as("SELECT * FROM users WHERE username = ?").bind(username).fetch_optional(p).await,
            Self::Postgres(p) => sqlx::query_as("SELECT * FROM users WHERE username = $1").bind(username).fetch_optional(p).await,
            Self::Mysql(p) => sqlx::query_as("SELECT * FROM users WHERE username = ?").bind(username).fetch_optional(p).await,
            Self::Mssql(p) => sqlx::query_as("SELECT * FROM users WHERE username = @P1").bind(username).fetch_optional(p).await,
        }
    }

    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, sqlx::Error> {
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => sqlx::query_as("SELECT * FROM users WHERE id = ?").bind(id).fetch_optional(p).await,
            Self::Postgres(p) => sqlx::query_as("SELECT * FROM users WHERE id = $1").bind(id).fetch_optional(p).await,
            Self::Mysql(p) => sqlx::query_as("SELECT * FROM users WHERE id = ?").bind(id).fetch_optional(p).await,
            Self::Mssql(p) => sqlx::query_as("SELECT * FROM users WHERE id = @P1").bind(id).fetch_optional(p).await,
        }
    }

    pub async fn list_users(&self) -> Result<Vec<User>, sqlx::Error> {
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => sqlx::query_as("SELECT * FROM users ORDER BY id").fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as("SELECT * FROM users ORDER BY id").fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as("SELECT * FROM users ORDER BY id").fetch_all(p).await,
            Self::Mssql(p) => sqlx::query_as("SELECT * FROM users ORDER BY id").fetch_all(p).await,
        }
    }

    pub async fn create_user(&self, req: &CreateUserRequest) -> Result<User, sqlx::Error> {
        let pw = crate::auth::hash_password(&req.password).unwrap_or_default();
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
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
                self.find_user_by_id(id.0).await.map(|u| u.unwrap())
            }
            Self::Mssql(p) => {
                sqlx::query_as::<_, User>(
                    "INSERT INTO users (username, password, full_name, email, role, active) OUTPUT INSERTED.* VALUES (@P1, @P2, @P3, @P4, @P5, 1)"
                ).bind(&req.username).bind(&pw).bind(&req.full_name).bind(&req.email).bind(&req.role)
                .fetch_one(p).await
            }
        }
    }

    pub async fn update_user(&self, id: i64, req: &UpdateUserRequest) -> Result<Option<User>, sqlx::Error> {
        let pw = req.password.as_ref().map(|v| crate::auth::hash_password(v).unwrap_or_default());
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
            Self::Mssql(p) => {
                let mut sets: Vec<String> = Vec::new();
                let mut idx = 0u32;
                if pw.is_some() { idx += 1; sets.push(format!("password = @P{}", idx)); }
                if req.full_name.is_some() { idx += 1; sets.push(format!("full_name = @P{}", idx)); }
                if req.email.is_some() { idx += 1; sets.push(format!("email = @P{}", idx)); }
                if req.role.is_some() { idx += 1; sets.push(format!("role = @P{}", idx)); }
                if req.active.is_some() { idx += 1; sets.push(format!("active = @P{}", idx)); }
                idx += 1;
                let sql = format!("UPDATE users SET {} WHERE id = @P{}", sets.join(", "), idx);
                let mut q = sqlx::query(AssertSqlSafe(sql));
                if let Some(ref v) = pw { q = q.bind(v.as_str()); }
                if let Some(ref v) = req.full_name { q = q.bind(v.as_str()); }
                if let Some(ref v) = req.email { q = q.bind(v.as_str()); }
                if let Some(ref v) = req.role { q = q.bind(v.as_str()); }
                if let Some(v) = req.active { q = q.bind(if v { 1i32 } else { 0i32 }); }
                q.bind(id).execute(p).await?;
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
            Self::Mssql(p) => {
                let r = sqlx::query("DELETE FROM users WHERE id = @P1").bind(id).execute(p).await?;
                r.rows_affected()
            }
        };
        Ok(affected > 0)
    }

    pub async fn count_users(&self) -> Result<i64, sqlx::Error> {
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(p).await,
            Self::Postgres(p) => sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(p).await,
            Self::Mysql(p) => sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(p).await,
            Self::Mssql(p) => sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(p).await,
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
                Self::Mssql(p) => sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM patients WHERE patient_id_str LIKE @P1").bind(format!("%{}%", s)).fetch_one(p).await?,
            }
        } else {
            match self {
                Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => sqlx::query_scalar("SELECT COUNT(*) FROM patients").fetch_one(p).await?,
                Self::Postgres(p) => sqlx::query_scalar("SELECT COUNT(*) FROM patients").fetch_one(p).await?,
                Self::Mysql(p) => sqlx::query_scalar("SELECT COUNT(*) FROM patients").fetch_one(p).await?,
                Self::Mssql(p) => sqlx::query_scalar("SELECT COUNT(*) FROM patients").fetch_one(p).await?,
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
            Self::Mssql(p) => {
                if let Some(s) = search {
                    sqlx::query_as::<_, Patient>("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.patient_id_str LIKE @P1 ORDER BY active_therapy_count DESC, p.id DESC OFFSET @P2 ROWS FETCH NEXT @P3 ROWS ONLY")
                        .bind(format!("%{}%", s)).bind(offset).bind(per_page).fetch_all(p).await?
                } else {
                    sqlx::query_as::<_, Patient>("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p ORDER BY active_therapy_count DESC, p.id DESC OFFSET @P1 ROWS FETCH NEXT @P2 ROWS ONLY")
                        .bind(offset).bind(per_page).fetch_all(p).await?
                }
            }
        };
        Ok(PaginatedResponse { total: count_total, page, per_page, total_pages: (count_total as f64 / per_page as f64).ceil() as i64, data })
    }

    pub async fn find_patient_by_id(&self, id: i64) -> Result<Option<Patient>, sqlx::Error> {
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => sqlx::query_as("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.id = ?").bind(id).fetch_optional(p).await,
            Self::Postgres(p) => sqlx::query_as("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.id = $1").bind(id).fetch_optional(p).await,
            Self::Mysql(p) => sqlx::query_as("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.id = ?").bind(id).fetch_optional(p).await,
            Self::Mssql(p) => sqlx::query_as("SELECT p.*, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'active') as active_therapy_count, (SELECT COUNT(*) FROM therapies t WHERE t.patient_id = p.id AND t.status = 'completed') as completed_therapy_count FROM patients p WHERE p.id = @P1").bind(id).fetch_optional(p).await,
        }
    }

    // --- Therapies ---
    pub async fn list_therapies_by_patient(&self, patient_id: i64) -> Result<Vec<TherapyWithMachine>, sqlx::Error> {
        let raw: Vec<TherapyRaw> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id WHERE t.patient_id = ? ORDER BY t.started_at DESC")
                    .bind(patient_id).fetch_all(p).await?
            }
            Self::Postgres(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id WHERE t.patient_id = $1 ORDER BY t.started_at DESC")
                    .bind(patient_id).fetch_all(p).await?
            }
            Self::Mysql(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as ip_address, (SELECT mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id LIMIT 1) as port FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id WHERE t.patient_id = ? ORDER BY t.started_at DESC")
                    .bind(patient_id).fetch_all(p).await?
            }
            Self::Mssql(p) => {
                sqlx::query_as("SELECT t.id, t.started_at, t.patient_id, t.machine_id, t.status, t.ended_at, m.serial_number, m.software_version, (SELECT TOP 1 mi.ip_address FROM machine_ips mi WHERE mi.machine_id = t.machine_id) as ip_address, (SELECT TOP 1 mi.port FROM machine_ips mi WHERE mi.machine_id = t.machine_id) as port FROM therapies t LEFT JOIN machines m ON t.machine_id = m.id WHERE t.patient_id = @P1 ORDER BY t.started_at DESC")
                    .bind(patient_id).fetch_all(p).await?
            }
        };
        Ok(raw.into_iter().map(TherapyWithMachine::from).collect())
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
        if let Some(_) = date_from { extra_where.push_str(" AND te.timestamp >= ?"); }
        if let Some(_) = date_to { extra_where.push_str(" AND te.timestamp <= ?"); }

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
                let sql = format!("SELECT COUNT(*) FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = $1{}", where_ext.replace('?', "$2"));
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
            Self::Mssql(p) => {
                let mut q = sqlx::query_scalar::<_, i64>(
                    AssertSqlSafe(format!("SELECT COUNT(*) FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = @P1{}", where_ext.replace('?', "@P2")))
                ).bind(patient_id);
                q = bind_extras!(q, signal_ids, date_from, date_to);
                q.fetch_one(p).await?
            }
        };

        let data: Vec<TelemetryReading> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => {
                let sql = format!("SELECT te.* FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = ?{} ORDER BY te.timestamp DESC LIMIT ? OFFSET ?", where_ext);
                let mut q = sqlx::query_as::<_, TelemetryReading>(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras!(q, signal_ids, date_from, date_to);
                q.bind(per_page).bind(offset).fetch_all(p).await?
            }
            Self::Postgres(p) => {
                let base_ph = 1 + sig_count + date_from.map(|_| 1).unwrap_or(0) + date_to.map(|_| 1).unwrap_or(0);
                let limit_ph = base_ph + 1;
                let offset_ph = base_ph + 2;
                let pg_where = where_ext.replace('?', "$2");
                let sql = format!("SELECT te.* FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = $1{} ORDER BY te.timestamp DESC LIMIT ${} OFFSET ${}", pg_where, limit_ph, offset_ph);
                let mut q = sqlx::query_as::<_, TelemetryReading>(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras!(q, signal_ids, date_from, date_to);
                q.bind(per_page).bind(offset).fetch_all(p).await?
            }
            Self::Mysql(p) => {
                let sql = format!("SELECT te.* FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = ?{} ORDER BY te.timestamp DESC LIMIT ? OFFSET ?", where_ext);
                let mut q = sqlx::query_as::<_, TelemetryReading>(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras!(q, signal_ids, date_from, date_to);
                q.bind(per_page).bind(offset).fetch_all(p).await?
            }
            Self::Mssql(p) => {
                let base_ph = 1 + sig_count + date_from.map(|_| 1).unwrap_or(0) + date_to.map(|_| 1).unwrap_or(0);
                let offset_ph = base_ph + 1;
                let limit_ph = base_ph + 2;
                let ms_where = where_ext.replace('?', "@P2");
                let sql = format!("SELECT te.* FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = @P1{} ORDER BY te.timestamp DESC OFFSET @P{} ROWS FETCH NEXT @P{} ROWS ONLY", ms_where, offset_ph, limit_ph);
                let mut q = sqlx::query_as::<_, TelemetryReading>(AssertSqlSafe(sql)).bind(patient_id);
                q = bind_extras!(q, signal_ids, date_from, date_to);
                q.bind(offset).bind(per_page).fetch_all(p).await?
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
            Self::Mssql(p) => {
                sqlx::query_as::<_, ActiveDeviceRaw>(
                    "SELECT mi.ip_address, mi.port, m.serial_number FROM therapies t JOIN machines m ON t.machine_id = m.id JOIN machine_ips mi ON mi.machine_id = m.id AND mi.is_active = 1 WHERE t.patient_id = @P1 AND t.status = 'active' ORDER BY t.started_at DESC OFFSET 0 ROWS FETCH NEXT 1 ROWS ONLY"
                ).bind(patient_id).fetch_optional(p).await
            }
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
            Self::Mssql(p) => sqlx::query_as(sql).fetch_all(p).await?,
        };
        Ok(raw.into_iter().map(MachineIpWithSerial::from).collect())
    }

    pub async fn create_machine_ip(&self, req: &CreateMachineIpRequest) -> Result<MachineIp, sqlx::Error> {
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => {
                sqlx::query_as::<_, MachineIp>(
                    "INSERT INTO machine_ips (machine_id, ip_address, port, label) VALUES (?, ?, ?, ?) RETURNING *"
                ).bind(req.machine_id).bind(&req.ip_address).bind(req.port.unwrap_or(9001)).bind(&req.label)
                .fetch_one(p).await
            }
            Self::Postgres(p) => {
                sqlx::query_as::<_, MachineIp>(
                    "INSERT INTO machine_ips (machine_id, ip_address, port, label) VALUES ($1, $2, $3, $4) RETURNING *"
                ).bind(req.machine_id).bind(&req.ip_address).bind(req.port.unwrap_or(9001)).bind(&req.label)
                .fetch_one(p).await
            }
            Self::Mysql(p) => {
                sqlx::query("INSERT INTO machine_ips (machine_id, ip_address, port, label) VALUES (?, ?, ?, ?)")
                    .bind(req.machine_id).bind(&req.ip_address).bind(req.port.unwrap_or(9001)).bind(&req.label)
                    .execute(p).await?;
                let id: (i64,) = sqlx::query_as("SELECT LAST_INSERT_ID()").fetch_one(p).await?;
                self.find_machine_ip_by_id(id.0).await.map(|o| o.unwrap())
            }
            Self::Mssql(p) => {
                sqlx::query_as::<_, MachineIp>(
                    "INSERT INTO machine_ips (machine_id, ip_address, port, label) OUTPUT INSERTED.* VALUES (@P1, @P2, @P3, @P4)"
                ).bind(req.machine_id).bind(&req.ip_address).bind(req.port.unwrap_or(9001)).bind(&req.label)
                .fetch_one(p).await
            }
        }
    }

    pub async fn find_machine_ip_by_id(&self, id: i64) -> Result<Option<MachineIp>, sqlx::Error> {
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => sqlx::query_as("SELECT * FROM machine_ips WHERE id = ?").bind(id).fetch_optional(p).await,
            Self::Postgres(p) => sqlx::query_as("SELECT * FROM machine_ips WHERE id = $1").bind(id).fetch_optional(p).await,
            Self::Mysql(p) => sqlx::query_as("SELECT * FROM machine_ips WHERE id = ?").bind(id).fetch_optional(p).await,
            Self::Mssql(p) => sqlx::query_as("SELECT * FROM machine_ips WHERE id = @P1").bind(id).fetch_optional(p).await,
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
            Self::Mssql(p) => {
                let mut sets: Vec<String> = Vec::new();
                let mut idx = 0u32;
                if req.ip_address.is_some() { idx += 1; sets.push(format!("ip_address = @P{}", idx)); }
                if req.port.is_some() { idx += 1; sets.push(format!("port = @P{}", idx)); }
                if req.label.is_some() { idx += 1; sets.push(format!("label = @P{}", idx)); }
                if req.is_active.is_some() { idx += 1; sets.push(format!("is_active = @P{}", idx)); }
                idx += 1;
                let sql = format!("UPDATE machine_ips SET {}, updated_at = GETUTCDATE() WHERE id = @P{}", sets.join(", "), idx);
                let mut q = sqlx::query(AssertSqlSafe(sql));
                if let Some(ref v) = req.ip_address { q = q.bind(v.as_str()); }
                if let Some(v) = req.port { q = q.bind(v); }
                if let Some(ref v) = req.label { q = q.bind(v.as_str()); }
                if let Some(v) = req.is_active { q = q.bind(if v { 1i32 } else { 0i32 }); }
                q.bind(id).execute(p).await?;
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
            Self::Mssql(p) => {
                let r = sqlx::query("DELETE FROM machine_ips WHERE id = @P1").bind(id).execute(p).await?;
                r.rows_affected()
            }
        };
        Ok(affected > 0)
    }

    // --- Dashboard ---
    pub async fn patient_dashboard(&self, patient_id: i64, _signal_ids: Option<&[i64]>, _date_from: Option<&str>, _date_to: Option<&str>) -> Result<PatientDashboard, sqlx::Error> {
        let agg_sql = "SELECT te.signal_id, s.internal_name, s.display_name, s.unit, AVG(CAST(te.physical_value AS REAL)) as average, MIN(CAST(te.physical_value AS REAL)) as minimum, MAX(CAST(te.physical_value AS REAL)) as maximum, COUNT(*) as count FROM telemetry te JOIN therapies t ON te.therapy_id = t.id JOIN signals s ON te.signal_id = s.id WHERE t.patient_id = ? AND te.physical_value IS NOT NULL AND te.physical_value != '' GROUP BY te.signal_id, s.internal_name, s.display_name, s.unit ORDER BY te.signal_id";
        let batch_vals_sql = "SELECT te.signal_id, te.timestamp, te.physical_value FROM telemetry te JOIN therapies t ON te.therapy_id = t.id WHERE t.patient_id = ? AND te.physical_value IS NOT NULL AND te.physical_value != '' ORDER BY te.signal_id, te.timestamp ASC";

        let raw_signals: Vec<DashboardSignalRaw> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => sqlx::query_as(agg_sql).bind(patient_id).fetch_all(p).await?,
            Self::Postgres(p) => {
                let pg_sql = agg_sql.replace('?', "$1");
                sqlx::query_as(AssertSqlSafe(pg_sql)).bind(patient_id).fetch_all(p).await?
            }
            Self::Mysql(p) => sqlx::query_as(agg_sql).bind(patient_id).fetch_all(p).await?,
            Self::Mssql(p) => {
                let mssql_sql = agg_sql.replace('?', "@P1").replace("CAST(te.physical_value AS REAL)", "TRY_CAST(te.physical_value AS REAL)");
                sqlx::query_as(AssertSqlSafe(mssql_sql)).bind(patient_id).fetch_all(p).await?
            }
        };

        let all_values: Vec<DashboardValueWithSignal> = match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => sqlx::query_as(batch_vals_sql).bind(patient_id).fetch_all(p).await?,
            Self::Postgres(p) => {
                let pg_sql = batch_vals_sql.replace('?', "$1");
                sqlx::query_as(AssertSqlSafe(pg_sql)).bind(patient_id).fetch_all(p).await?
            }
            Self::Mysql(p) => sqlx::query_as(batch_vals_sql).bind(patient_id).fetch_all(p).await?,
            Self::Mssql(p) => {
                let ms_sql = batch_vals_sql.replace('?', "@P1");
                sqlx::query_as(AssertSqlSafe(ms_sql)).bind(patient_id).fetch_all(p).await?
            }
        };

        let mut signals = Vec::new();
        for sig in raw_signals {
            let values: Vec<DashboardValue> = all_values.iter()
                .filter(|v| v.signal_id == sig.signal_id)
                .filter_map(|v| {
                    let ts = v.timestamp?;
                    let val: f64 = v.physical_value.clone()?.parse().ok()?;
                    Some(DashboardValue { timestamp: ts, value: val })
                })
                .collect();
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
            Self::Mssql(p) => {
                let mssql_sql = agg_sql.replace('?', "@P1").replace("CAST(te.physical_value AS REAL)", "TRY_CAST(te.physical_value AS REAL)");
                sqlx::query_as(AssertSqlSafe(mssql_sql)).bind(therapy_id).fetch_all(p).await?
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
            Self::Mssql(p) => sqlx::query_as::<_, DashboardValueWithSignal>(
                "SELECT te.signal_id, te.timestamp, te.physical_value FROM telemetry te WHERE te.therapy_id = @P1 AND te.physical_value IS NOT NULL AND te.physical_value != '' ORDER BY te.signal_id, te.timestamp ASC"
            ).bind(therapy_id).fetch_all(p).await?,
        };

        let mut signals = Vec::new();
        for sig in raw_signals {
            let values: Vec<DashboardValue> = all_values.iter()
                .filter(|v| v.signal_id == sig.signal_id)
                .filter_map(|v| {
                    let ts = v.timestamp?;
                    let val: f64 = v.physical_value.clone()?.parse().ok()?;
                    Some(DashboardValue { timestamp: ts, value: val })
                })
                .collect();
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
            Self::NoDb => unreachable!(),
        };
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql.clone())).bind(patient_id).fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql)).bind(patient_id).fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql.clone())).bind(patient_id).fetch_all(p).await,
            Self::Mssql(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql)).bind(patient_id).fetch_all(p).await,
        }
    }

    pub async fn export_therapy_telemetry(&self, therapy_id: i64) -> Result<Vec<TelemetryExportRow>, sqlx::Error> {
        const EXPORT_LIMIT: i64 = 100_000;
        let sql = "SELECT te.id, te.timestamp, te.signal_id, te.physical_value, COALESCE(s.unit, te.unit) as unit, COALESCE(s.display_name, s.internal_name) as signal_name FROM telemetry te LEFT JOIN signals s ON te.signal_id = s.id WHERE te.therapy_id = ? ORDER BY te.timestamp";
        let limited_sql = match self {
            Self::Sqlite(_) | Self::Mysql(_) => format!("{} LIMIT {}", sql, EXPORT_LIMIT),
            Self::Postgres(_) => format!("{} LIMIT ${}", sql.replace('?', "$1"), EXPORT_LIMIT),
            Self::Mssql(_) => format!("{} OFFSET 0 ROWS FETCH NEXT {} ROWS ONLY", sql.replace('?', "@P1"), EXPORT_LIMIT),
            Self::NoDb => unreachable!(),
        };
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
            Self::Sqlite(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql.clone())).bind(therapy_id).fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql)).bind(therapy_id).fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql.clone())).bind(therapy_id).fetch_all(p).await,
            Self::Mssql(p) => sqlx::query_as::<_, TelemetryExportRow>(AssertSqlSafe(limited_sql)).bind(therapy_id).fetch_all(p).await,
        }
    }

    pub async fn load_equivalences(&self) -> Result<Vec<AttributeEquivalence>, sqlx::Error> {
        match self {
            Self::NoDb => Err(sqlx::Error::Configuration("Database not available".into())),
            Self::Sqlite(p) => sqlx::query_as::<_, AttributeEquivalence>("SELECT signal_id, numeric_value, display_name FROM attribute_equivalences").fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as::<_, AttributeEquivalence>("SELECT signal_id, numeric_value, display_name FROM attribute_equivalences").fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as::<_, AttributeEquivalence>("SELECT signal_id, numeric_value, display_name FROM attribute_equivalences").fetch_all(p).await,
            Self::Mssql(p) => sqlx::query_as::<_, AttributeEquivalence>("SELECT signal_id, numeric_value, display_name FROM attribute_equivalences").fetch_all(p).await,
        }
    }

    pub async fn list_machines(&self) -> Result<Vec<Machine>, sqlx::Error> {
        match self {
            Self::NoDb => { return Err(sqlx::Error::Configuration("Database not available".into())); },
                Self::Sqlite(p) => sqlx::query_as::<_, Machine>("SELECT * FROM machines ORDER BY serial_number").fetch_all(p).await,
            Self::Postgres(p) => sqlx::query_as::<_, Machine>("SELECT * FROM machines ORDER BY serial_number").fetch_all(p).await,
            Self::Mysql(p) => sqlx::query_as::<_, Machine>("SELECT * FROM machines ORDER BY serial_number").fetch_all(p).await,
            Self::Mssql(p) => sqlx::query_as::<_, Machine>("SELECT * FROM machines ORDER BY serial_number").fetch_all(p).await,
        }
    }

    pub async fn seed_admin(&self, password: &str) -> Result<(), sqlx::Error> {
        let count = self.count_users().await.unwrap_or(0);
        if count == 0 {
            let pw = crate::auth::hash_password(password).unwrap_or_default();
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
                Self::Mssql(p) => {
                    sqlx::query("INSERT INTO users (username, password, full_name, email, role, active) VALUES ('admin', @P1, 'Administrator', 'admin@monitor.local', 'admin', 1)")
                        .bind(&pw).execute(p).await?;
                }
            }
        }
        Ok(())
    }
}

// Raw structs for JOIN queries
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
struct TherapyRaw {
    pub id: i64,
    pub started_at: Option<NaiveDateTime>,
    pub patient_id: Option<i64>,
    pub machine_id: Option<i64>,
    pub status: Option<String>,
    pub ended_at: Option<NaiveDateTime>,
    pub serial_number: Option<String>,
    pub software_version: Option<String>,
    pub ip_address: Option<String>,
    pub port: Option<i32>,
}

impl From<TherapyRaw> for TherapyWithMachine {
    fn from(r: TherapyRaw) -> Self {
        Self { id: r.id, started_at: r.started_at, ended_at: r.ended_at, status: r.status, machine_id: r.machine_id, serial_number: r.serial_number, software_version: r.software_version, ip_address: r.ip_address, port: r.port }
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
        let port = r.port.unwrap_or(9001);
        Self { url: format!("http://{}:{}", r.ip_address, port), ip_address: r.ip_address, port, serial_number: r.serial_number.unwrap_or_default() }
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
