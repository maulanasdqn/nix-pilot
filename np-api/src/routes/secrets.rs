//! Secrets management API endpoints
//!
//! Provides REST API for managing SOPS-encrypted secrets:
//! - Age key management
//! - Secret CRUD operations
//! - sops-nix configuration generation

use axum::{
    extract::{Path, State},
    Json,
};
use np_core::{
    AgeKeyInfo, CreateSecretRequest, SecretId, SecretSummary, SecretsConfig, SecretsManager,
    UpdateSecretRequest,
};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

// ============================================================================
// Response types
// ============================================================================

#[derive(Serialize)]
pub struct KeyListResponse {
    pub keys: Vec<AgeKeyInfo>,
}

#[derive(Serialize)]
pub struct KeyResponse {
    pub public_key: String,
    pub comment: Option<String>,
}

#[derive(Serialize)]
pub struct SecretListResponse {
    pub secrets: Vec<SecretSummary>,
}

#[derive(Serialize)]
pub struct SecretResponse {
    pub secret: SecretSummary,
}

#[derive(Serialize)]
pub struct SecretValueResponse {
    pub id: String,
    pub name: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct SopsNixConfigResponse {
    pub config: String,
}

// ============================================================================
// Request types
// ============================================================================

#[derive(Deserialize)]
pub struct GenerateKeyRequest {
    pub comment: Option<String>,
}

#[derive(Deserialize)]
pub struct ImportKeyRequest {
    pub private_key: String,
    pub comment: Option<String>,
}

// ============================================================================
// Age Key Endpoints
// ============================================================================

/// List all age keys
pub async fn list_keys(State(state): State<AppState>) -> ApiResult<Json<KeyListResponse>> {
    let manager = get_secrets_manager(&state)?;
    let keys = manager
        .key_manager()
        .list_keys()
        .map_err(|e| ApiError::Internal(format!("Failed to list keys: {}", e)))?;

    Ok(Json(KeyListResponse { keys }))
}

/// Generate a new age key
pub async fn generate_key(
    State(state): State<AppState>,
    Json(request): Json<GenerateKeyRequest>,
) -> ApiResult<Json<KeyResponse>> {
    let manager = get_secrets_manager(&state)?;
    let key_pair = manager
        .key_manager()
        .generate_key(request.comment.as_deref())
        .map_err(|e| ApiError::Internal(format!("Failed to generate key: {}", e)))?;

    Ok(Json(KeyResponse {
        public_key: key_pair.public_key,
        comment: key_pair.comment,
    }))
}

/// Import an existing age key
pub async fn import_key(
    State(state): State<AppState>,
    Json(request): Json<ImportKeyRequest>,
) -> ApiResult<Json<KeyResponse>> {
    let manager = get_secrets_manager(&state)?;
    let key_pair = manager
        .key_manager()
        .import_key(&request.private_key, request.comment.as_deref())
        .map_err(|e| ApiError::Internal(format!("Failed to import key: {}", e)))?;

    Ok(Json(KeyResponse {
        public_key: key_pair.public_key,
        comment: key_pair.comment,
    }))
}

/// Get public keys only (for .sops.yaml configuration)
pub async fn get_public_keys(State(state): State<AppState>) -> ApiResult<Json<Vec<String>>> {
    let manager = get_secrets_manager(&state)?;
    let keys = manager
        .key_manager()
        .get_public_keys()
        .map_err(|e| ApiError::Internal(format!("Failed to get public keys: {}", e)))?;

    Ok(Json(keys))
}

// ============================================================================
// Secret Endpoints
// ============================================================================

/// List all secrets (metadata only, no values)
pub async fn list_secrets(State(state): State<AppState>) -> ApiResult<Json<SecretListResponse>> {
    let manager = get_secrets_manager(&state)?;
    let secrets = manager
        .list_secrets()
        .map_err(|e| ApiError::Internal(format!("Failed to list secrets: {}", e)))?;

    Ok(Json(SecretListResponse { secrets }))
}

/// Get a secret's metadata (without value)
pub async fn get_secret(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<SecretResponse>> {
    let manager = get_secrets_manager(&state)?;
    let secrets = manager
        .list_secrets()
        .map_err(|e| ApiError::Internal(format!("Failed to list secrets: {}", e)))?;

    let secret = secrets
        .into_iter()
        .find(|s| s.id.as_str() == id)
        .ok_or_else(|| ApiError::NotFound(format!("Secret not found: {}", id)))?;

    Ok(Json(SecretResponse { secret }))
}

/// Get a secret's decrypted value
pub async fn get_secret_value(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<SecretValueResponse>> {
    let manager = get_secrets_manager(&state)?;
    let secret_id = SecretId::new(&id);

    let secret = manager
        .get_secret(&secret_id)
        .map_err(|e| ApiError::NotFound(format!("Secret not found: {}", e)))?;

    let value = secret.expose_value().to_string();
    Ok(Json(SecretValueResponse {
        id: secret.id.to_string(),
        name: secret.name,
        value,
    }))
}

/// Create a new secret
pub async fn create_secret(
    State(state): State<AppState>,
    Json(request): Json<CreateSecretRequest>,
) -> ApiResult<Json<SecretResponse>> {
    let manager = get_secrets_manager(&state)?;

    // Ensure we have at least one key
    manager
        .ensure_key(Some("auto-generated"))
        .map_err(|e| ApiError::Internal(format!("Failed to ensure key: {}", e)))?;

    let secret = manager
        .create_secret(&request)
        .map_err(|e| ApiError::Internal(format!("Failed to create secret: {}", e)))?;

    Ok(Json(SecretResponse { secret }))
}

/// Update an existing secret
pub async fn update_secret(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateSecretRequest>,
) -> ApiResult<Json<SecretResponse>> {
    let manager = get_secrets_manager(&state)?;
    let secret_id = SecretId::new(&id);

    // Get existing secret to get metadata
    let existing = manager
        .get_secret(&secret_id)
        .map_err(|e| ApiError::NotFound(format!("Secret not found: {}", e)))?;

    // Create updated request
    let create_request = CreateSecretRequest {
        name: existing.name.clone(),
        value: request.value.unwrap_or_else(|| existing.expose_value().to_string()),
        secret_type: existing.secret_type,
        description: request.description.or(existing.description.clone()),
        tags: request.tags.unwrap_or_else(|| existing.metadata.tags.clone()),
        machines: request
            .machines
            .unwrap_or_else(|| existing.metadata.machines.clone()),
        environment: existing.metadata.environment.clone(),
    };

    // Delete old and create new
    manager
        .delete_secret(&secret_id)
        .map_err(|e| ApiError::Internal(format!("Failed to delete old secret: {}", e)))?;

    let secret = manager
        .create_secret(&create_request)
        .map_err(|e| ApiError::Internal(format!("Failed to create updated secret: {}", e)))?;

    Ok(Json(SecretResponse { secret }))
}

/// Delete a secret
pub async fn delete_secret(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let manager = get_secrets_manager(&state)?;
    let secret_id = SecretId::new(&id);

    manager
        .delete_secret(&secret_id)
        .map_err(|e| ApiError::NotFound(format!("Secret not found: {}", e)))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ============================================================================
// Configuration Endpoints
// ============================================================================

/// Update .sops.yaml with current public keys
pub async fn update_sops_config(
    State(state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    let manager = get_secrets_manager(&state)?;

    manager
        .update_sops_config()
        .map_err(|e| ApiError::Internal(format!("Failed to update SOPS config: {}", e)))?;

    Ok(Json(serde_json::json!({ "updated": true })))
}

/// Generate sops-nix configuration snippet
pub async fn generate_sops_nix_config(
    State(state): State<AppState>,
) -> ApiResult<Json<SopsNixConfigResponse>> {
    let manager = get_secrets_manager(&state)?;
    let secrets = manager
        .list_secrets()
        .map_err(|e| ApiError::Internal(format!("Failed to list secrets: {}", e)))?;

    let public_keys = manager
        .key_manager()
        .get_public_keys()
        .map_err(|e| ApiError::Internal(format!("Failed to get public keys: {}", e)))?;

    // Generate Nix configuration
    let mut config = String::new();
    config.push_str("# sops-nix configuration generated by nix-pilot\n");
    config.push_str("{\n");
    config.push_str("  sops = {\n");

    // Default settings
    config.push_str("    defaultSopsFile = ./secrets/secrets.yaml;\n");
    if let Some(key) = public_keys.first() {
        config.push_str(&format!("    # age.publicKeys = [ \"{}\" ];\n", key));
    }
    config.push_str("    age.keyFile = \"/var/lib/sops-nix/key.txt\";\n\n");

    // Secrets
    config.push_str("    secrets = {\n");
    for secret in &secrets {
        config.push_str(&format!(
            "      \"{}\" = {{\n",
            secret.name.replace('\"', "\\\"")
        ));
        config.push_str(&format!(
            "        sopsFile = {};\n",
            secret.file_path.display()
        ));
        config.push_str("        owner = \"root\";\n");
        config.push_str("        group = \"root\";\n");
        config.push_str("        mode = \"0400\";\n");
        config.push_str("      };\n");
    }
    config.push_str("    };\n");
    config.push_str("  };\n");
    config.push_str("}\n");

    Ok(Json(SopsNixConfigResponse { config }))
}

// ============================================================================
// Helper functions
// ============================================================================

fn get_secrets_manager(state: &AppState) -> ApiResult<SecretsManager> {
    let config = SecretsConfig {
        secrets_dir: state.config.data_dir.join("secrets"),
        sops_config_path: state.config.data_dir.join(".sops.yaml"),
        age_key_path: state.config.data_dir.join("age/keys.txt"),
        default_recipients: Vec::new(),
        use_sops_cli: false,
    };

    SecretsManager::with_config(config)
        .map_err(|e| ApiError::Internal(format!("Failed to initialize secrets manager: {}", e)))
}
