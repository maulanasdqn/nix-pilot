//! Authentication module for nix-pilot API
//!
//! Supports:
//! - Username/password login
//! - Session-based authentication with disk persistence
//! - API key authentication (optional)

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Session store type
pub type SessionStore = Arc<RwLock<HashSet<String>>>;

/// Authentication state
#[derive(Clone)]
pub struct AuthState {
    pub username: String,
    pub password_hash: String,
    pub sessions: SessionStore,
    pub enabled: bool,
    pub sessions_file: PathBuf,
}

impl AuthState {
    pub fn from_env() -> Self {
        let username = std::env::var("NP_AUTH_USERNAME").unwrap_or_else(|_| "admin".to_string());
        let password = std::env::var("NP_AUTH_PASSWORD").unwrap_or_else(|_| "changeme".to_string());
        let enabled = std::env::var("NP_AUTH_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true); // Enabled by default
        let data_dir = std::env::var("NP_DATA_DIR").unwrap_or_else(|_| "/var/lib/nix-pilot".to_string());

        // Simple hash using first 32 chars of sha256-like hash
        let password_hash = simple_hash(&password);

        let sessions_file = PathBuf::from(&data_dir).join("sessions.json");

        // Load existing sessions from disk
        let sessions = load_sessions(&sessions_file);

        Self {
            username,
            password_hash,
            sessions: Arc::new(RwLock::new(sessions)),
            enabled,
            sessions_file,
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        simple_hash(password) == self.password_hash
    }

    pub async fn create_session(&self) -> String {
        let token = uuid::Uuid::new_v4().to_string();
        {
            self.sessions.write().await.insert(token.clone());
        }
        self.persist_sessions().await;
        token
    }

    pub async fn validate_session(&self, token: &str) -> bool {
        self.sessions.read().await.contains(token)
    }

    pub async fn invalidate_session(&self, token: &str) {
        {
            self.sessions.write().await.remove(token);
        }
        self.persist_sessions().await;
    }

    async fn persist_sessions(&self) {
        let sessions = self.sessions.read().await;
        let sessions_vec: Vec<&String> = sessions.iter().collect();
        if let Ok(json) = serde_json::to_string(&sessions_vec) {
            let _ = tokio::fs::write(&self.sessions_file, json).await;
        }
    }
}

fn load_sessions(path: &PathBuf) -> HashSet<String> {
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(sessions) = serde_json::from_str::<Vec<String>>(&content) {
            return sessions.into_iter().collect();
        }
    }
    HashSet::new()
}

/// Simple hash function (not cryptographically secure, but works without extra deps)
fn simple_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    // Add salt
    "nix-pilot-salt".hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub message: String,
}

/// Auth check response
#[derive(Debug, Serialize)]
pub struct AuthCheckResponse {
    pub authenticated: bool,
    pub auth_enabled: bool,
}

/// Login handler
pub async fn login(
    State(state): State<crate::state::AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let auth = &state.auth;
    if !auth.enabled {
        return (
            StatusCode::OK,
            Json(LoginResponse {
                success: true,
                token: None,
                message: "Authentication disabled".to_string(),
            }),
        );
    }

    if req.username == auth.username && auth.verify_password(&req.password) {
        let token = auth.create_session().await;
        tracing::info!("User '{}' logged in successfully", req.username);
        (
            StatusCode::OK,
            Json(LoginResponse {
                success: true,
                token: Some(token),
                message: "Login successful".to_string(),
            }),
        )
    } else {
        tracing::warn!("Failed login attempt for user '{}'", req.username);
        (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                success: false,
                token: None,
                message: "Invalid username or password".to_string(),
            }),
        )
    }
}

/// Logout handler
pub async fn logout(State(state): State<crate::state::AppState>, req: Request) -> impl IntoResponse {
    let auth = &state.auth;
    if let Some(token) = extract_token(&req) {
        auth.invalidate_session(&token).await;
        tracing::info!("User logged out");
    }
    Json(serde_json::json!({"success": true, "message": "Logged out"}))
}

/// Check auth status
pub async fn check_auth(State(state): State<crate::state::AppState>, req: Request) -> impl IntoResponse {
    let auth = &state.auth;
    if !auth.enabled {
        return Json(AuthCheckResponse {
            authenticated: true,
            auth_enabled: false,
        });
    }

    let authenticated = if let Some(token) = extract_token(&req) {
        auth.validate_session(&token).await
    } else {
        false
    };

    Json(AuthCheckResponse {
        authenticated,
        auth_enabled: true,
    })
}

/// Extract token from request (header or cookie)
fn extract_token(req: &Request) -> Option<String> {
    // Try Authorization header first
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }

    // Try cookie
    if let Some(cookie_header) = req.headers().get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(token) = cookie.strip_prefix("np_token=") {
                    return Some(token.to_string());
                }
            }
        }
    }

    None
}

/// Auth middleware - protects API routes
pub async fn auth_middleware(
    State(state): State<crate::state::AppState>,
    req: Request,
    next: Next,
) -> Response {
    let auth = &state.auth;

    // If auth is disabled, allow all
    if !auth.enabled {
        return next.run(req).await;
    }

    let path = req.uri().path();

    // Allow public routes
    let public_routes = [
        "/api/auth/login",
        "/api/auth/check",
        "/api/health",
    ];

    if public_routes.iter().any(|r| path == *r) || !path.starts_with("/api/") {
        return next.run(req).await;
    }

    // Check for valid session token
    if let Some(token) = extract_token(&req) {
        if auth.validate_session(&token).await {
            return next.run(req).await;
        }
    }

    // Unauthorized
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": "Unauthorized",
            "code": "AUTH_REQUIRED",
            "message": "Please login to access this resource"
        })),
    )
        .into_response()
}
