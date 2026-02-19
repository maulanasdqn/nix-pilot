use crate::error::{NpError, Result};
use crate::nix::OutputLine;
use russh::client::{self, Handle};
use russh::keys::ssh_key::PublicKey;
use russh::keys::{agent::client::AgentClient, ssh_key::private::PrivateKey, PrivateKeyWithHashAlg};
use russh::ChannelMsg;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

use super::auth::{SshAuthMethod, SshCredentials};

/// SSH connection target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshTarget {
    /// Hostname or IP address
    pub host: String,

    /// SSH port (default: 22)
    #[serde(default = "default_port")]
    pub port: u16,

    /// Connection credentials
    pub credentials: SshCredentials,
}

fn default_port() -> u16 {
    22
}

impl SshTarget {
    /// Create a new SSH target
    pub fn new(host: impl Into<String>, credentials: SshCredentials) -> Self {
        Self {
            host: host.into(),
            port: 22,
            credentials,
        }
    }

    /// Set a custom port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

/// Result of a command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub exit_code: u32,
    pub stdout: String,
    pub stderr: String,
}

/// SSH client handler for russh
struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
        // TODO: Implement proper host key verification
        // For now, accept all keys (like StrictHostKeyChecking=no)
        Ok(true)
    }
}

/// An active SSH session
pub struct SshSession {
    handle: Handle<ClientHandler>,
    target: SshTarget,
}

impl SshSession {
    /// Connect to an SSH target
    pub async fn connect(target: SshTarget) -> Result<Self> {
        let config = client::Config::default();
        let config = Arc::new(config);

        let addr = format!("{}:{}", target.host, target.port);

        let mut handle = client::connect(config, &addr, ClientHandler)
            .await
            .map_err(|e| NpError::SshConnection {
                host: target.host.clone(),
                message: e.to_string(),
            })?;

        // Authenticate based on method
        let authenticated = match &target.credentials.auth {
            SshAuthMethod::Agent => {
                Self::auth_with_agent(&mut handle, &target.credentials.username).await?
            }
            SshAuthMethod::KeyFile { path, passphrase } => {
                Self::auth_with_key_file(
                    &mut handle,
                    &target.credentials.username,
                    path,
                    passphrase.as_deref(),
                )
                .await?
            }
            SshAuthMethod::Password { password } => {
                let result = handle
                    .authenticate_password(&target.credentials.username, password)
                    .await
                    .map_err(|e| NpError::SshAuth(e.to_string()))?;
                result.success()
            }
        };

        if !authenticated {
            return Err(NpError::SshAuth("Authentication failed".to_string()));
        }

        Ok(Self { handle, target })
    }

    /// Authenticate using SSH agent
    async fn auth_with_agent(
        handle: &mut Handle<ClientHandler>,
        username: &str,
    ) -> Result<bool> {
        // Try to connect to SSH agent
        let mut agent: AgentClient<tokio::net::UnixStream> = AgentClient::connect_env()
            .await
            .map_err(|e| NpError::SshAuth(format!("Failed to connect to SSH agent: {}", e)))?;

        let identities = agent
            .request_identities()
            .await
            .map_err(|e| NpError::SshAuth(format!("Failed to get agent identities: {}", e)))?;

        // Try each identity
        for pubkey in identities {
            // Use authenticate_publickey_with which takes a signer (the agent)
            let result = handle
                .authenticate_publickey_with(username, pubkey, None, &mut agent)
                .await;

            match result {
                Ok(auth_result) if auth_result.success() => return Ok(true),
                _ => continue,
            }
        }

        Ok(false)
    }

    /// Authenticate using a key file
    async fn auth_with_key_file(
        handle: &mut Handle<ClientHandler>,
        username: &str,
        path: &PathBuf,
        passphrase: Option<&str>,
    ) -> Result<bool> {
        let key_data = std::fs::read_to_string(path)
            .map_err(|e| NpError::SshAuth(format!("Failed to read key file: {}", e)))?;

        let key = if let Some(pass) = passphrase {
            PrivateKey::from_openssh(&key_data)
                .and_then(|k| k.decrypt(pass.as_bytes()))
                .map_err(|e| NpError::SshAuth(format!("Failed to decrypt key: {}", e)))?
        } else {
            PrivateKey::from_openssh(&key_data)
                .map_err(|e| NpError::SshAuth(format!("Failed to parse key: {}", e)))?
        };

        let key_with_hash = PrivateKeyWithHashAlg::new(Arc::new(key), None);

        let auth_result = handle
            .authenticate_publickey(username, key_with_hash)
            .await
            .map_err(|e| NpError::SshAuth(e.to_string()))?;

        Ok(auth_result.success())
    }

    /// Get the target this session is connected to
    pub fn target(&self) -> &SshTarget {
        &self.target
    }

