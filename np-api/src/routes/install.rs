//! Installation routes

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
use np_core::{InstallJob, InstallRequest, JobId, JobSummary, OutputLine};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

/// List all installation jobs
pub async fn list_jobs(State(state): State<AppState>) -> Result<Json<Vec<JobSummary>>, ApiError> {
    let jobs = state.jobs.list().await;
    Ok(Json(jobs))
}

/// Get a specific job
pub async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<InstallJob>, ApiError> {
    let job_id = JobId(id);
    let job = state.jobs.get(&job_id).await?;
    Ok(Json(job))
}

/// Create a new installation job
pub async fn create_job(
    State(state): State<AppState>,
    Json(request): Json<InstallRequest>,
) -> Result<(StatusCode, Json<InstallJob>), ApiError> {
    let job = state.jobs.create(request).await?;
    Ok((StatusCode::CREATED, Json(job)))
}

/// Start an installation job (non-WebSocket, just starts it)
pub async fn start_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let job_id = JobId(id);

    // Create a dummy channel since we're not streaming here
    let (tx, _rx) = tokio::sync::mpsc::channel(1);

    // Clone state for the background task
    let jobs = state.jobs.clone();

    // Start the job in the background
    tokio::spawn(async move {
        let _ = jobs.start(&job_id, tx).await;
    });

    Ok(StatusCode::ACCEPTED)
}

/// Cancel an installation job
pub async fn cancel_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let job_id = JobId(id);
    state.jobs.cancel(&job_id).await?;
    Ok(StatusCode::OK)
}

/// Delete a completed job
pub async fn delete_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let job_id = JobId(id);
    state.jobs.delete(&job_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// WebSocket handler for job output streaming
pub async fn ws_job_output(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_job_output(socket, state, id))
}

/// Message sent over job WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum JobWsMessage {
    /// Output line from command
    Output(OutputLine),
    /// Job phase update
    Phase { phase: String, progress: u8 },
    /// Job completed
    Completed { exit_code: i32 },
    /// Error occurred
    Error { message: String },
}

async fn handle_job_output(socket: WebSocket, state: AppState, id: String) {
    let (mut sender, _receiver) = socket.split();

    let job_id = JobId(id);

    // Get the job and subscribe to output
    let (job, mut rx) = match state.jobs.subscribe_output(&job_id).await {
        Ok((job, rx)) => (job, rx),
        Err(e) => {
            let msg = JobWsMessage::Error {
                message: e.to_string(),
            };
            let _ = sender
                .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                .await;
            return;
        }
    };

    // Send initial job state
    let phase_msg = JobWsMessage::Phase {
        phase: format!("{:?}", job.phase),
        progress: job.progress_percent,
    };
    let _ = sender
        .send(Message::Text(serde_json::to_string(&phase_msg).unwrap().into()))
        .await;

    // Stream output to WebSocket
    while let Some(line) = rx.recv().await {
        let msg = JobWsMessage::Output(line);
        if sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await
            .is_err()
        {
            break;
        }
    }

    // Get final job state
    if let Ok(final_job) = state.jobs.get(&job_id).await {
        let msg = if let Some(exit_code) = final_job.exit_code {
            JobWsMessage::Completed { exit_code }
        } else if let Some(error) = final_job.error {
            JobWsMessage::Error { message: error }
        } else {
            JobWsMessage::Phase {
                phase: format!("{:?}", final_job.phase),
                progress: final_job.progress_percent,
            }
        };
        let _ = sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await;
    }
}

/// WebSocket handler for starting and streaming a job
pub async fn ws_start_job(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_start_job(socket, state, id))
}

async fn handle_start_job(socket: WebSocket, state: AppState, id: String) {
    let (mut sender, _receiver) = socket.split();

    let job_id = JobId(id);

    // Create output channel
    let (tx, mut rx) = tokio::sync::mpsc::channel::<OutputLine>(100);

    // Start the job in the background
    let jobs = state.jobs.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        let _ = jobs.start(&job_id_clone, tx).await;
    });

    // Stream output to WebSocket
    while let Some(line) = rx.recv().await {
        let msg = JobWsMessage::Output(line);
        if sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await
            .is_err()
        {
            break;
        }
    }

    // Get final job state
    if let Ok(final_job) = state.jobs.get(&job_id).await {
        let msg = if let Some(exit_code) = final_job.exit_code {
            JobWsMessage::Completed { exit_code }
        } else if let Some(error) = final_job.error {
            JobWsMessage::Error { message: error }
        } else {
            JobWsMessage::Phase {
                phase: format!("{:?}", final_job.phase),
                progress: final_job.progress_percent,
            }
        };
        let _ = sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await;
    }
}

/// Response for VM test
#[derive(Debug, Serialize)]
pub struct VmTestResponse {
    pub job_id: String,
    pub message: String,
}

/// Start a VM test (dry run)
pub async fn start_vm_test(
    State(state): State<AppState>,
    Json(mut request): Json<InstallRequest>,
) -> Result<(StatusCode, Json<VmTestResponse>), ApiError> {
    // Set VM test mode
    request.vm_test = true;

    let job = state.jobs.create(request).await?;
    let job_id = job.id.clone();

    // Create a dummy channel since we're not streaming here
    let (tx, _rx) = tokio::sync::mpsc::channel(1);

    // Clone state for the background task
    let jobs = state.jobs.clone();
    let job_id_clone = job_id.clone();

    // Start the job in the background
    tokio::spawn(async move {
        let _ = jobs.start(&job_id_clone, tx).await;
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(VmTestResponse {
            job_id: job_id.to_string(),
            message: "VM test started".to_string(),
        }),
    ))
}
