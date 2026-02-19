//! WebSocket routes for streaming output

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use np_core::{FlakeId, OutputLine};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::state::AppState;

/// Message sent over WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// Output line from command
    Output(OutputLine),
    /// Command started
    Started { command: String },
    /// Command completed
    Completed { exit_code: i32 },
    /// Error occurred
    Error { message: String },
}

/// Request to update flake lock
#[derive(Debug, Deserialize)]
pub struct UpdateLockWsRequest {
    /// Specific input to update (or all if None)
    pub input: Option<String>,
}

/// WebSocket handler for flake lock update with streaming output
pub async fn ws_flake_lock_update(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_lock_update(socket, state, id))
}

async fn handle_lock_update(socket: WebSocket, state: AppState, id: String) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for the update request
    let request: Option<UpdateLockWsRequest> = if let Some(Ok(Message::Text(text))) = receiver.next().await {
        serde_json::from_str(&text).ok()
    } else {
        None
    };

    // Get the flake
    let flake_id = FlakeId(id);
    let flake = match state.flakes.get(&flake_id).await {
        Ok(f) => f,
        Err(e) => {
            let msg = WsMessage::Error {
                message: e.to_string(),
            };
            let _ = sender.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await;
            return;
        }
    };

    // Create channel for output
    let (tx, mut rx) = mpsc::channel::<OutputLine>(100);

    // Send started message
    let input_desc = request
        .as_ref()
        .and_then(|r| r.input.clone())
        .map(|i| format!(" for '{}'", i))
        .unwrap_or_default();
    let started_msg = WsMessage::Started {
        command: format!("nix flake update{}", input_desc),
    };
    let _ = sender
        .send(Message::Text(serde_json::to_string(&started_msg).unwrap().into()))
        .await;

    // Spawn the update task
    let flake_path = flake.path.clone();
    let input = request.and_then(|r| r.input);
    let flakes = state.flakes.clone();

    let update_handle = tokio::spawn(async move {
        flakes
            .update_lock(&flake_path, input.as_deref(), tx)
            .await
    });

    // Stream output to WebSocket
    while let Some(line) = rx.recv().await {
        let msg = WsMessage::Output(line);
        if sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await
            .is_err()
        {
            break;
        }
    }

    // Send completion message
    let exit_code = update_handle.await.unwrap_or(Err(np_core::NpError::Other("Task cancelled".into())));
    let completed_msg = match exit_code {
        Ok(code) => WsMessage::Completed { exit_code: code },
        Err(e) => WsMessage::Error {
            message: e.to_string(),
        },
    };
    let _ = sender
        .send(Message::Text(serde_json::to_string(&completed_msg).unwrap().into()))
        .await;
}

/// WebSocket handler for nix build with streaming output
pub async fn ws_nix_build(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_nix_build(socket, state))
}

/// Build request
#[derive(Debug, Deserialize)]
pub struct BuildWsRequest {
    pub flake_ref: String,
}

async fn handle_nix_build(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for the build request
    let request: Option<BuildWsRequest> = if let Some(Ok(Message::Text(text))) = receiver.next().await {
        serde_json::from_str(&text).ok()
    } else {
        None
    };

    let flake_ref = match request {
        Some(r) => r.flake_ref,
        None => {
            let msg = WsMessage::Error {
                message: "Missing flake reference".into(),
            };
            let _ = sender.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await;
            return;
        }
    };

    // Create channel for output
    let (tx, mut rx) = mpsc::channel::<OutputLine>(100);

    // Send started message
    let started_msg = WsMessage::Started {
        command: format!("nix build {}", flake_ref),
    };
    let _ = sender
        .send(Message::Text(serde_json::to_string(&started_msg).unwrap().into()))
        .await;

    // Spawn the build task
    let nix = state.nix.clone();
    let build_handle = tokio::spawn(async move {
        nix.build(&flake_ref, tx).await
    });

    // Stream output to WebSocket
    while let Some(line) = rx.recv().await {
        let msg = WsMessage::Output(line);
        if sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await
            .is_err()
        {
            break;
        }
    }

    // Send completion message
    let exit_code = build_handle.await.unwrap_or(Err(np_core::NpError::Other("Task cancelled".into())));
    let completed_msg = match exit_code {
        Ok(code) => WsMessage::Completed { exit_code: code },
        Err(e) => WsMessage::Error {
            message: e.to_string(),
        },
    };
    let _ = sender
        .send(Message::Text(serde_json::to_string(&completed_msg).unwrap().into()))
        .await;
}

/// WebSocket handler for garbage collection with streaming output
pub async fn ws_nix_gc(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_nix_gc(socket, state))
}

/// GC request
#[derive(Debug, Deserialize)]
pub struct GcWsRequest {
    pub delete_older_than: Option<String>,
}

async fn handle_nix_gc(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for the GC request
    let request: Option<GcWsRequest> = if let Some(Ok(Message::Text(text))) = receiver.next().await {
        serde_json::from_str(&text).ok()
    } else {
        None
    };

    let delete_older_than = request.and_then(|r| r.delete_older_than);

    // Create channel for output
    let (tx, mut rx) = mpsc::channel::<OutputLine>(100);

    // Send started message
    let started_msg = WsMessage::Started {
        command: format!(
            "nix store gc{}",
            delete_older_than
                .as_ref()
                .map(|d| format!(" --delete-older-than {}", d))
                .unwrap_or_default()
        ),
    };
    let _ = sender
        .send(Message::Text(serde_json::to_string(&started_msg).unwrap().into()))
        .await;

    // Spawn the GC task
    let nix = state.nix.clone();
    let gc_handle = tokio::spawn(async move {
        nix.gc(delete_older_than.as_deref(), tx).await
    });

    // Stream output to WebSocket
    while let Some(line) = rx.recv().await {
        let msg = WsMessage::Output(line);
        if sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await
            .is_err()
        {
            break;
        }
    }

    // Send completion message
    let exit_code = gc_handle.await.unwrap_or(Err(np_core::NpError::Other("Task cancelled".into())));
    let completed_msg = match exit_code {
        Ok(code) => WsMessage::Completed { exit_code: code },
        Err(e) => WsMessage::Error {
            message: e.to_string(),
        },
    };
    let _ = sender
        .send(Message::Text(serde_json::to_string(&completed_msg).unwrap().into()))
        .await;
}
