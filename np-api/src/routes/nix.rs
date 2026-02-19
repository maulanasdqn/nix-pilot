use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
    Json,
};
use futures::{SinkExt, StreamExt};
use np_core::{ClosureInfo, OutputLine, PathInfo, SearchResult, StoreInfo};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

use super::websocket::WsMessage;

#[derive(Deserialize)]
pub struct FlakeMetadataRequest {
    pub flake_ref: String,
}

#[derive(Serialize)]
pub struct FlakeMetadataResponse {
    pub metadata: serde_json::Value,
}

/// Get flake metadata
pub async fn get_flake_metadata(
    State(state): State<AppState>,
    Json(req): Json<FlakeMetadataRequest>,
) -> ApiResult<Json<FlakeMetadataResponse>> {
    let metadata = state.nix.flake_metadata(&req.flake_ref).await?;

    Ok(Json(FlakeMetadataResponse { metadata }))
}

#[derive(Serialize)]
pub struct FlakeShowResponse {
    pub outputs: serde_json::Value,
}

/// Get flake outputs
pub async fn get_flake_show(
    State(state): State<AppState>,
    Path(flake_ref): Path<String>,
) -> ApiResult<Json<FlakeShowResponse>> {
    // URL decode the flake ref
    let flake_ref = urlencoding::decode(&flake_ref)
        .map_err(|_| ApiError::BadRequest("Invalid flake reference".to_string()))?;

    let outputs = state.nix.flake_show(&flake_ref).await?;

    Ok(Json(FlakeShowResponse { outputs }))
}

#[derive(Deserialize)]
pub struct EvalRequest {
    pub expr: String,
}

#[derive(Serialize)]
pub struct EvalResponse {
    pub result: serde_json::Value,
}

/// Evaluate a nix expression
pub async fn eval_expression(
    State(state): State<AppState>,
    Json(req): Json<EvalRequest>,
) -> ApiResult<Json<EvalResponse>> {
    let result = state.nix.eval(&req.expr).await?;

    Ok(Json(EvalResponse { result }))
}

// ============================================================================
// Store Operations
// ============================================================================

#[derive(Serialize)]
pub struct StoreInfoResponse {
    pub info: StoreInfo,
}

/// Get nix store info
pub async fn get_store_info(State(state): State<AppState>) -> ApiResult<Json<StoreInfoResponse>> {
    let info = state.nix.store_info().await?;
    Ok(Json(StoreInfoResponse { info }))
}

#[derive(Deserialize)]
pub struct PathInfoRequest {
    pub path: String,
}

#[derive(Serialize)]
pub struct PathInfoResponse {
    pub info: PathInfo,
}

/// Get info about a store path
pub async fn get_path_info(
    State(state): State<AppState>,
    Json(req): Json<PathInfoRequest>,
) -> ApiResult<Json<PathInfoResponse>> {
    let info = state.nix.path_info(&req.path).await?;
    Ok(Json(PathInfoResponse { info }))
}

#[derive(Deserialize)]
pub struct ClosureSizeRequest {
    pub path: String,
}

#[derive(Serialize)]
pub struct ClosureSizeResponse {
    pub info: ClosureInfo,
}

/// Get closure size of a path
pub async fn get_closure_size(
    State(state): State<AppState>,
    Json(req): Json<ClosureSizeRequest>,
) -> ApiResult<Json<ClosureSizeResponse>> {
    let info = state.nix.closure_size(&req.path).await?;
    Ok(Json(ClosureSizeResponse { info }))
}

// ============================================================================
// Package Search
// ============================================================================

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default)]
    pub flake: Option<String>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

/// Search for packages
pub async fn search_packages(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> ApiResult<Json<SearchResponse>> {
    let results = state.nix.search(&req.query, req.flake.as_deref()).await?;
    Ok(Json(SearchResponse { results }))
}

// ============================================================================
// Dependency Analysis
// ============================================================================

#[derive(Deserialize)]
pub struct WhyDependsRequest {
    pub package: String,
    pub dependency: String,
}

#[derive(Serialize)]
pub struct WhyDependsResponse {
    pub chain: Vec<String>,
}

/// Show why one package depends on another
pub async fn why_depends(
    State(state): State<AppState>,
    Json(req): Json<WhyDependsRequest>,
) -> ApiResult<Json<WhyDependsResponse>> {
    let chain = state.nix.why_depends(&req.package, &req.dependency).await?;
    Ok(Json(WhyDependsResponse { chain }))
}

// ============================================================================
// Derivation Operations
// ============================================================================

#[derive(Deserialize)]
pub struct DerivationShowRequest {
    pub path: String,
}

#[derive(Serialize)]
pub struct DerivationShowResponse {
    pub derivation: serde_json::Value,
}

/// Show derivation details
pub async fn show_derivation(
    State(state): State<AppState>,
    Json(req): Json<DerivationShowRequest>,
) -> ApiResult<Json<DerivationShowResponse>> {
    let derivation = state.nix.derivation_show(&req.path).await?;
    Ok(Json(DerivationShowResponse { derivation }))
}

// ============================================================================
// WebSocket Operations
// ============================================================================

/// WebSocket handler for flake check with streaming output
pub async fn ws_flake_check(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_flake_check(socket, state))
}

#[derive(Debug, Deserialize)]
pub struct FlakeCheckRequest {
    pub flake_ref: String,
}

