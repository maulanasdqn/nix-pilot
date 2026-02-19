//! Flake management routes

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use np_core::{
    CreateFlakeRequest, FlakeId, FlakeMetadata, FlakeOutputs, RegisteredFlake, UpdateFlakeRequest,
    UpdateInputRequest,
};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

/// List all registered flakes
pub async fn list_flakes(State(state): State<AppState>) -> Result<Json<Vec<RegisteredFlake>>, ApiError> {
    let flakes = state.flakes.list().await?;
    Ok(Json(flakes))
}

/// Get a specific registered flake
pub async fn get_flake(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RegisteredFlake>, ApiError> {
    let flake_id = FlakeId(id);
    let flake = state.flakes.get(&flake_id).await?;
    Ok(Json(flake))
}

/// Register a new flake
pub async fn create_flake(
    State(state): State<AppState>,
    Json(request): Json<CreateFlakeRequest>,
) -> Result<(StatusCode, Json<RegisteredFlake>), ApiError> {
    let flake = state.flakes.create(request).await?;
    Ok((StatusCode::CREATED, Json(flake)))
}

/// Update a registered flake
pub async fn update_flake(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateFlakeRequest>,
) -> Result<Json<RegisteredFlake>, ApiError> {
    let flake_id = FlakeId(id);
    let flake = state.flakes.update(&flake_id, request).await?;
    Ok(Json(flake))
}

/// Delete a registered flake
pub async fn delete_flake(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let flake_id = FlakeId(id);
    state.flakes.delete(&flake_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Refresh metadata for a registered flake
pub async fn refresh_flake_metadata(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RegisteredFlake>, ApiError> {
    let flake_id = FlakeId(id);
    let flake = state.flakes.refresh_metadata(&flake_id).await?;
    Ok(Json(flake))
}

/// Get flake outputs
pub async fn get_flake_outputs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<FlakeOutputs>, ApiError> {
    let flake_id = FlakeId(id);
    let outputs = state.flakes.get_outputs(&flake_id).await?;
    Ok(Json(outputs))
}

/// Request body for getting metadata by path
#[derive(Debug, Deserialize)]
pub struct MetadataByPathRequest {
    pub path: String,
}

/// Get metadata for any flake path (not necessarily registered)
pub async fn get_metadata_by_path(
    State(state): State<AppState>,
    Json(request): Json<MetadataByPathRequest>,
) -> Result<Json<FlakeMetadata>, ApiError> {
    let metadata = state.flakes.get_metadata(&request.path).await?;
    Ok(Json(metadata))
}

/// Update a flake input
pub async fn update_flake_input(
    State(state): State<AppState>,
    Path((id, input_name)): Path<(String, String)>,
    Json(request): Json<UpdateInputRequest>,
) -> Result<StatusCode, ApiError> {
    let flake_id = FlakeId(id);
    let flake = state.flakes.get(&flake_id).await?;
    state.flakes.update_input(&flake.path, &input_name, request).await?;
    Ok(StatusCode::OK)
}

/// Request body for updating the lock file
#[derive(Debug, Deserialize)]
pub struct UpdateLockRequest {
    /// Optional specific input to update (updates all if None)
    pub input: Option<String>,
}

/// Response for lock update job
#[derive(Debug, Serialize)]
pub struct UpdateLockResponse {
    pub job_id: String,
    pub message: String,
}

/// Update flake.lock (synchronous version - for WebSocket see ws routes)
pub async fn update_flake_lock(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateLockRequest>,
) -> Result<Json<UpdateLockResponse>, ApiError> {
    let flake_id = FlakeId(id.clone());
    let flake = state.flakes.get(&flake_id).await?;

    // Create a channel to collect output
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // Spawn the update task
    let flake_path = flake.path.clone();
    let input = request.input.clone();
    let flakes = state.flakes.clone();

    tokio::spawn(async move {
        let _ = flakes
            .update_lock(&flake_path, input.as_deref(), tx)
            .await;
    });

    // Collect some initial output (non-blocking)
    let mut output_lines = Vec::new();
    while let Ok(line) = rx.try_recv() {
        output_lines.push(line.content);
        if output_lines.len() >= 5 {
            break;
        }
    }

    Ok(Json(UpdateLockResponse {
        job_id: id,
        message: format!(
            "Lock update started{}",
            if request.input.is_some() {
                format!(" for input '{}'", request.input.unwrap())
            } else {
                String::new()
            }
        ),
    }))
}
