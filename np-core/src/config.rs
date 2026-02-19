use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Core configuration for np-core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Path to the nix executable (defaults to "nix" in PATH)
    #[serde(default = "default_nix_path")]
    pub nix_path: PathBuf,

    /// Directory for storing SSH keys
    #[serde(default = "default_ssh_keys_dir")]
    pub ssh_keys_dir: PathBuf,

    /// Directory for storing machine configurations
    #[serde(default = "default_machines_dir")]
    pub machines_dir: PathBuf,

    /// Directory for storing flake registrations
    #[serde(default = "default_flakes_dir")]
    pub flakes_dir: PathBuf,

    /// Default timeout for nix commands in seconds
    #[serde(default = "default_command_timeout")]
    pub command_timeout_secs: u64,

    /// Maximum concurrent jobs
    #[serde(default = "default_max_concurrent_jobs")]
    pub max_concurrent_jobs: usize,
}

fn default_nix_path() -> PathBuf {
    PathBuf::from("nix")
}

fn default_ssh_keys_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "nix-pilot", "nix-pilot")
        .map(|d| d.data_dir().join("ssh-keys"))
        .unwrap_or_else(|| PathBuf::from(".nix-pilot/ssh-keys"))
}

fn default_machines_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "nix-pilot", "nix-pilot")
        .map(|d| d.data_dir().join("machines"))
        .unwrap_or_else(|| PathBuf::from(".nix-pilot/machines"))
}

fn default_flakes_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "nix-pilot", "nix-pilot")
        .map(|d| d.data_dir().join("flakes"))
        .unwrap_or_else(|| PathBuf::from(".nix-pilot/flakes"))
}

fn default_command_timeout() -> u64 {
    300 // 5 minutes
}

fn default_max_concurrent_jobs() -> usize {
    4
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            nix_path: default_nix_path(),
            ssh_keys_dir: default_ssh_keys_dir(),
            machines_dir: default_machines_dir(),
            flakes_dir: default_flakes_dir(),
            command_timeout_secs: default_command_timeout(),
            max_concurrent_jobs: default_max_concurrent_jobs(),
        }
    }
}

impl CoreConfig {
    /// Create config with custom data directory
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        Self {
            ssh_keys_dir: data_dir.join("ssh-keys"),
            machines_dir: data_dir.join("machines"),
            flakes_dir: data_dir.join("flakes"),
            ..Default::default()
        }
    }

    /// Ensure all required directories exist
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.ssh_keys_dir)?;
        std::fs::create_dir_all(&self.machines_dir)?;
        std::fs::create_dir_all(&self.flakes_dir)?;
        Ok(())
    }
}