async fn handle_flake_check(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for the request
    let request: Option<FlakeCheckRequest> =
        if let Some(Ok(Message::Text(text))) = receiver.next().await {
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
            let _ = sender
                .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                .await;
            return;
        }
    };

    // Create channel for output
    let (tx, mut rx) = mpsc::channel::<OutputLine>(100);

    // Send started message
    let started_msg = WsMessage::Started {
        command: format!("nix flake check {}", flake_ref),
    };
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&started_msg).unwrap().into(),
        ))
        .await;

    // Spawn the check task
    let nix = state.nix.clone();
    let check_handle = tokio::spawn(async move { nix.flake_check(&flake_ref, tx).await });

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
    let exit_code =
        check_handle
            .await
            .unwrap_or(Err(np_core::NpError::Other("Task cancelled".into())));
    let completed_msg = match exit_code {
        Ok(code) => WsMessage::Completed { exit_code: code },
        Err(e) => WsMessage::Error {
            message: e.to_string(),
        },
    };
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&completed_msg).unwrap().into(),
        ))
        .await;
}

/// WebSocket handler for store optimise
pub async fn ws_store_optimise(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_store_optimise(socket, state))
}

async fn handle_store_optimise(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for start signal (can be empty object)
    let _ = receiver.next().await;

    // Create channel for output
    let (tx, mut rx) = mpsc::channel::<OutputLine>(100);

    // Send started message
    let started_msg = WsMessage::Started {
        command: "nix store optimise".to_string(),
    };
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&started_msg).unwrap().into(),
        ))
        .await;

    // Spawn the optimise task
    let nix = state.nix.clone();
    let optimise_handle = tokio::spawn(async move { nix.store_optimise(tx).await });

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
    let exit_code =
        optimise_handle
            .await
            .unwrap_or(Err(np_core::NpError::Other("Task cancelled".into())));
    let completed_msg = match exit_code {
        Ok(code) => WsMessage::Completed { exit_code: code },
        Err(e) => WsMessage::Error {
            message: e.to_string(),
        },
    };
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&completed_msg).unwrap().into(),
        ))
        .await;
}

/// WebSocket handler for store verify
pub async fn ws_store_verify(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_store_verify(socket, state))
}

#[derive(Debug, Deserialize)]
pub struct StoreVerifyRequest {
    #[serde(default)]
    pub paths: Option<Vec<String>>,
    #[serde(default)]
    pub check_contents: bool,
}

async fn handle_store_verify(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for the request
    let request: StoreVerifyRequest =
        if let Some(Ok(Message::Text(text))) = receiver.next().await {
            serde_json::from_str(&text).unwrap_or(StoreVerifyRequest {
                paths: None,
                check_contents: false,
            })
        } else {
            StoreVerifyRequest {
                paths: None,
                check_contents: false,
            }
        };

    // Create channel for output
    let (tx, mut rx) = mpsc::channel::<OutputLine>(100);

    // Send started message
    let started_msg = WsMessage::Started {
        command: "nix store verify".to_string(),
    };
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&started_msg).unwrap().into(),
        ))
        .await;

    // Spawn the verify task
    let nix = state.nix.clone();
    let paths = request.paths.clone();
    let check_contents = request.check_contents;
    let verify_handle = tokio::spawn(async move {
        let path_refs: Option<Vec<&str>> =
            paths.as_ref().map(|p| p.iter().map(|s| s.as_str()).collect());
        nix.store_verify(path_refs.as_deref(), check_contents, tx)
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
    let exit_code =
        verify_handle
            .await
            .unwrap_or(Err(np_core::NpError::Other("Task cancelled".into())));
    let completed_msg = match exit_code {
        Ok(code) => WsMessage::Completed { exit_code: code },
        Err(e) => WsMessage::Error {
            message: e.to_string(),
        },
    };
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&completed_msg).unwrap().into(),
        ))
        .await;
}

/// WebSocket handler for store repair
pub async fn ws_store_repair(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_store_repair(socket, state))
}

#[derive(Debug, Deserialize)]
pub struct StoreRepairRequest {
    pub path: String,
}

async fn handle_store_repair(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for the request
    let request: Option<StoreRepairRequest> =
        if let Some(Ok(Message::Text(text))) = receiver.next().await {
            serde_json::from_str(&text).ok()
        } else {
            None
        };

    let path = match request {
        Some(r) => r.path,
        None => {
            let msg = WsMessage::Error {
                message: "Missing path".into(),
            };
            let _ = sender
                .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                .await;
            return;
        }
    };

    // Create channel for output
    let (tx, mut rx) = mpsc::channel::<OutputLine>(100);

    // Send started message
    let started_msg = WsMessage::Started {
        command: format!("nix store repair {}", path),
    };
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&started_msg).unwrap().into(),
        ))
        .await;

    // Spawn the repair task
    let nix = state.nix.clone();
    let repair_handle = tokio::spawn(async move { nix.store_repair(&path, tx).await });

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
    let exit_code =
        repair_handle
            .await
            .unwrap_or(Err(np_core::NpError::Other("Task cancelled".into())));
    let completed_msg = match exit_code {
        Ok(code) => WsMessage::Completed { exit_code: code },
        Err(e) => WsMessage::Error {
            message: e.to_string(),
        },
    };
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&completed_msg).unwrap().into(),
        ))
        .await;
}
