use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// SSH authentication method
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SshAuthMethod {
    /// Use SSH agent for authentication
    Agent,

    /// Use a private key file
    KeyFile {
        /// Path to the private key file
        path: PathBuf,
        /// Optional passphrase for encrypted keys
        #[serde(skip_serializing)]
        passphrase: Option<String>,
    },

    /// Use password authentication (not recommended)
    Password {
        #[serde(skip_serializing)]
        password: String,
    },
}

impl Default for SshAuthMethod {
    fn default() -> Self {
        Self::Agent
    }
}

/// SSH credentials container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshCredentials {
    /// Username for SSH connection
    pub username: String,

    /// Authentication method
    #[serde(flatten)]
    pub auth: SshAuthMethod,
}

impl SshCredentials {
    /// Create credentials using SSH agent
    pub fn with_agent(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            auth: SshAuthMethod::Agent,
        }
    }

    /// Create credentials using a key file
    pub fn with_key_file(
        username: impl Into<String>,
        path: PathBuf,
        passphrase: Option<String>,
    ) -> Self {
        Self {
            username: username.into(),
            auth: SshAuthMethod::KeyFile { path, passphrase },
        }
    }

    /// Create credentials using password
    pub fn with_password(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            auth: SshAuthMethod::Password {
                password: password.into(),
            },
        }
    }

    /// Get the default SSH key paths to try
    pub fn default_key_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(home) = home::home_dir() {
            let ssh_dir = home.join(".ssh");
            // Try common key names in order of preference
            for name in &["id_ed25519", "id_ecdsa", "id_rsa"] {
                let path = ssh_dir.join(name);
                if path.exists() {
                    paths.push(path);
                }
            }
        }

        paths
    }
}
