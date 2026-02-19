use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// API error type with structured error codes
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Unauthorized,
    Forbidden(String),
    Conflict(String),
    Internal(String),
    ServiceUnavailable(String),
    Core(np_core::NpError),
}

/// Error codes for programmatic error handling
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // General errors
    NotFound,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    InternalError,
    ServiceUnavailable,

    // SSH/Connection errors
    SshConnectionFailed,
    SshAuthFailed,

    // Nix errors
    NixCommandFailed,
    NixTimeout,
    NixNotFound,

    // Flake errors
    FlakeParseError,
    FlakeNotFound,
    InvalidFlakeRef,

    // Job errors
    JobNotFound,
    JobCancelled,
    JobFailed,

    // Machine errors
    MachineNotFound,

    // Service errors
    ServiceOperationFailed,

    // Deploy errors
    DeploymentFailed,
    InstallationFailed,

    // Secret errors
    SecretNotFound,
    EncryptionFailed,
    DecryptionFailed,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    /// Human-readable error message
    pub error: String,
    /// Machine-readable error code
    pub code: ErrorCode,
    /// Additional details (for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Request ID for tracing (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match self {
            ApiError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorCode::NotFound,
                msg,
                None,
            ),
            ApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorCode::BadRequest,
                msg,
                None,
            ),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                "Authentication required".to_string(),
                None,
            ),
            ApiError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                ErrorCode::Forbidden,
                msg,
                None,
            ),
            ApiError::Conflict(msg) => (
                StatusCode::CONFLICT,
                ErrorCode::Conflict,
                msg,
                None,
            ),
            ApiError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalError,
                "Internal server error".to_string(),
                Some(msg),
            ),
            ApiError::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorCode::ServiceUnavailable,
                msg,
                None,
            ),
            ApiError::Core(e) => map_core_error(e),
        };

        let body = Json(ErrorResponse {
            error: message,
            code,
            details,
            request_id: None, // Could be set from request extensions
        });

        (status, body).into_response()
    }
}

/// Map np_core errors to appropriate HTTP responses
fn map_core_error(err: np_core::NpError) -> (StatusCode, ErrorCode, String, Option<String>) {
    use np_core::NpError;

    match err {
        NpError::SshConnection { host, message } => (
            StatusCode::BAD_GATEWAY,
            ErrorCode::SshConnectionFailed,
            format!("SSH connection to {} failed", host),
            Some(message),
        ),
        NpError::SshAuth(msg) => (
            StatusCode::UNAUTHORIZED,
            ErrorCode::SshAuthFailed,
            "SSH authentication failed".to_string(),
            Some(msg),
        ),
        NpError::NixCommand { code, stderr } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::NixCommandFailed,
            format!("Nix command failed with exit code {}", code),
            Some(stderr),
        ),
        NpError::NixTimeout { command, timeout_secs } => (
            StatusCode::GATEWAY_TIMEOUT,
            ErrorCode::NixTimeout,
            format!("Command timed out after {}s", timeout_secs),
            Some(command),
        ),
        NpError::NixNotFound { path } => (
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::NixNotFound,
            "Nix executable not found".to_string(),
            path.map(|p| p.display().to_string()),
        ),
        NpError::FlakeParse(msg) => (
            StatusCode::BAD_REQUEST,
            ErrorCode::FlakeParseError,
            "Failed to parse flake".to_string(),
            Some(msg),
        ),
        NpError::FlakeNotFound(path) => (
            StatusCode::NOT_FOUND,
            ErrorCode::FlakeNotFound,
            format!("Flake not found at {}", path.display()),
            None,
        ),
        NpError::InvalidFlakeRef(msg) => (
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidFlakeRef,
            "Invalid flake reference".to_string(),
            Some(msg),
        ),
        NpError::MachineNotFound(id) => (
            StatusCode::NOT_FOUND,
            ErrorCode::MachineNotFound,
            format!("Machine not found: {}", id),
            None,
        ),
        NpError::JobNotFound(id) => (
            StatusCode::NOT_FOUND,
            ErrorCode::JobNotFound,
            format!("Job not found: {}", id),
            None,
        ),
        NpError::JobCancelled(id) => (
            StatusCode::CONFLICT,
            ErrorCode::JobCancelled,
            format!("Job was cancelled: {}", id),
            None,
        ),
        NpError::ServiceOperation { service, message } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::ServiceOperationFailed,
            format!("Service operation failed for {}", service),
            Some(message),
        ),
        NpError::Deployment(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::DeploymentFailed,
            "Deployment failed".to_string(),
            Some(msg),
        ),
        NpError::Installation(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InstallationFailed,
            "Installation failed".to_string(),
            Some(msg),
        ),
        NpError::Config(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            "Configuration error".to_string(),
            Some(msg),
        ),
        NpError::Io(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            "I/O error".to_string(),
            Some(e.to_string()),
        ),
        NpError::Json(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            "JSON serialization error".to_string(),
            Some(e.to_string()),
        ),
        NpError::InvalidUtf8 => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            "Invalid UTF-8 in command output".to_string(),
            None,
        ),
        NpError::Other(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            "An error occurred".to_string(),
            Some(msg),
        ),
        NpError::SecretNotFound(id) => (
            StatusCode::NOT_FOUND,
            ErrorCode::SecretNotFound,
            format!("Secret not found: {}", id),
            None,
        ),
        NpError::Encryption(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::EncryptionFailed,
            "Encryption failed".to_string(),
            Some(msg),
        ),
        NpError::Decryption(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::DecryptionFailed,
            "Decryption failed".to_string(),
            Some(msg),
        ),
    }
}

impl From<np_core::NpError> for ApiError {
    fn from(err: np_core::NpError) -> Self {
        ApiError::Core(err)
    }
}

/// Result type for API handlers
pub type ApiResult<T> = std::result::Result<T, ApiError>;
