use crate::ssh::SshTarget;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a machine
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MachineId(pub String);

impl MachineId {
    /// Generate a new random machine ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from an existing string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Default for MachineId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MachineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for MachineId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Status of a machine connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MachineStatus {
    /// Connection status unknown (not tested)
    Unknown,
    /// Machine is reachable and authenticated
    Online,
    /// Machine is not reachable
    Offline,
    /// Authentication failed
    AuthFailed,
}

impl Default for MachineStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// A managed machine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    /// Unique identifier
    pub id: MachineId,

    /// Display name for the machine
    pub name: String,

    /// Optional description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// SSH connection target
    pub target: SshTarget,

    /// Tags for organizing machines
    #[serde(default)]
    pub tags: Vec<String>,

    /// Current connection status
    #[serde(default)]
    pub status: MachineStatus,

    /// Last time the machine was successfully contacted
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<DateTime<Utc>>,

    /// System information from last successful connection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_info: Option<String>,

    /// When the machine was added
    pub created_at: DateTime<Utc>,

    /// When the machine config was last updated
    pub updated_at: DateTime<Utc>,
}

impl Machine {
    /// Create a new machine with the given name and target
    pub fn new(name: impl Into<String>, target: SshTarget) -> Self {
        let now = Utc::now();
        Self {
            id: MachineId::new(),
            name: name.into(),
            description: None,
            target,
            tags: Vec::new(),
            status: MachineStatus::Unknown,
            last_seen: None,
            system_info: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Update the status
    pub fn update_status(&mut self, status: MachineStatus, system_info: Option<String>) {
        self.status = status;
        self.updated_at = Utc::now();

        if status == MachineStatus::Online {
            self.last_seen = Some(Utc::now());
            if let Some(info) = system_info {
                self.system_info = Some(info);
            }
        }
    }
}

/// Request to create a new machine
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMachineRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    #[serde(default)]
    pub auth_method: CreateMachineAuthMethod,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_port() -> u16 {
    22
}

/// Auth method for creating a machine
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreateMachineAuthMethod {
    #[default]
    Agent,
    KeyFile {
        path: String,
        #[serde(default)]
        passphrase: Option<String>,
    },
    Password {
        password: String,
    },
}

/// Request to update a machine
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMachineRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub auth_method: Option<CreateMachineAuthMethod>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}
