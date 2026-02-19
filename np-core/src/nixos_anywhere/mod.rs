//! nixos-anywhere integration module
//!
//! Provides functionality for:
//! - Installing NixOS on remote machines using nixos-anywhere
//! - Managing installation jobs
//! - VM testing support

pub mod installer;
pub mod job_manager;
pub mod types;

pub use installer::NixosAnywhereInstaller;
pub use job_manager::JobManager;
pub use types::{
    DiskoMode, InstallJob, InstallPhase, InstallRequest, JobId, JobStatus, JobSummary,
};
