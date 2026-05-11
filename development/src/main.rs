use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;

mod calendar;
mod config;
mod db;
mod error;
mod market_data;
mod models;
mod monitor;
mod risk;
mod routes;
mod scanner;
mod ta;
mod telegram;

pub use config::Config;
pub use db::Db;

/// Shared application state injected into every handler.
pub struct AppState {
    pub config: Config,
    pub db: Db,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present; ignore if missing (production uses real env vars).
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;

    init_tracing(&config.rust_log);

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "swingbot starting");

    let db = db::connect(&config.database_url).await?;

    let state = Arc::new(AppState { config, db });

    let app = routes::build(state.clone());

    let addr = format!("{}:{}", state.config.host, state.config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("server shut down cleanly");
    Ok(())
}

fn init_tracing(rust_log: &str) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(rust_log)),
        )
        .with(fmt::layer().json())
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c   => tracing::info!("received SIGINT"),
        _ = terminate => tracing::info!("received SIGTERM"),
    }
}
