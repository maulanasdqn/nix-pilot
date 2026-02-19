//! Secret types and data structures for SOPS integration

use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Unique identifier for a secret
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecretId(pub String);

impl SecretId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SecretId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of secret being stored
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretType {
    /// Plain text secret (password, API key, etc.)
    Text,
    /// SSH private key
    SshKey,
    /// TLS/SSL certificate
    Certificate,
    /// TLS/SSL private key
    TlsKey,
    /// Environment variable file
    EnvFile,
    /// Generic binary data
    Binary,
    /// JSON structured data
    Json,
    /// YAML structured data
    Yaml,
}

impl Default for SecretType {
    fn default() -> Self {
        Self::Text
    }
}

/// A decrypted secret with its metadata
#[derive(Debug, Clone)]
pub struct Secret {
    /// Unique identifier
    pub id: SecretId,
    /// Human-readable name
    pub name: String,
    /// Description of the secret
    pub description: Option<String>,
    /// Type of secret
    pub secret_type: SecretType,
    /// The actual secret value (wrapped for security)
    pub value: SecretString,
    /// Additional metadata
    pub metadata: SecretMetadata,
}

impl Secret {
    pub fn new(
        id: SecretId,
        name: impl Into<String>,
        value: impl Into<String>,
        secret_type: SecretType,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: None,
            secret_type,
            value: SecretString::from(value.into()),
            metadata: SecretMetadata::default(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Expose the secret value (use sparingly)
    pub fn expose_value(&self) -> &str {
        self.value.expose_secret()
    }
}

/// Metadata associated with a secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    /// When the secret was created
    pub created_at: DateTime<Utc>,
    /// When the secret was last modified
    pub modified_at: DateTime<Utc>,
    /// Who created the secret
    pub created_by: Option<String>,
    /// Version number
    pub version: u32,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Target machines (if applicable)
    pub machines: Vec<String>,
    /// Target environment (production, staging, etc.)
    pub environment: Option<String>,
    /// Custom key-value pairs
    pub extra: HashMap<String, String>,
}

impl Default for SecretMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            modified_at: now,
            created_by: None,
            version: 1,
            tags: Vec::new(),
            machines: Vec::new(),
            environment: None,
            extra: HashMap::new(),
        }
    }
}

/// An encrypted secret as stored on disk (SOPS format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSecret {
    /// The encrypted data
    pub data: HashMap<String, serde_yaml::Value>,
    /// SOPS metadata
    pub sops: SopsMetadata,
}

/// SOPS metadata block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopsMetadata {
    /// Key groups used for encryption
    #[serde(default)]
    pub key_groups: Vec<KeyGroup>,
    /// Age recipients
    #[serde(default)]
    pub age: Vec<AgeRecipient>,
    /// Last modified timestamp
    #[serde(rename = "lastmodified")]
    pub last_modified: Option<String>,
    /// MAC (Message Authentication Code)
    pub mac: Option<String>,
    /// Version of SOPS
    pub version: Option<String>,
}

impl Default for SopsMetadata {
    fn default() -> Self {
        Self {
            key_groups: Vec::new(),
            age: Vec::new(),
            last_modified: None,
            mac: None,
            version: Some("3.8.1".to_string()),
        }
    }
}

/// A key group for SOPS encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGroup {
    #[serde(default)]
    pub age: Vec<AgeRecipient>,
}

/// Age recipient information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeRecipient {
    /// Age public key
    pub recipient: String,
    /// Encrypted data key
    pub enc: Option<String>,
}

/// Configuration for the secrets manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// Base directory for secrets storage
    pub secrets_dir: PathBuf,
    /// Path to .sops.yaml configuration
    pub sops_config_path: PathBuf,
    /// Path to age key file
    pub age_key_path: PathBuf,
    /// Default recipients for new secrets
    pub default_recipients: Vec<String>,
    /// Whether to use SOPS CLI (vs native Rust implementation)
    pub use_sops_cli: bool,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            secrets_dir: PathBuf::from("secrets"),
            sops_config_path: PathBuf::from(".sops.yaml"),
            age_key_path: home.join(".config/sops/age/keys.txt"),
            default_recipients: Vec::new(),
            use_sops_cli: false,
        }
    }
}

/// A secret file reference (for sops-nix integration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretFile {
    /// Path to the encrypted secret file
    pub source_path: PathBuf,
    /// Target path on the NixOS system
    pub target_path: PathBuf,
    /// Owner user
    pub owner: String,
    /// Owner group
    pub group: String,
    /// File permissions (octal)
    pub mode: String,
    /// Whether to restart services on change
    pub restart_units: Vec<String>,
}

impl Default for SecretFile {
    fn default() -> Self {
        Self {
            source_path: PathBuf::new(),
            target_path: PathBuf::new(),
            owner: "root".to_string(),
            group: "root".to_string(),
            mode: "0400".to_string(),
            restart_units: Vec::new(),
        }
    }
}

/// Request to create a new secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSecretRequest {
    pub name: String,
    pub value: String,
    pub secret_type: SecretType,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub machines: Vec<String>,
    pub environment: Option<String>,
}

/// Request to update an existing secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSecretRequest {
    pub value: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub machines: Option<Vec<String>>,
}

/// Summary info for listing secrets (without exposing values)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretSummary {
    pub id: SecretId,
    pub name: String,
    pub description: Option<String>,
    pub secret_type: SecretType,
    pub metadata: SecretMetadata,
    pub file_path: PathBuf,
}

mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf())
    }
}
