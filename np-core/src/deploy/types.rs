//! Types for deployment operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ssh::SshTarget;

/// Unique identifier for a deployment job
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeployJobId(pub String);

impl DeployJobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for DeployJobId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DeployJobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Deployment action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeployAction {
    /// Switch to the new configuration (nixos-rebuild switch)
    Switch,
    /// Boot into new configuration on next reboot (nixos-rebuild boot)
    Boot,
    /// Build and activate without adding to boot menu (nixos-rebuild test)
    Test,
    /// Only build, don't activate (nixos-rebuild build)
    Build,
    /// Build on remote host
    BuildRemote,
    /// Dry run (show what would be built)
    DryBuild,
    /// Dry run activation
    DryActivate,
}

impl DeployAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Switch => "switch",
            Self::Boot => "boot",
            Self::Test => "test",
            Self::Build => "build",
            Self::BuildRemote => "build",
            Self::DryBuild => "dry-build",
            Self::DryActivate => "dry-activate",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Switch => "Build, activate, and add to boot menu",
            Self::Boot => "Build and add to boot menu (activate on reboot)",
            Self::Test => "Build and activate (don't add to boot menu)",
            Self::Build => "Build only (don't activate)",
            Self::BuildRemote => "Build on the remote host",
            Self::DryBuild => "Show what would be built",
            Self::DryActivate => "Show what would change on activation",
        }
    }
}

/// Deployment job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeployStatus {
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

/// Phase of deployment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeployPhase {
    /// Initializing deployment
    Initializing,
    /// Evaluating flake
    Evaluating,
    /// Building the system
    Building,
    /// Copying closure to target
    Copying,
    /// Activating the configuration
    Activating,
    /// Deployment complete
    Complete,
}

impl DeployPhase {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Initializing => "Initializing deployment...",
            Self::Evaluating => "Evaluating flake configuration...",
            Self::Building => "Building system configuration...",
            Self::Copying => "Copying closure to target...",
            Self::Activating => "Activating configuration...",
            Self::Complete => "Deployment complete!",
        }
    }

    pub fn progress(&self) -> u8 {
        match self {
            Self::Initializing => 0,
            Self::Evaluating => 10,
            Self::Building => 30,
            Self::Copying => 70,
            Self::Activating => 90,
            Self::Complete => 100,
        }
    }
}

/// Request to start a deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployRequest {
    /// SSH target for the deployment
    pub target: SshTarget,
    /// Flake reference (e.g., "github:user/repo#nixosConfigurations.myhost")
    pub flake_ref: String,
    /// Deployment action (switch, boot, test, etc.)
    pub action: DeployAction,
    /// Build on the target host instead of locally
    #[serde(default)]
    pub build_on_target: bool,
    /// Use substitutes (binary caches)
    #[serde(default = "default_true")]
    pub use_substitutes: bool,
    /// Extra arguments to pass to nixos-rebuild
    #[serde(default)]
    pub extra_args: Vec<String>,
    /// Rollback to previous generation if activation fails
    #[serde(default)]
    pub rollback_on_failure: bool,
}

fn default_true() -> bool {
    true
}

/// A deployment job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployJob {
    pub id: DeployJobId,
    pub request: DeployRequest,
    pub status: DeployStatus,
    pub phase: DeployPhase,
    pub progress_percent: u8,
    pub output_lines: Vec<String>,
    pub error: Option<String>,
    pub exit_code: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// Store path of the built system (for rollback)
    pub built_system: Option<String>,
    /// Previous generation number (for rollback info)
    pub previous_generation: Option<u32>,
}

impl DeployJob {
    pub fn new(request: DeployRequest) -> Self {
        Self {
            id: DeployJobId::new(),
            request,
            status: DeployStatus::Pending,
            phase: DeployPhase::Initializing,
            progress_percent: 0,
            output_lines: Vec::new(),
            error: None,
            exit_code: None,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
            built_system: None,
            previous_generation: None,
        }
    }

    pub fn start(&mut self) {
        self.status = DeployStatus::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn set_phase(&mut self, phase: DeployPhase) {
        self.phase = phase;
        self.progress_percent = phase.progress();
    }

    pub fn add_output(&mut self, line: String) {
        self.output_lines.push(line);
    }

    pub fn complete(&mut self, exit_code: i32) {
        self.status = if exit_code == 0 {
            DeployStatus::Completed
        } else {
            DeployStatus::Failed
        };
        self.exit_code = Some(exit_code);
        self.completed_at = Some(Utc::now());
        if exit_code == 0 {
            self.phase = DeployPhase::Complete;
            self.progress_percent = 100;
        }
    }

    pub fn fail(&mut self, error: String) {
        self.status = DeployStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
    }

    pub fn cancel(&mut self) {
        self.status = DeployStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }
}

/// Summary of a deployment job (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployJobSummary {
    pub id: DeployJobId,
    pub target_host: String,
    pub flake_ref: String,
    pub action: DeployAction,
    pub status: DeployStatus,
    pub phase: DeployPhase,
    pub progress_percent: u8,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<&DeployJob> for DeployJobSummary {
    fn from(job: &DeployJob) -> Self {
        Self {
            id: job.id.clone(),
            target_host: job.request.target.host.clone(),
            flake_ref: job.request.flake_ref.clone(),
            action: job.request.action,
            status: job.status,
            phase: job.phase,
            progress_percent: job.progress_percent,
            created_at: job.created_at,
            completed_at: job.completed_at,
        }
    }
}

/// Request to rollback to a previous generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRequest {
    /// SSH target for the rollback
    pub target: SshTarget,
    /// Generation number to rollback to (None = previous)
    pub generation: Option<u32>,
}

/// Information about a system generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationInfo {
    pub number: u32,
    pub date: DateTime<Utc>,
    pub current: bool,
    pub nixos_version: Option<String>,
    pub kernel_version: Option<String>,
    pub configuration_revision: Option<String>,
}
