use std::path::PathBuf;
use thiserror::Error;

/// Main error type for np-core operations
#[derive(Error, Debug)]
pub enum NpError {
    #[error("SSH connection failed to {host}: {message}")]
    SshConnection { host: String, message: String },

    #[error("SSH authentication failed: {0}")]
    SshAuth(String),

    #[error("Nix command failed with exit code {code}: {stderr}")]
    NixCommand { code: i32, stderr: String },

    #[error("Nix command '{command}' timed out after {timeout_secs}s")]
    NixTimeout { command: String, timeout_secs: u64 },

    #[error("Nix executable not found at {path:?}")]
    NixNotFound { path: Option<PathBuf> },

    #[error("Flake parse error: {0}")]
    FlakeParse(String),

    #[error("Flake not found at {0}")]
    FlakeNotFound(PathBuf),

    #[error("Invalid flake reference: {0}")]
    InvalidFlakeRef(String),

    #[error("Installation failed: {0}")]
    Installation(String),

    #[error("Deployment failed: {0}")]
    Deployment(String),

    #[error("Service operation failed: {service}: {message}")]
    ServiceOperation { service: String, message: String },

    #[error("Machine not found: {0}")]
    MachineNotFound(String),

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Job cancelled: {0}")]
    JobCancelled(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid UTF-8 in command output")]
    InvalidUtf8,

    #[error("{0}")]
    Other(String),
}

/// Result type alias for np-core operations
pub type Result<T> = std::result::Result<T, NpError>;

impl NpError {
    /// Create an "other" error from any displayable type
    pub fn other<S: ToString>(msg: S) -> Self {
        Self::Other(msg.to_string())
    }
}
