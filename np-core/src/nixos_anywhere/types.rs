//! Types for nixos-anywhere operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ssh::SshTarget;

/// Unique identifier for an installation job
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub String);

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Installation job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Job is waiting to start
    Pending,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
    /// Job was cancelled
    Cancelled,
}

/// Phase of installation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallPhase {
    /// Initializing installation
    Initializing,
    /// Connecting to target
    Connecting,
    /// Running kexec to boot installer
    Kexec,
    /// Formatting disks with disko
    Formatting,
    /// Installing NixOS
    Installing,
    /// Rebooting into installed system
    Rebooting,
    /// Verifying installation
    Verifying,
    /// Installation complete
    Complete,
}

impl InstallPhase {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Initializing => "Initializing installation...",
            Self::Connecting => "Connecting to target machine...",
            Self::Kexec => "Booting into NixOS installer (kexec)...",
            Self::Formatting => "Formatting disks with disko...",
            Self::Installing => "Installing NixOS...",
            Self::Rebooting => "Rebooting into installed system...",
            Self::Verifying => "Verifying installation...",
            Self::Complete => "Installation complete!",
        }
    }
}

/// Request to start an installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRequest {
    /// SSH target for the installation
    pub target: SshTarget,
    /// Flake reference (e.g., "github:user/repo#nixosConfigurations.myhost")
    pub flake_ref: String,
    /// Whether to run in VM test mode
    pub vm_test: bool,
    /// Extra arguments to pass to nixos-anywhere
    pub extra_args: Vec<String>,
    /// Build host (if different from target)
    pub build_host: Option<SshTarget>,
    /// Whether to use kexec (default: true)
    #[serde(default = "default_true")]
    pub kexec: bool,
    /// Whether to wipe existing disks
    #[serde(default)]
    pub wipe: bool,
    /// Disko mode (default: disko)
    pub disko_mode: Option<DiskoMode>,
}

fn default_true() -> bool {
    true
}

/// Disko mode for disk management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiskoMode {
    /// Use disko for disk management
    Disko,
    /// Use disko with --mode format
    DiskoFormat,
    /// Use disko with --mode mount
    DiskoMount,
    /// Don't use disko
    None,
}

/// An installation job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallJob {
    pub id: JobId,
    pub request: InstallRequest,
    pub status: JobStatus,
    pub phase: InstallPhase,
    pub progress_percent: u8,
    pub output_lines: Vec<String>,
    pub error: Option<String>,
    pub exit_code: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl InstallJob {
    pub fn new(request: InstallRequest) -> Self {
        Self {
            id: JobId::new(),
            request,
            status: JobStatus::Pending,
            phase: InstallPhase::Initializing,
            progress_percent: 0,
            output_lines: Vec::new(),
            error: None,
            exit_code: None,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn start(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn set_phase(&mut self, phase: InstallPhase) {
        self.phase = phase;
        // Update progress based on phase
        self.progress_percent = match phase {
            InstallPhase::Initializing => 0,
            InstallPhase::Connecting => 10,
            InstallPhase::Kexec => 20,
            InstallPhase::Formatting => 40,
            InstallPhase::Installing => 60,
            InstallPhase::Rebooting => 85,
            InstallPhase::Verifying => 95,
            InstallPhase::Complete => 100,
        };
    }

    pub fn add_output(&mut self, line: String) {
        self.output_lines.push(line);
    }

    pub fn complete(&mut self, exit_code: i32) {
        self.status = if exit_code == 0 {
            JobStatus::Completed
        } else {
            JobStatus::Failed
        };
        self.exit_code = Some(exit_code);
        self.completed_at = Some(Utc::now());
        if exit_code == 0 {
            self.phase = InstallPhase::Complete;
            self.progress_percent = 100;
        }
    }

    pub fn fail(&mut self, error: String) {
        self.status = JobStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
    }

    pub fn cancel(&mut self) {
        self.status = JobStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }
}

/// Summary of a job (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSummary {
    pub id: JobId,
    pub target_host: String,
    pub flake_ref: String,
    pub status: JobStatus,
    pub phase: InstallPhase,
    pub progress_percent: u8,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<&InstallJob> for JobSummary {
    fn from(job: &InstallJob) -> Self {
        Self {
            id: job.id.clone(),
            target_host: job.request.target.host.clone(),
            flake_ref: job.request.flake_ref.clone(),
            status: job.status,
            phase: job.phase,
            progress_percent: job.progress_percent,
            created_at: job.created_at,
            completed_at: job.completed_at,
        }
    }
}
