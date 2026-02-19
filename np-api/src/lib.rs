//! np-api - REST API server for nix-pilot
//!
//! This crate provides the HTTP API layer for nix-pilot,
//! including REST endpoints and WebSocket streaming.

pub mod app;
pub mod auth;
pub mod error;
pub mod routes;
pub mod state;

pub use app::create_router;
pub use auth::{AuthConfig, AuthState};
pub use error::{ApiError, ApiResult, ErrorCode, ErrorResponse};
pub use state::AppState;
