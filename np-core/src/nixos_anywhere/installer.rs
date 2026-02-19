//! nixos-anywhere installer implementation

use crate::error::{NpError, Result};
use crate::nix::OutputLine;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use super::types::{DiskoMode, InstallPhase, InstallRequest};

/// Installer for running nixos-anywhere
pub struct NixosAnywhereInstaller {
    /// Path to nixos-anywhere executable
    nixos_anywhere_path: PathBuf,
    /// Default timeout for installation (in seconds)
    timeout_secs: u64,
}

impl NixosAnywhereInstaller {
    pub fn new() -> Self {
        Self {
            nixos_anywhere_path: PathBuf::from("nixos-anywhere"),
            timeout_secs: 3600, // 1 hour default
        }
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.nixos_anywhere_path = path;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Check if nixos-anywhere is available
    pub async fn check_available(&self) -> Result<String> {
        let output = Command::new(&self.nixos_anywhere_path)
            .arg("--help")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(NpError::NixNotFound {
                path: Some(self.nixos_anywhere_path.clone()),
            });
        }

        Ok("nixos-anywhere available".to_string())
    }

    /// Build the command arguments for an installation
    fn build_args(&self, request: &InstallRequest) -> Vec<String> {
        let mut args = Vec::new();

        // Flake reference
        args.push("--flake".to_string());
        args.push(request.flake_ref.clone());

        // SSH target
        let ssh_target = format!(
            "{}@{}",
            request.target.credentials.username,
            request.target.host
        );

        // Port if not default
        if request.target.port != 22 {
            args.push("--ssh-port".to_string());
            args.push(request.target.port.to_string());
        }

        // Kexec option
        if !request.kexec {
            args.push("--no-reboot".to_string());
        }

        // Disko mode
        if let Some(disko_mode) = &request.disko_mode {
            match disko_mode {
                DiskoMode::Disko => {}
                DiskoMode::DiskoFormat => {
                    args.push("--disk-encryption-keys".to_string());
                }
                DiskoMode::DiskoMount => {
                    args.push("--disko-mode".to_string());
                    args.push("mount".to_string());
                }
                DiskoMode::None => {
                    args.push("--no-disko".to_string());
                }
            }
        }

        // Build host
        if request.build_host.is_some() {
            args.push("--build-on-remote".to_string());
        }

        // Extra arguments
        args.extend(request.extra_args.clone());

        // Target (must be last)
        args.push(ssh_target);

        args
    }

