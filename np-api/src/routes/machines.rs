use axum::{
    extract::{Path, State},
    Json,
};
use np_core::{CreateMachineRequest, Machine, MachineId, UpdateMachineRequest};
use serde::Serialize;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct MachineListResponse {
    pub machines: Vec<Machine>,
}

#[derive(Serialize)]
pub struct MachineResponse {
    pub machine: Machine,
}

/// List all machines
pub async fn list_machines(State(state): State<AppState>) -> ApiResult<Json<MachineListResponse>> {
    let machines = state.machines.list().await;
    Ok(Json(MachineListResponse { machines }))
}

/// Get a single machine by ID
pub async fn get_machine(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<MachineResponse>> {
    let machine_id = MachineId::from_string(id);
    let machine = state
        .machines
        .get(&machine_id)
        .await
        .ok_or_else(|| ApiError::NotFound("Machine not found".to_string()))?;

    Ok(Json(MachineResponse { machine }))
}

/// Create a new machine
pub async fn create_machine(
    State(state): State<AppState>,
    Json(request): Json<CreateMachineRequest>,
) -> ApiResult<Json<MachineResponse>> {
    let machine = state.machines.create(request).await?;
    Ok(Json(MachineResponse { machine }))
}

/// Update an existing machine
pub async fn update_machine(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateMachineRequest>,
) -> ApiResult<Json<MachineResponse>> {
    let machine_id = MachineId::from_string(id);
    let machine = state.machines.update(&machine_id, request).await?;
    Ok(Json(MachineResponse { machine }))
}

/// Delete a machine
pub async fn delete_machine(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let machine_id = MachineId::from_string(id);
    state.machines.delete(&machine_id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// Test connection to a machine
pub async fn test_machine_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<MachineResponse>> {
    let machine_id = MachineId::from_string(id);
    let machine = state.machines.test_connection(&machine_id).await?;
    Ok(Json(MachineResponse { machine }))
}
