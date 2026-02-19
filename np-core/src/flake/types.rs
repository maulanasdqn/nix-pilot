//! Flake-related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for a registered flake
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlakeId(pub String);

impl FlakeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for FlakeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for FlakeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A registered flake in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredFlake {
    pub id: FlakeId,
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub metadata: Option<FlakeMetadata>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Metadata parsed from `nix flake metadata --json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeMetadata {
    pub description: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub locked_url: Option<String>,
    pub original_url: Option<String>,
    pub path: PathBuf,
    pub resolved_url: Option<String>,
    pub revision: Option<String>,
    pub inputs: HashMap<String, FlakeInput>,
    pub locks: Option<FlakeLocks>,
}

/// A single input in a flake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeInput {
    pub name: String,
    pub original: FlakeInputRef,
    pub locked: Option<FlakeInputLocked>,
    pub follows: Option<Vec<String>>,
}

/// Original (unlocked) input reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeInputRef {
    /// Input type (github, path, git, etc.)
    #[serde(rename = "type")]
    pub input_type: String,
    /// Owner for github/gitlab inputs
    pub owner: Option<String>,
    /// Repo name for github/gitlab inputs
    pub repo: Option<String>,
    /// Path for local/path inputs
    pub path: Option<String>,
    /// URL for git/tarball inputs
    pub url: Option<String>,
    /// Git ref (branch/tag)
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
    /// Git revision
    pub rev: Option<String>,
}

impl FlakeInputRef {
    /// Format as a flake reference string
    pub fn to_flake_ref(&self) -> String {
        match self.input_type.as_str() {
            "github" => {
                let mut ref_str = format!(
                    "github:{}/{}",
                    self.owner.as_deref().unwrap_or(""),
                    self.repo.as_deref().unwrap_or("")
                );
                if let Some(r) = &self.git_ref {
                    ref_str.push('/');
                    ref_str.push_str(r);
                }
                if let Some(rev) = &self.rev {
                    ref_str.push('/');
                    ref_str.push_str(rev);
                }
                ref_str
            }
            "gitlab" => {
                format!(
                    "gitlab:{}/{}",
                    self.owner.as_deref().unwrap_or(""),
                    self.repo.as_deref().unwrap_or("")
                )
            }
            "git" => {
                let mut ref_str = format!("git+{}", self.url.as_deref().unwrap_or(""));
                if let Some(r) = &self.git_ref {
                    ref_str.push_str("?ref=");
                    ref_str.push_str(r);
                }
                ref_str
            }
            "path" => format!("path:{}", self.path.as_deref().unwrap_or(".")),
            "tarball" => self.url.clone().unwrap_or_default(),
            _ => self.url.clone().unwrap_or_else(|| self.input_type.clone()),
        }
    }
}

/// Locked input information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeInputLocked {
    /// Input type
    #[serde(rename = "type")]
    pub input_type: String,
    /// Owner for github/gitlab
    pub owner: Option<String>,
    /// Repo name
    pub repo: Option<String>,
    /// Locked revision
    pub rev: Option<String>,
    /// NAR hash
    #[serde(rename = "narHash")]
    pub nar_hash: Option<String>,
    /// Last modified timestamp
    #[serde(rename = "lastModified")]
    pub last_modified: Option<i64>,
}

/// Flake lock file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeLocks {
    pub version: i32,
    pub root: String,
    pub nodes: HashMap<String, FlakeLockNode>,
}

/// A node in the flake.lock file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeLockNode {
    pub inputs: Option<HashMap<String, LockNodeInput>>,
    pub locked: Option<FlakeInputLocked>,
    pub original: Option<FlakeInputRef>,
}

/// Input reference in a lock node (can be string or array for follows)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LockNodeInput {
    Direct(String),
    Follows(Vec<String>),
}

/// Flake outputs structure from `nix flake show --json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeOutputs {
    #[serde(rename = "nixosConfigurations")]
    pub nixos_configurations: Option<HashMap<String, FlakeOutputEntry>>,
    #[serde(rename = "darwinConfigurations")]
    pub darwin_configurations: Option<HashMap<String, FlakeOutputEntry>>,
    pub packages: Option<HashMap<String, HashMap<String, FlakeOutputEntry>>>,
    #[serde(rename = "devShells")]
    pub dev_shells: Option<HashMap<String, HashMap<String, FlakeOutputEntry>>>,
    pub apps: Option<HashMap<String, HashMap<String, FlakeOutputEntry>>>,
    pub overlays: Option<HashMap<String, FlakeOutputEntry>>,
    #[serde(rename = "nixosModules")]
    pub nixos_modules: Option<HashMap<String, FlakeOutputEntry>>,
    pub templates: Option<HashMap<String, FlakeOutputEntry>>,
}

/// A single flake output entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeOutputEntry {
    #[serde(rename = "type")]
    pub output_type: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Request to create/register a new flake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFlakeRequest {
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
}

/// Request to update a registered flake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFlakeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Request to update a flake input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInputRequest {
    /// New flake reference for the input
    pub flake_ref: Option<String>,
    /// List of inputs to follow (e.g., ["nixpkgs"])
    pub follows: Option<Vec<String>>,
}