    /// Run nixos-anywhere installation with streaming output
    pub async fn install(
        &self,
        request: &InstallRequest,
        tx: mpsc::Sender<OutputLine>,
        phase_tx: mpsc::Sender<InstallPhase>,
    ) -> Result<i32> {
        let args = self.build_args(request);

        // Log the command
        let _ = tx
            .send(OutputLine::stdout(format!(
                "$ {} {}",
                self.nixos_anywhere_path.display(),
                args.join(" ")
            )))
            .await;

        // Send initial phase
        let _ = phase_tx.send(InstallPhase::Connecting).await;

        let mut child = Command::new(&self.nixos_anywhere_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().expect("stdout not captured");
        let stderr = child.stderr.take().expect("stderr not captured");

        let stdout_tx = tx.clone();
        let phase_tx_stdout = phase_tx.clone();
        let stderr_tx = tx;
        let phase_tx_stderr = phase_tx;

        // Spawn tasks to read stdout and stderr concurrently
        let stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                // Detect phase from output
                Self::detect_phase(&line, &phase_tx_stdout).await;
                if stdout_tx.send(OutputLine::stdout(&line)).await.is_err() {
                    break;
                }
            }
        });

        let stderr_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                // Detect phase from output
                Self::detect_phase(&line, &phase_tx_stderr).await;
                if stderr_tx.send(OutputLine::stderr(&line)).await.is_err() {
                    break;
                }
            }
        });

        // Wait for the process to complete with timeout
        let timeout = std::time::Duration::from_secs(self.timeout_secs);
        let status = tokio::time::timeout(timeout, child.wait())
            .await
            .map_err(|_| NpError::NixTimeout {
                command: format!(
                    "{} {}",
                    self.nixos_anywhere_path.display(),
                    args.join(" ")
                ),
                timeout_secs: self.timeout_secs,
            })??;

        // Wait for output tasks to complete
        let _ = stdout_handle.await;
        let _ = stderr_handle.await;

        Ok(status.code().unwrap_or(-1))
    }

    /// Detect installation phase from output line
    async fn detect_phase(line: &str, phase_tx: &mpsc::Sender<InstallPhase>) {
        let line_lower = line.to_lowercase();

        // Detect phases based on common nixos-anywhere output patterns
        if line_lower.contains("connecting") || line_lower.contains("ssh") {
            let _ = phase_tx.send(InstallPhase::Connecting).await;
        } else if line_lower.contains("kexec") || line_lower.contains("booting") {
            let _ = phase_tx.send(InstallPhase::Kexec).await;
        } else if line_lower.contains("disko") || line_lower.contains("formatting") || line_lower.contains("partitioning") {
            let _ = phase_tx.send(InstallPhase::Formatting).await;
        } else if line_lower.contains("nixos-install") || line_lower.contains("installing") || line_lower.contains("building") {
            let _ = phase_tx.send(InstallPhase::Installing).await;
        } else if line_lower.contains("reboot") {
            let _ = phase_tx.send(InstallPhase::Rebooting).await;
        } else if line_lower.contains("success") || line_lower.contains("complete") {
            let _ = phase_tx.send(InstallPhase::Complete).await;
        }
    }

    /// Run VM test (dry run in a VM)
    pub async fn vm_test(
        &self,
        request: &InstallRequest,
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<i32> {
        // For VM test, we build the system and run it in a VM
        // This uses `nix run .#nixosConfigurations.xxx.config.system.build.vm`

        let flake_parts: Vec<&str> = request.flake_ref.splitn(2, '#').collect();
        let (flake_path, config_name) = if flake_parts.len() == 2 {
            (flake_parts[0], flake_parts[1].trim_start_matches("nixosConfigurations."))
        } else {
            return Err(NpError::InvalidFlakeRef(
                "Flake reference must include nixosConfigurations".to_string(),
            ));
        };

        let vm_ref = format!(
            "{}#nixosConfigurations.{}.config.system.build.vm",
            flake_path, config_name
        );

        let _ = tx
            .send(OutputLine::stdout(format!("$ nix run {}", vm_ref)))
            .await;

        let mut child = Command::new("nix")
            .args(["run", &vm_ref])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().expect("stdout not captured");
        let stderr = child.stderr.take().expect("stderr not captured");

        let stdout_tx = tx.clone();
        let stderr_tx = tx;

        let stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if stdout_tx.send(OutputLine::stdout(line)).await.is_err() {
                    break;
                }
            }
        });

        let stderr_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if stderr_tx.send(OutputLine::stderr(line)).await.is_err() {
                    break;
                }
            }
        });

        let status = child.wait().await?;

        let _ = stdout_handle.await;
        let _ = stderr_handle.await;

        Ok(status.code().unwrap_or(-1))
    }
}

impl Default for NixosAnywhereInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::{SshAuthMethod, SshCredentials, SshTarget};

    #[test]
    fn test_build_args() {
        let installer = NixosAnywhereInstaller::new();

        let request = InstallRequest {
            target: SshTarget {
                host: "192.168.1.100".to_string(),
                port: 22,
                credentials: SshCredentials {
                    username: "root".to_string(),
                    auth_method: SshAuthMethod::Agent,
                },
            },
            flake_ref: "github:user/repo#nixosConfigurations.myhost".to_string(),
            vm_test: false,
            extra_args: vec![],
            build_host: None,
            kexec: true,
            wipe: false,
            disko_mode: None,
        };

        let args = installer.build_args(&request);
        assert!(args.contains(&"--flake".to_string()));
        assert!(args.contains(&"github:user/repo#nixosConfigurations.myhost".to_string()));
        assert!(args.contains(&"root@192.168.1.100".to_string()));
    }
}
