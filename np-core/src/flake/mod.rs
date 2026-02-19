//! Flake management module
//!
//! Provides functionality for:
//! - Parsing flake metadata and outputs
//! - Managing registered flakes
//! - Editing flake inputs
//! - Updating flake.lock files

pub mod manager;
pub mod parser;
pub mod types;

pub use manager::FlakeManager;
pub use parser::FlakeParser;
pub use types::{
    CreateFlakeRequest, FlakeId, FlakeInput, FlakeInputLocked, FlakeInputRef, FlakeLockNode,
    FlakeLocks, FlakeMetadata, FlakeOutputEntry, FlakeOutputs, LockNodeInput, RegisteredFlake,
    UpdateFlakeRequest, UpdateInputRequest,
};
