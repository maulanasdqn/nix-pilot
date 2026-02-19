//! Deployment module
//!
//! Provides functionality for:
//! - Deploying NixOS configurations to remote machines
//! - Managing deployment jobs
//! - Rollback support
//! - Generation management

pub mod executor;
pub mod manager;
pub mod types;

pub use executor::DeployExecutor;
pub use manager::DeployManager;
pub use types::{
    DeployAction, DeployJob, DeployJobId, DeployJobSummary, DeployPhase, DeployRequest,
    DeployStatus, GenerationInfo, RollbackRequest,
};
