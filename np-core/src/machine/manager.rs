use crate::error::{NpError, Result};
use crate::ssh::{SshAuthMethod, SshCredentials, SshSession, SshTarget};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{
    CreateMachineAuthMethod, CreateMachineRequest, Machine, MachineId, MachineStatus,
    UpdateMachineRequest,
};

/// Manager for machine configurations
pub struct MachineManager {
    /// In-memory cache of machines
    machines: Arc<RwLock<HashMap<MachineId, Machine>>>,
    /// Directory for storing machine configs
    storage_dir: PathBuf,
}

impl MachineManager {
    /// Create a new machine manager with the given storage directory
    pub fn new(storage_dir: PathBuf) -> Self {
        Self {
            machines: Arc::new(RwLock::new(HashMap::new())),
            storage_dir,
        }
    }

    /// Load all machines from storage
    pub async fn load(&self) -> Result<()> {
        if !self.storage_dir.exists() {
            tokio::fs::create_dir_all(&self.storage_dir).await?;
            return Ok(());
        }

        let mut machines = self.machines.write().await;
        let mut entries = tokio::fs::read_dir(&self.storage_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_machine_file(&path).await {
                    Ok(machine) => {
                        machines.insert(machine.id.clone(), machine);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load machine from {:?}: {}", path, e);
                    }
                }
            }
        }

        tracing::info!("Loaded {} machines", machines.len());
        Ok(())
    }

    /// Load a single machine from a file
    async fn load_machine_file(&self, path: &Path) -> Result<Machine> {
        let content = tokio::fs::read_to_string(path).await?;
        let machine: Machine = serde_json::from_str(&content)?;
        Ok(machine)
    }

    /// Save a machine to storage
    async fn save_machine(&self, machine: &Machine) -> Result<()> {
        tokio::fs::create_dir_all(&self.storage_dir).await?;

        let path = self.storage_dir.join(format!("{}.json", machine.id));
        let content = serde_json::to_string_pretty(machine)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Delete a machine file from storage
    async fn delete_machine_file(&self, id: &MachineId) -> Result<()> {
        let path = self.storage_dir.join(format!("{}.json", id));
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }
        Ok(())
    }

    /// List all machines
    pub async fn list(&self) -> Vec<Machine> {
        let machines = self.machines.read().await;
        machines.values().cloned().collect()
    }

    /// Get a machine by ID
    pub async fn get(&self, id: &MachineId) -> Option<Machine> {
        let machines = self.machines.read().await;
        machines.get(id).cloned()
    }

    /// Create a new machine
    pub async fn create(&self, request: CreateMachineRequest) -> Result<Machine> {
        let auth = match request.auth_method {
            CreateMachineAuthMethod::Agent => SshAuthMethod::Agent,
            CreateMachineAuthMethod::KeyFile { path, passphrase } => SshAuthMethod::KeyFile {
                path: PathBuf::from(path),
                passphrase,
            },
            CreateMachineAuthMethod::Password { password } => SshAuthMethod::Password { password },
        };

        let credentials = SshCredentials {
            username: request.username,
            auth,
        };

        let target = SshTarget::new(request.host, credentials).with_port(request.port);

        let mut machine = Machine::new(request.name, target);

        if let Some(desc) = request.description {
            machine.description = Some(desc);
        }

        machine.tags = request.tags;

        // Save to storage
        self.save_machine(&machine).await?;

        // Add to cache
        let mut machines = self.machines.write().await;
        machines.insert(machine.id.clone(), machine.clone());

        Ok(machine)
    }

    /// Update an existing machine
    pub async fn update(&self, id: &MachineId, request: UpdateMachineRequest) -> Result<Machine> {
        let mut machines = self.machines.write().await;

        let machine = machines
            .get_mut(id)
            .ok_or_else(|| NpError::MachineNotFound(id.to_string()))?;

        if let Some(name) = request.name {
            machine.name = name;
        }

        if let Some(description) = request.description {
            machine.description = Some(description);
        }

        if let Some(host) = request.host {
            machine.target.host = host;
        }

        if let Some(port) = request.port {
            machine.target.port = port;
        }

        if let Some(username) = request.username {
            machine.target.credentials.username = username;
        }

        if let Some(auth_method) = request.auth_method {
            machine.target.credentials.auth = match auth_method {
                CreateMachineAuthMethod::Agent => SshAuthMethod::Agent,
                CreateMachineAuthMethod::KeyFile { path, passphrase } => SshAuthMethod::KeyFile {
                    path: PathBuf::from(path),
                    passphrase,
                },
                CreateMachineAuthMethod::Password { password } => {
                    SshAuthMethod::Password { password }
                }
            };
        }

        if let Some(tags) = request.tags {
            machine.tags = tags;
        }

        machine.updated_at = Utc::now();

        let machine = machine.clone();

        // Save to storage
        drop(machines);
        self.save_machine(&machine).await?;

        Ok(machine)
    }

    /// Delete a machine
    pub async fn delete(&self, id: &MachineId) -> Result<()> {
        let mut machines = self.machines.write().await;

        if machines.remove(id).is_none() {
            return Err(NpError::MachineNotFound(id.to_string()));
        }

        drop(machines);
        self.delete_machine_file(id).await?;

        Ok(())
    }

    /// Test connection to a machine
    pub async fn test_connection(&self, id: &MachineId) -> Result<Machine> {
        let machine = self
            .get(id)
            .await
            .ok_or_else(|| NpError::MachineNotFound(id.to_string()))?;

        let (status, system_info) = match SshSession::connect(machine.target.clone()).await {
            Ok(session) => match session.test_connection().await {
                Ok(info) => {
                    let _ = session.close().await;
                    (MachineStatus::Online, Some(info))
                }
                Err(e) => {
                    tracing::warn!("Connection test failed for {}: {}", machine.name, e);
                    (MachineStatus::Offline, None)
                }
            },
            Err(NpError::SshAuth(_)) => (MachineStatus::AuthFailed, None),
            Err(_) => (MachineStatus::Offline, None),
        };

        // Update machine status
        let mut machines = self.machines.write().await;
        if let Some(m) = machines.get_mut(id) {
            m.update_status(status, system_info.clone());
            let updated = m.clone();
            drop(machines);
            self.save_machine(&updated).await?;
            return Ok(updated);
        }

        Err(NpError::MachineNotFound(id.to_string()))
    }

    /// Create an SSH session to a machine
    pub async fn connect(&self, id: &MachineId) -> Result<SshSession> {
        let machine = self
            .get(id)
            .await
            .ok_or_else(|| NpError::MachineNotFound(id.to_string()))?;

        SshSession::connect(machine.target).await
    }
}
