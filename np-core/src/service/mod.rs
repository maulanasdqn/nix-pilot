//! Systemd service management module
//!
//! Provides functionality for:
//! - Listing and querying systemd services
//! - Starting, stopping, restarting services
//! - Enabling/disabling services
//! - Streaming journalctl logs

pub mod manager;
pub mod types;

pub use manager::SystemdManager;
pub use types::{
    LoadState, LogEntry, LogOptions, LogOutput, LogPriority, ServiceAction, ServiceActionRequest,
    ServiceActionResult, ServiceInfo, ServiceState, ServiceStatus, ServiceSubState,
};
