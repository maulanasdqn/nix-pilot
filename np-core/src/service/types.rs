//! Types for systemd service management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Service state (from systemctl)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceState {
    Active,
    Inactive,
    Failed,
    Activating,
    Deactivating,
    Reloading,
    Unknown,
}

impl ServiceState {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => Self::Active,
            "inactive" => Self::Inactive,
            "failed" => Self::Failed,
            "activating" => Self::Activating,
            "deactivating" => Self::Deactivating,
            "reloading" => Self::Reloading,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Failed => "failed",
            Self::Activating => "activating",
            Self::Deactivating => "deactivating",
            Self::Reloading => "reloading",
            Self::Unknown => "unknown",
        }
    }
}

/// Service sub-state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceSubState {
    Running,
    Exited,
    Dead,
    Failed,
    Waiting,
    Starting,
    Stopping,
    Unknown,
}

impl ServiceSubState {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "running" => Self::Running,
            "exited" => Self::Exited,
            "dead" => Self::Dead,
            "failed" => Self::Failed,
            "waiting" => Self::Waiting,
            "start" | "starting" => Self::Starting,
            "stop" | "stopping" => Self::Stopping,
            _ => Self::Unknown,
        }
    }
}

/// Service load state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoadState {
    Loaded,
    NotFound,
    Error,
    Masked,
    Unknown,
}

impl LoadState {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "loaded" => Self::Loaded,
            "not-found" => Self::NotFound,
            "error" => Self::Error,
            "masked" => Self::Masked,
            _ => Self::Unknown,
        }
    }
}

/// Information about a systemd service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service unit name (e.g., "nginx.service")
    pub name: String,
    /// Service description
    pub description: Option<String>,
    /// Load state
    pub load_state: LoadState,
    /// Active state
    pub active_state: ServiceState,
    /// Sub-state (running, exited, etc.)
    pub sub_state: ServiceSubState,
    /// Whether service is enabled
    pub enabled: bool,
    /// Main PID (if running)
    pub main_pid: Option<u32>,
    /// Memory usage in bytes
    pub memory_bytes: Option<u64>,
    /// CPU time in microseconds
    pub cpu_time_usec: Option<u64>,
    /// When the service was started
    pub started_at: Option<DateTime<Utc>>,
    /// Unit file path
    pub unit_file_path: Option<String>,
}

/// Detailed service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub info: ServiceInfo,
    /// Recent log lines
    pub recent_logs: Vec<String>,
    /// Triggered by (for .timer units)
    pub triggered_by: Option<String>,
    /// Triggers (for .timer units)
    pub triggers: Option<String>,
    /// Dependencies
    pub requires: Vec<String>,
    pub wanted_by: Vec<String>,
    pub after: Vec<String>,
    pub before: Vec<String>,
}

/// Service action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
    Reload,
    Enable,
    Disable,
    Mask,
    Unmask,
}

impl ServiceAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
            Self::Reload => "reload",
            Self::Enable => "enable",
            Self::Disable => "disable",
            Self::Mask => "mask",
            Self::Unmask => "unmask",
        }
    }
}

/// Request to perform a service action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceActionRequest {
    /// Service name (e.g., "nginx.service" or "nginx")
    pub service: String,
    /// Action to perform
    pub action: ServiceAction,
}

/// Result of a service action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceActionResult {
    pub service: String,
    pub action: ServiceAction,
    pub success: bool,
    pub message: String,
    pub exit_code: i32,
}

/// Log entry from journalctl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub priority: LogPriority,
    pub unit: String,
    pub message: String,
    pub pid: Option<u32>,
    pub hostname: Option<String>,
}

/// Log priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogPriority {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
}

impl LogPriority {
    pub fn from_number(n: u8) -> Self {
        match n {
            0 => Self::Emergency,
            1 => Self::Alert,
            2 => Self::Critical,
            3 => Self::Error,
            4 => Self::Warning,
            5 => Self::Notice,
            6 => Self::Info,
            7 => Self::Debug,
            _ => Self::Info,
        }
    }
}

/// Options for fetching logs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogOptions {
    /// Number of lines to fetch (default: 100)
    pub lines: Option<u32>,
    /// Fetch logs since this time
    pub since: Option<DateTime<Utc>>,
    /// Fetch logs until this time
    pub until: Option<DateTime<Utc>>,
    /// Filter by priority (show this and higher)
    pub priority: Option<LogPriority>,
    /// Follow logs (streaming)
    pub follow: bool,
    /// Show full messages (no truncation)
    pub full: bool,
    /// Output format
    pub output: Option<LogOutput>,
}

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogOutput {
    Short,
    ShortIso,
    ShortPrecise,
    Cat,
    Json,
}

impl LogOutput {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::ShortIso => "short-iso",
            Self::ShortPrecise => "short-precise",
            Self::Cat => "cat",
            Self::Json => "json",
        }
    }
}
