use axum::{extract::State, Json};
use serde::Serialize;

use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub nix_version: Option<String>,
}

/// Health check endpoint
pub async fn health_check(State(state): State<AppState>) -> ApiResult<Json<HealthResponse>> {
    let nix_version = state.nix.check_available().await.ok();

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        nix_version,
    }))
}
