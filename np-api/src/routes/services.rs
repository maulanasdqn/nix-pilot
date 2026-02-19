//! Service management routes

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::Response,
    Json,
};
use futures::{SinkExt, StreamExt};
use np_core::{
    LogOptions, MachineId, OutputLine, ServiceAction, ServiceActionRequest, ServiceActionResult,
    ServiceInfo, ServiceStatus, SystemdManager,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// Response for service list
#[derive(Serialize)]
pub struct ServiceListResponse {
    pub services: Vec<ServiceInfo>,
    pub total: usize,
}

/// Response for single service
#[derive(Serialize)]
pub struct ServiceResponse {
    pub service: ServiceStatus,
}

/// Response for service action
#[derive(Serialize)]
pub struct ServiceActionResponse {
    pub result: ServiceActionResult,
}

/// Request for service action
#[derive(Debug, Deserialize)]
pub struct ServiceActionBody {
    pub action: ServiceAction,
}

/// Query params for log fetching
#[derive(Debug, Deserialize)]
pub struct LogQueryParams {
    /// Number of lines to fetch (default: 100)
    #[serde(default)]
    pub lines: Option<u32>,
    /// Fetch logs since (ISO 8601 timestamp)
    #[serde(default)]
    pub since: Option<String>,
    /// Fetch logs until (ISO 8601 timestamp)
    #[serde(default)]
    pub until: Option<String>,
    /// Show full messages (no truncation)
    #[serde(default)]
    pub full: Option<bool>,
}

/// Log response
#[derive(Serialize)]
pub struct LogResponse {
    pub logs: Vec<String>,
}

/// List all services on a machine
pub async fn list_services(
    State(state): State<AppState>,
    Path(machine_id): Path<String>,
) -> ApiResult<Json<ServiceListResponse>> {
    let machine_id = MachineId::from_string(machine_id);
    let machine = state
        .machines
        .get(&machine_id)
        .await
        .ok_or_else(|| ApiError::NotFound("Machine not found".to_string()))?;

    let manager = SystemdManager::new();
    let services = manager.list_services(&machine.target).await?;
    let total = services.len();

    Ok(Json(ServiceListResponse { services, total }))
}

/// Get failed services on a machine
pub async fn list_failed_services(
    State(state): State<AppState>,
    Path(machine_id): Path<String>,
) -> ApiResult<Json<ServiceListResponse>> {
    let machine_id = MachineId::from_string(machine_id);
    let machine = state
        .machines
        .get(&machine_id)
        .await
        .ok_or_else(|| ApiError::NotFound("Machine not found".to_string()))?;

    let manager = SystemdManager::new();
    let services = manager.get_failed_services(&machine.target).await?;
    let total = services.len();

    Ok(Json(ServiceListResponse { services, total }))
}

/// Get detailed status of a specific service
pub async fn get_service(
    State(state): State<AppState>,
    Path((machine_id, service_name)): Path<(String, String)>,
) -> ApiResult<Json<ServiceResponse>> {
    let machine_id = MachineId::from_string(machine_id);
    let machine = state
        .machines
        .get(&machine_id)
        .await
        .ok_or_else(|| ApiError::NotFound("Machine not found".to_string()))?;

    let manager = SystemdManager::new();
    let service = manager.get_service_status(&machine.target, &service_name).await?;

    Ok(Json(ServiceResponse { service }))
}

/// Perform an action on a service (start, stop, restart, etc.)
pub async fn service_action(
    State(state): State<AppState>,
    Path((machine_id, service_name)): Path<(String, String)>,
    Json(body): Json<ServiceActionBody>,
) -> ApiResult<Json<ServiceActionResponse>> {
    let machine_id = MachineId::from_string(machine_id);
    let machine = state
        .machines
        .get(&machine_id)
        .await
        .ok_or_else(|| ApiError::NotFound("Machine not found".to_string()))?;

    let request = ServiceActionRequest {
        service: service_name,
        action: body.action,
    };

    let manager = SystemdManager::new();
    let result = manager.service_action(&machine.target, &request).await?;

    Ok(Json(ServiceActionResponse { result }))
}

/// Get logs for a service
pub async fn get_service_logs(
    State(state): State<AppState>,
    Path((machine_id, service_name)): Path<(String, String)>,
    Query(params): Query<LogQueryParams>,
) -> ApiResult<Json<LogResponse>> {
    let machine_id = MachineId::from_string(machine_id);
    let machine = state
        .machines
        .get(&machine_id)
        .await
        .ok_or_else(|| ApiError::NotFound("Machine not found".to_string()))?;

    let options = LogOptions {
        lines: params.lines,
        since: params.since.as_deref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
        until: params.until.as_deref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
        full: params.full.unwrap_or(false),
        ..Default::default()
    };

    let manager = SystemdManager::new();
    let logs = manager.get_logs(&machine.target, &service_name, &options).await?;

    Ok(Json(LogResponse { logs }))
}

/// Get system-wide logs for a machine
pub async fn get_system_logs(
    State(state): State<AppState>,
    Path(machine_id): Path<String>,
    Query(params): Query<LogQueryParams>,
) -> ApiResult<Json<LogResponse>> {
    let machine_id = MachineId::from_string(machine_id);
    let machine = state
        .machines
        .get(&machine_id)
        .await
        .ok_or_else(|| ApiError::NotFound("Machine not found".to_string()))?;

    let options = LogOptions {
        lines: params.lines,
        since: params.since.as_deref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
        until: params.until.as_deref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
        full: params.full.unwrap_or(false),
        ..Default::default()
    };

    let manager = SystemdManager::new();
    let logs = manager.get_system_logs(&machine.target, &options).await?;

    Ok(Json(LogResponse { logs }))
}

/// WebSocket message for log streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum LogWsMessage {
    /// Log line received
    Log(OutputLine),
    /// Stream started
    Started { service: String },
    /// Error occurred
    Error { message: String },
    /// Stream ended (connection closed)
    Ended,
}

