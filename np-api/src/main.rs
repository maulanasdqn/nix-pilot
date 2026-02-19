use np_core::CoreConfig;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod error;
mod routes;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "np_api=debug,np_core=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = CoreConfig::default();
    if let Err(e) = config.ensure_dirs() {
        tracing::warn!("Failed to create data directories: {}", e);
    }

    // Create application state
    let state = AppState::new(config);

    // Initialize state (load data from disk)
    if let Err(e) = state.init().await {
        tracing::error!("Failed to initialize state: {}", e);
    }

    // Build router
    let app = app::create_router(state);

    // Read address and port from environment
    let address = std::env::var("NP_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("NP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    // Start server
    let addr: SocketAddr = format!("{}:{}", address, port).parse().unwrap();
    tracing::info!("Starting nix-pilot API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
