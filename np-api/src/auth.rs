//! Authentication middleware for np-api
//!
//! Supports:
//! - API key authentication via X-API-Key header
//! - Bearer token authentication via Authorization header
//! - Optional authentication (can be disabled for local development)

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Authentication configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// Whether authentication is enabled
    #[serde(default)]
    pub enabled: bool,

    /// API keys that are allowed to access the API
    #[serde(default)]
    pub api_keys: Vec<String>,

    /// Secret for signing tokens (if using JWT in the future)
    #[serde(default)]
    pub secret: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_keys: Vec::new(),
            secret: None,
        }
    }
}

/// Authentication state shared across requests
#[derive(Clone)]
pub struct AuthState {
    config: Arc<AuthConfig>,
}

impl AuthState {
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Check if an API key is valid
    pub fn validate_api_key(&self, key: &str) -> bool {
        self.config.api_keys.iter().any(|k| k == key)
    }

    /// Check if authentication is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Authenticated user info (attached to request extensions)
#[derive(Debug, Clone, Serialize)]
pub struct AuthenticatedUser {
    /// Type of authentication used
    pub auth_type: AuthType,
    /// Identifier (API key name, user ID, etc.)
    pub identifier: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    ApiKey,
    Bearer,
    None,
}

/// Authentication middleware
///
/// If authentication is disabled, all requests are allowed.
/// If enabled, requests must have a valid API key or bearer token.
pub async fn auth_middleware(
    State(auth): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If auth is disabled, allow all requests
    if !auth.is_enabled() {
        request.extensions_mut().insert(AuthenticatedUser {
            auth_type: AuthType::None,
            identifier: None,
        });
        return Ok(next.run(request).await);
    }

    // Extract API key if present
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Extract Bearer token if present
    let bearer_token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // Try API key authentication first
    if let Some(api_key) = api_key {
        if auth.validate_api_key(&api_key) {
            request.extensions_mut().insert(AuthenticatedUser {
                auth_type: AuthType::ApiKey,
                identifier: Some(api_key),
            });
            return Ok(next.run(request).await);
        }
    }

    // Try Bearer token authentication
    if let Some(token) = bearer_token {
        // For now, just check if it's a valid API key
        // In the future, this could validate JWTs
        if auth.validate_api_key(&token) {
            request.extensions_mut().insert(AuthenticatedUser {
                auth_type: AuthType::Bearer,
                identifier: Some(token),
            });
            return Ok(next.run(request).await);
        }
    }

    // No valid authentication found
    Err(StatusCode::UNAUTHORIZED)
}

/// Extract authenticated user from request extensions
pub fn get_authenticated_user(request: &Request) -> Option<&AuthenticatedUser> {
    request.extensions().get::<AuthenticatedUser>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_state_disabled() {
        let config = AuthConfig::default();
        let state = AuthState::new(config);
        assert!(!state.is_enabled());
    }

    #[test]
    fn test_auth_state_validate_key() {
        let config = AuthConfig {
            enabled: true,
            api_keys: vec!["test-key-123".to_string()],
            secret: None,
        };
        let state = AuthState::new(config);

        assert!(state.validate_api_key("test-key-123"));
        assert!(!state.validate_api_key("invalid-key"));
    }
}
