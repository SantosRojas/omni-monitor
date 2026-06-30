#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tower_http::cors::{AllowOrigin, CorsLayer};
    use tower_http::services::{ServeDir, ServeFile};
    use tower_http::trace::TraceLayer;
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    dotenvy::dotenv().ok();

    let config = monitor::config::MonitorConfig::from_env();

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
        if origins.is_empty() {
            CorsLayer::permissive()
        } else {
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION])
        }
    };

    let app = axum::Router::new()
        .nest("/api", api_router)
        .fallback_service(ServeDir::new("static").fallback(ServeFile::new("static/index.html")))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::new(config.monitor_host.parse().expect("Invalid host"), config.monitor_port);

    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server failed");
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
