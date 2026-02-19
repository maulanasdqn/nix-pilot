//! np-core - Core business logic for nix-pilot
//!
//! This crate provides the core functionality for:
//! - Executing nix commands with streaming output
//! - SSH connection management
//! - Flake parsing and manipulation
//! - NixOS service management
//! - nixos-anywhere integration

pub mod config;
pub mod deploy;
pub mod error;
pub mod flake;
pub mod machine;
pub mod nix;
pub mod nixos_anywhere;
pub mod secrets;
pub mod service;
pub mod ssh;

// Re-export commonly used types
pub use config::CoreConfig;
pub use deploy::{
    DeployAction, DeployExecutor, DeployJob, DeployJobId, DeployJobSummary, DeployManager,
    DeployPhase, DeployRequest, DeployStatus, GenerationInfo, RollbackRequest,
};
pub use error::{NpError, Result};
pub use flake::{
    CreateFlakeRequest, FlakeId, FlakeInput, FlakeManager, FlakeMetadata, FlakeOutputs,
    FlakeParser, RegisteredFlake, UpdateFlakeRequest, UpdateInputRequest,
};
pub use machine::{
    CreateMachineRequest, Machine, MachineId, MachineManager, MachineStatus, UpdateMachineRequest,
};
pub use nix::{
    ClosureInfo, CommandOutput, NixExecutor, OutputLine, OutputStream, PathInfo,
    ProfileGeneration, SearchResult, StoreInfo,
};
pub use nixos_anywhere::{
    DiskoMode, InstallJob, InstallPhase, InstallRequest, JobId, JobManager, JobStatus, JobSummary,
    NixosAnywhereInstaller,
};
pub use service::{
    LoadState, LogEntry, LogOptions, LogOutput, LogPriority, ServiceAction, ServiceActionRequest,
    ServiceActionResult, ServiceInfo, ServiceState, ServiceStatus, ServiceSubState, SystemdManager,
};
pub use secrets::{
    AgeKeyInfo, AgeKeyManager, AgeKeyPair, CreateSecretRequest, EncryptedSecret, Secret,
    SecretFile, SecretId, SecretMetadata, SecretSummary, SecretType, SecretsConfig, SecretsManager,
    UpdateSecretRequest,
};
pub use ssh::{SshAuthMethod, SshCredentials, SshSession, SshTarget};
