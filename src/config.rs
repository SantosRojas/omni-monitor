use std::env;

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub db_connection: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_database: String,
    pub db_username: String,
    pub db_password: String,
    pub monitor_host: String,
    pub monitor_port: u16,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub jwt_issuer: String,
    pub admin_password: String,
    pub cors_origins: Vec<String>,
}

impl MonitorConfig {
    pub fn from_env() -> Self {
        Self {
            db_connection: env::var("DB_CONNECTION").unwrap_or_else(|_| "sqlite".into()),
            db_host: env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            db_port: env::var("DB_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1433),
            db_database: env::var("DB_DATABASE").unwrap_or_else(|_| "database.db".into()),
            db_username: env::var("DB_USERNAME").unwrap_or_else(|_| "root".into()),
            db_password: env::var("DB_PASSWORD").unwrap_or_default(),
            monitor_host: env::var("MONITOR_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            monitor_port: env::var("MONITOR_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(9002),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "change-me".into()),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            jwt_issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| "monitor".into()),
            admin_password: env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".into()),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:9002".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        }
    }
}
