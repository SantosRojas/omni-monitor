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
    pub fn from_env() -> Result<Self, String> {
        let db_connection = env::var("DB_CONNECTION").unwrap_or_else(|_| "sqlite".into());
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "change-me".into());
        if jwt_secret == "change-me" {
            return Err("JWT_SECRET is set to default value 'change-me' — this is INSECURE for production".into());
        }
        let cors_origins = {
            let raw = env::var("CORS_ORIGINS").unwrap_or_else(|_| "http://localhost:9002".into());
            let origins: Vec<String> = raw.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if origins.is_empty() {
                return Err("No valid CORS origins configured. Set CORS_ORIGINS environment variable.".into());
            }
            origins
        };
        Ok(Self {
            db_connection,
            db_host: env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            db_port: env::var("DB_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| {
                    match env::var("DB_CONNECTION").as_deref() {
                        Ok("postgres") | Ok("pgsql") | Ok("postgresql") => 5432,
                        Ok("mysql") | Ok("mariadb") => 3306,
                        _ => 1433,
                    }
                }),
            db_database: env::var("DB_DATABASE").unwrap_or_else(|_| "database.db".into()),
            db_username: env::var("DB_USERNAME").unwrap_or_else(|_| "root".into()),
            db_password: env::var("DB_PASSWORD").unwrap_or_default(),
            monitor_host: env::var("MONITOR_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            monitor_port: env::var("MONITOR_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(9002),
            jwt_secret,
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            jwt_issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| "monitor".into()),
            admin_password: env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".into()),
            cors_origins,
        })
    }
}