/// Request to start log streaming
#[derive(Debug, Deserialize)]
pub struct LogStreamRequest {
    /// Number of lines to show initially (default: 50)
    #[serde(default)]
    pub lines: Option<u32>,
}

/// WebSocket handler for streaming service logs
pub async fn ws_stream_service_logs(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path((machine_id, service_name)): Path<(String, String)>,
) -> Response {
    ws.on_upgrade(move |socket| handle_log_stream(socket, state, machine_id, service_name))
}

async fn handle_log_stream(
    socket: WebSocket,
    state: AppState,
    machine_id: String,
    service_name: String,
) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for optional configuration
    let request: Option<LogStreamRequest> = if let Some(Ok(Message::Text(text))) = receiver.next().await {
        serde_json::from_str(&text).ok()
    } else {
        None
    };

    // Get the machine
    let machine_id = MachineId::from_string(machine_id);
    let machine = match state.machines.get(&machine_id).await {
        Some(m) => m,
        None => {
            let msg = LogWsMessage::Error {
                message: "Machine not found".to_string(),
            };
            let _ = sender.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await;
            return;
        }
    };

    // Create channel for output
    let (tx, mut rx) = mpsc::channel::<OutputLine>(100);

    // Send started message
    let started_msg = LogWsMessage::Started {
        service: service_name.clone(),
    };
    let _ = sender
        .send(Message::Text(serde_json::to_string(&started_msg).unwrap().into()))
        .await;

    // Build log options
    let options = LogOptions {
        lines: request.and_then(|r| r.lines),
        follow: true,
        ..Default::default()
    };

    // Spawn the streaming task
    let manager = SystemdManager::new();
    let target = machine.target.clone();
    let service = service_name.clone();
    let stream_handle = tokio::spawn(async move {
        manager.stream_logs(&target, &service, &options, tx).await
    });

    // Stream logs to WebSocket
    loop {
        tokio::select! {
            // Receive log line
            Some(line) = rx.recv() => {
                let msg = LogWsMessage::Log(line);
                if sender
                    .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            // Check for WebSocket close
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // Cancel the streaming task
    stream_handle.abort();

    // Send ended message (best effort)
    let ended_msg = LogWsMessage::Ended;
    let _ = sender
        .send(Message::Text(serde_json::to_string(&ended_msg).unwrap().into()))
        .await;
}

/// WebSocket handler for streaming system logs
pub async fn ws_stream_system_logs(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(machine_id): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_system_log_stream(socket, state, machine_id))
}

async fn handle_system_log_stream(socket: WebSocket, state: AppState, machine_id: String) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for optional configuration
    let request: Option<LogStreamRequest> = if let Some(Ok(Message::Text(text))) = receiver.next().await {
        serde_json::from_str(&text).ok()
    } else {
        None
    };

    // Get the machine
    let machine_id = MachineId::from_string(machine_id);
    let machine = match state.machines.get(&machine_id).await {
        Some(m) => m,
        None => {
            let msg = LogWsMessage::Error {
                message: "Machine not found".to_string(),
            };
            let _ = sender.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await;
            return;
        }
    };

    // Create channel for output
    let (tx, mut rx) = mpsc::channel::<OutputLine>(100);

    // Send started message
    let started_msg = LogWsMessage::Started {
        service: "system".to_string(),
    };
    let _ = sender
        .send(Message::Text(serde_json::to_string(&started_msg).unwrap().into()))
        .await;

    // Build log options
    let options = LogOptions {
        lines: request.and_then(|r| r.lines),
        follow: true,
        ..Default::default()
    };

    // Spawn the streaming task
    // Note: For system logs, we'd need a stream_system_logs method
    // For now, we'll stream all services using journalctl -f
    let manager = SystemdManager::new();
    let target = machine.target.clone();
    let stream_handle = tokio::spawn(async move {
        // Stream all system logs
        manager.stream_logs(&target, "*", &options, tx).await
    });

    // Stream logs to WebSocket
    loop {
        tokio::select! {
            // Receive log line
            Some(line) = rx.recv() => {
                let msg = LogWsMessage::Log(line);
                if sender
                    .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            // Check for WebSocket close
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // Cancel the streaming task
    stream_handle.abort();

    // Send ended message (best effort)
    let ended_msg = LogWsMessage::Ended;
    let _ = sender
        .send(Message::Text(serde_json::to_string(&ended_msg).unwrap().into()))
        .await;
}
