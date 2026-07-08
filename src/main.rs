#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;
    use tower_http::cors::{AllowOrigin, CorsLayer};
    use tower_http::services::{ServeDir, ServeFile};
    use tower_http::trace::TraceLayer;
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    dotenvy::dotenv().ok();

    let config = match monitor::config::MonitorConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Configuration error: {}. See .env.example for required settings.", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Starting monitor at http://{}:{}", config.monitor_host, config.monitor_port);

    let pool = match monitor::database::DbPool::from_config(&config).await {
        Ok(pool) => {
            if let Err(e) = pool.seed_admin(&config.admin_password).await {
                tracing::warn!("Failed to seed admin: {}", e);
            }
            Arc::new(pool)
        }
        Err(e) => {
            tracing::warn!(
                "Failed to connect to database: {}. Running without database. Only static files will be served.",
                e
            );
            Arc::new(monitor::database::DbPool::NoDb)
        }
    };

    let config = Arc::new(config);

    let api_state = monitor::api::AppState {
        pool: pool.clone(),
        config: config.clone(),
    };
    let api_router = monitor::api::create_router(api_state);

    let cors = if config.cors_origins.iter().any(|o| o == "*") {
        CorsLayer::permissive()
    } else {
        let origins: Vec<axum::http::HeaderValue> = config
            .cors_origins
            .iter()
            .filter_map(|o| o.parse::<axum::http::HeaderValue>().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([axum::http::header::CONTENT_TYPE])
            .allow_credentials(true)
    };

    let app = axum::Router::new()
        .nest("/api", api_router)
        .fallback_service(ServeDir::new("static").fallback(ServeFile::new("static/index.html")))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = match tokio::net::lookup_host(format!("{}:{}", config.monitor_host, config.monitor_port)).await {
        Ok(mut addrs) => match addrs.next() {
            Some(a) => a,
            None => {
                tracing::error!("No address resolved for {}:{}", config.monitor_host, config.monitor_port);
                std::process::exit(1);
            }
        },
        Err(e) => {
            tracing::error!("Failed to resolve host {}:{}: {}", config.monitor_host, config.monitor_port, e);
            std::process::exit(1);
        }
    };

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        tracing::error!("Server failed: {}", e);
        std::process::exit(1);
    }
}

#[cfg(feature = "ssr")]
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::info!("Received Ctrl+C, shutting down..."); }
        _ = terminate => { tracing::info!("Received SIGTERM, shutting down..."); }
    }
}

#[cfg(not(feature = "ssr"))]
fn main() {
    eprintln!("This binary requires the 'ssr' feature. Build with: cargo build --features ssr");
    std::process::exit(1);
}