    /// Execute a command and return the result
    pub async fn execute(&self, command: &str) -> Result<CommandResult> {
        let mut channel = self
            .handle
            .channel_open_session()
            .await
            .map_err(|e| NpError::SshConnection {
                host: self.target.host.clone(),
                message: e.to_string(),
            })?;

        channel
            .exec(true, command.as_bytes())
            .await
            .map_err(|e| NpError::SshConnection {
                host: self.target.host.clone(),
                message: e.to_string(),
            })?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit_code = 0u32;

        loop {
            match channel.wait().await {
                Some(ChannelMsg::Data { data }) => {
                    stdout.extend_from_slice(&data);
                }
                Some(ChannelMsg::ExtendedData { data, ext }) => {
                    if ext == 1 {
                        // stderr
                        stderr.extend_from_slice(&data);
                    }
                }
                Some(ChannelMsg::ExitStatus { exit_status }) => {
                    exit_code = exit_status;
                }
                Some(ChannelMsg::Eof) | None => break,
                _ => {}
            }
        }

        Ok(CommandResult {
            exit_code,
            stdout: String::from_utf8_lossy(&stdout).to_string(),
            stderr: String::from_utf8_lossy(&stderr).to_string(),
        })
    }

    /// Execute a command with streaming output
    pub async fn execute_streaming(
        &self,
        command: &str,
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<u32> {
        let mut channel = self
            .handle
            .channel_open_session()
            .await
            .map_err(|e| NpError::SshConnection {
                host: self.target.host.clone(),
                message: e.to_string(),
            })?;

        channel
            .exec(true, command.as_bytes())
            .await
            .map_err(|e| NpError::SshConnection {
                host: self.target.host.clone(),
                message: e.to_string(),
            })?;

        let mut exit_code = 0u32;
        let mut stdout_buffer = Vec::new();
        let mut stderr_buffer = Vec::new();

        loop {
            match channel.wait().await {
                Some(ChannelMsg::Data { data }) => {
                    stdout_buffer.extend_from_slice(&data);
                    // Process complete lines
                    while let Some(pos) = stdout_buffer.iter().position(|&b| b == b'\n') {
                        let line = String::from_utf8_lossy(&stdout_buffer[..pos]).to_string();
                        stdout_buffer.drain(..=pos);
                        if tx.send(OutputLine::stdout(line)).await.is_err() {
                            break;
                        }
                    }
                }
                Some(ChannelMsg::ExtendedData { data, ext }) => {
                    if ext == 1 {
                        stderr_buffer.extend_from_slice(&data);
                        while let Some(pos) = stderr_buffer.iter().position(|&b| b == b'\n') {
                            let line = String::from_utf8_lossy(&stderr_buffer[..pos]).to_string();
                            stderr_buffer.drain(..=pos);
                            if tx.send(OutputLine::stderr(line)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                Some(ChannelMsg::ExitStatus { exit_status }) => {
                    exit_code = exit_status;
                }
                Some(ChannelMsg::Eof) | None => break,
                _ => {}
            }
        }

        // Send any remaining data
        if !stdout_buffer.is_empty() {
            let line = String::from_utf8_lossy(&stdout_buffer).to_string();
            let _ = tx.send(OutputLine::stdout(line)).await;
        }
        if !stderr_buffer.is_empty() {
            let line = String::from_utf8_lossy(&stderr_buffer).to_string();
            let _ = tx.send(OutputLine::stderr(line)).await;
        }

        Ok(exit_code)
    }

    /// Upload a file to the remote host
    pub async fn upload_file(&self, local_path: &std::path::Path, remote_path: &str) -> Result<()> {
        let content = tokio::fs::read(local_path).await?;

        let mut channel = self
            .handle
            .channel_open_session()
            .await
            .map_err(|e| NpError::SshConnection {
                host: self.target.host.clone(),
                message: e.to_string(),
            })?;

        // Use cat to write the file
        let command = format!("cat > {}", shell_escape(remote_path));
        channel
            .exec(true, command.as_bytes())
            .await
            .map_err(|e| NpError::SshConnection {
                host: self.target.host.clone(),
                message: e.to_string(),
            })?;

        channel
            .data(&content[..])
            .await
            .map_err(|e| NpError::SshConnection {
                host: self.target.host.clone(),
                message: e.to_string(),
            })?;

        channel.eof().await.map_err(|e| NpError::SshConnection {
            host: self.target.host.clone(),
            message: e.to_string(),
        })?;

        // Wait for completion
        loop {
            match channel.wait().await {
                Some(ChannelMsg::Eof) | None => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Download a file from the remote host
    pub async fn download_file(
        &self,
        remote_path: &str,
        local_path: &std::path::Path,
    ) -> Result<()> {
        let result = self
            .execute(&format!("cat {}", shell_escape(remote_path)))
            .await?;

        if result.exit_code != 0 {
            return Err(NpError::SshConnection {
                host: self.target.host.clone(),
                message: format!("Failed to read remote file: {}", result.stderr),
            });
        }

        tokio::fs::write(local_path, result.stdout.as_bytes()).await?;

        Ok(())
    }

    /// Test the connection by running a simple command
    pub async fn test_connection(&self) -> Result<String> {
        let result = self.execute("uname -a").await?;
        if result.exit_code != 0 {
            return Err(NpError::SshConnection {
                host: self.target.host.clone(),
                message: result.stderr,
            });
        }
        Ok(result.stdout.trim().to_string())
    }

    /// Close the SSH session
    pub async fn close(self) -> Result<()> {
        self.handle
            .disconnect(russh::Disconnect::ByApplication, "", "")
            .await
            .map_err(|e| NpError::SshConnection {
                host: self.target.host.clone(),
                message: e.to_string(),
            })?;
        Ok(())
    }
}

/// Escape a string for safe use in shell commands
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape("hello"), "'hello'");
        assert_eq!(shell_escape("hello world"), "'hello world'");
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
    }
}
