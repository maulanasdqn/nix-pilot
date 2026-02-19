//! Deployment routes

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::Response,
    Json,
};
use futures::{SinkExt, StreamExt};
use np_core::{
    DeployJob, DeployJobId, DeployJobSummary, DeployRequest, GenerationInfo, OutputLine,
    RollbackRequest, SshTarget,
};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

/// List all deployment jobs
pub async fn list_deploy_jobs(
    State(state): State<AppState>,
) -> Result<Json<Vec<DeployJobSummary>>, ApiError> {
    let jobs = state.deploys.list().await;
    Ok(Json(jobs))
}

/// Get a specific deployment job
pub async fn get_deploy_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeployJob>, ApiError> {
    let job_id = DeployJobId(id);
    let job = state.deploys.get(&job_id).await?;
    Ok(Json(job))
}

/// Create a new deployment job
pub async fn create_deploy_job(
    State(state): State<AppState>,
    Json(request): Json<DeployRequest>,
) -> Result<(StatusCode, Json<DeployJob>), ApiError> {
    let job = state.deploys.create(request).await?;
    Ok((StatusCode::CREATED, Json(job)))
}

/// Start a deployment job (non-WebSocket)
pub async fn start_deploy_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let job_id = DeployJobId(id);

    // Create a dummy channel
    let (tx, _rx) = tokio::sync::mpsc::channel(1);

    // Clone state for background task
    let deploys = state.deploys.clone();

    // Start in background
    tokio::spawn(async move {
        let _ = deploys.start(&job_id, tx).await;
    });

    Ok(StatusCode::ACCEPTED)
}

/// Cancel a deployment job
pub async fn cancel_deploy_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let job_id = DeployJobId(id);
    state.deploys.cancel(&job_id).await?;
    Ok(StatusCode::OK)
}

/// Delete a deployment job
pub async fn delete_deploy_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let job_id = DeployJobId(id);
    state.deploys.delete(&job_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// WebSocket handler for deployment output streaming
pub async fn ws_deploy_job_output(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_deploy_output(socket, state, id))
}

/// Message sent over deployment WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DeployWsMessage {
    /// Output line
    Output(OutputLine),
    /// Phase update
    Phase { phase: String, progress: u8 },
    /// Job completed
    Completed { exit_code: i32 },
    /// Error occurred
    Error { message: String },
}

async fn handle_deploy_output(socket: WebSocket, state: AppState, id: String) {
    let (mut sender, _receiver) = socket.split();

    let job_id = DeployJobId(id);

    // Get the job and subscribe to output
    let (job, mut rx) = match state.deploys.subscribe_output(&job_id).await {
        Ok((job, rx)) => (job, rx),
        Err(e) => {
            let msg = DeployWsMessage::Error {
                message: e.to_string(),
            };
            let _ = sender
                .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                .await;
            return;
        }
    };

    // Send initial job state
    let phase_msg = DeployWsMessage::Phase {
        phase: format!("{:?}", job.phase),
        progress: job.progress_percent,
    };
    let _ = sender
        .send(Message::Text(serde_json::to_string(&phase_msg).unwrap().into()))
        .await;

    // Stream output
    while let Some(line) = rx.recv().await {
        let msg = DeployWsMessage::Output(line);
        if sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await
            .is_err()
        {
            break;
        }
    }

    // Get final state
    if let Ok(final_job) = state.deploys.get(&job_id).await {
        let msg = if let Some(exit_code) = final_job.exit_code {
            DeployWsMessage::Completed { exit_code }
        } else if let Some(error) = final_job.error {
            DeployWsMessage::Error { message: error }
        } else {
            DeployWsMessage::Phase {
                phase: format!("{:?}", final_job.phase),
                progress: final_job.progress_percent,
            }
        };
        let _ = sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await;
    }
}

/// WebSocket handler for starting and streaming deployment
pub async fn ws_start_deploy_job(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_start_deploy(socket, state, id))
}

async fn handle_start_deploy(socket: WebSocket, state: AppState, id: String) {
    let (mut sender, _receiver) = socket.split();

    let job_id = DeployJobId(id);

    // Create output channel
    let (tx, mut rx) = tokio::sync::mpsc::channel::<OutputLine>(100);

    // Start the job in background
    let deploys = state.deploys.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        let _ = deploys.start(&job_id_clone, tx).await;
    });

    // Stream output
    while let Some(line) = rx.recv().await {
        let msg = DeployWsMessage::Output(line);
        if sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await
            .is_err()
        {
            break;
        }
    }

    // Get final state
    if let Ok(final_job) = state.deploys.get(&job_id).await {
        let msg = if let Some(exit_code) = final_job.exit_code {
            DeployWsMessage::Completed { exit_code }
        } else if let Some(error) = final_job.error {
            DeployWsMessage::Error { message: error }
        } else {
            DeployWsMessage::Phase {
                phase: format!("{:?}", final_job.phase),
                progress: final_job.progress_percent,
            }
        };
        let _ = sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await;
    }
}

/// Request to list generations
#[derive(Debug, Deserialize)]
pub struct ListGenerationsRequest {
    pub target: SshTarget,
}

/// List generations on a target
pub async fn list_generations(
    State(state): State<AppState>,
    Json(request): Json<ListGenerationsRequest>,
) -> Result<Json<Vec<GenerationInfo>>, ApiError> {
    let generations = state.deploys.list_generations(&request.target).await?;
    Ok(Json(generations))
}

/// Rollback response
#[derive(Debug, Serialize)]
pub struct RollbackResponse {
    pub success: bool,
    pub exit_code: i32,
    pub message: String,
}

/// Perform a rollback
pub async fn rollback(
    State(state): State<AppState>,
    Json(request): Json<RollbackRequest>,
) -> Result<Json<RollbackResponse>, ApiError> {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);

    let exit_code = state.deploys.rollback(request.clone(), tx).await?;

    let response = RollbackResponse {
        success: exit_code == 0,
        exit_code,
        message: if exit_code == 0 {
            format!(
                "Successfully rolled back to generation {}",
                request.generation.map(|g| g.to_string()).unwrap_or("previous".to_string())
            )
        } else {
            "Rollback failed".to_string()
        },
    };

    Ok(Json(response))
}

/// WebSocket handler for rollback with streaming output
pub async fn ws_rollback(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_rollback(socket, state))
}

async fn handle_rollback(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for rollback request
    let request: Option<RollbackRequest> = if let Some(Ok(Message::Text(text))) = receiver.next().await {
        serde_json::from_str(&text).ok()
    } else {
        None
    };

    let request = match request {
        Some(r) => r,
        None => {
            let msg = DeployWsMessage::Error {
                message: "Missing rollback request".to_string(),
            };
            let _ = sender
                .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                .await;
            return;
        }
    };

    // Create output channel
    let (tx, mut rx) = tokio::sync::mpsc::channel::<OutputLine>(100);

    // Spawn rollback task
    let deploys = state.deploys.clone();
    let rollback_handle = tokio::spawn(async move {
        deploys.rollback(request, tx).await
    });

    // Stream output
    while let Some(line) = rx.recv().await {
        let msg = DeployWsMessage::Output(line);
        if sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await
            .is_err()
        {
            break;
        }
    }

    // Get result
    let result = rollback_handle.await;
    let msg = match result {
        Ok(Ok(exit_code)) => DeployWsMessage::Completed { exit_code },
        Ok(Err(e)) => DeployWsMessage::Error {
            message: e.to_string(),
        },
        Err(e) => DeployWsMessage::Error {
            message: e.to_string(),
        },
    };

    let _ = sender
        .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
        .await;
}
