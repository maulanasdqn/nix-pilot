//! Deployment executor implementation

use crate::error::{NpError, Result};
use crate::nix::OutputLine;
use crate::ssh::SshSession;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use super::types::{DeployPhase, DeployRequest, GenerationInfo, RollbackRequest};

/// Executor for NixOS deployments
pub struct DeployExecutor {
    /// Default timeout for deployment (in seconds)
    timeout_secs: u64,
    /// Path to nixos-rebuild binary
    nixos_rebuild_path: String,
}

impl DeployExecutor {
    pub fn new() -> Self {
        // Try NixOS default path first
        let nixos_rebuild_path = if std::path::Path::new("/run/current-system/sw/bin/nixos-rebuild").exists() {
            "/run/current-system/sw/bin/nixos-rebuild".to_string()
        } else {
            "nixos-rebuild".to_string()
        };
        Self {
            timeout_secs: 3600, // 1 hour default
            nixos_rebuild_path,
        }
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Build the nixos-rebuild command arguments
    fn build_args(&self, request: &DeployRequest) -> Vec<String> {
        let mut args = Vec::new();

        // Action
        args.push(request.action.as_str().to_string());

        // Flake reference
        args.push("--flake".to_string());
        args.push(request.flake_ref.clone());

        // Target host
        let target_host = format!(
            "{}@{}",
            request.target.credentials.username, request.target.host
        );

        if request.target.port != 22 {
            // SSH options for non-standard port
            args.push("--target-host".to_string());
            args.push(target_host);
            args.push("--ssh-options".to_string());
            args.push(format!("-p {}", request.target.port));
        } else {
            args.push("--target-host".to_string());
            args.push(target_host);
        }

        // Build location
        if request.build_on_target {
            args.push("--build-host".to_string());
            args.push(format!(
                "{}@{}",
                request.target.credentials.username, request.target.host
            ));
        }

        // Substitutes
        if !request.use_substitutes {
            args.push("--no-substitutes".to_string());
        }

        // Extra arguments
        args.extend(request.extra_args.clone());

        args
    }

    /// Deploy a NixOS configuration with streaming output
    pub async fn deploy(
        &self,
        request: &DeployRequest,
        tx: mpsc::Sender<OutputLine>,
        phase_tx: mpsc::Sender<DeployPhase>,
    ) -> Result<i32> {
        let args = self.build_args(request);

        // Log the command
        let _ = tx
            .send(OutputLine::stdout(format!("$ nixos-rebuild {}", args.join(" "))))
            .await;

        // Send initial phase
        let _ = phase_tx.send(DeployPhase::Evaluating).await;

        let mut child = Command::new(&self.nixos_rebuild_path)
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
                command: format!("nixos-rebuild {}", args.join(" ")),
                timeout_secs: self.timeout_secs,
            })??;

        // Wait for output tasks to complete
        let _ = stdout_handle.await;
        let _ = stderr_handle.await;

        Ok(status.code().unwrap_or(-1))
    }

    /// Detect deployment phase from output line
    async fn detect_phase(line: &str, phase_tx: &mpsc::Sender<DeployPhase>) {
        let line_lower = line.to_lowercase();

        if line_lower.contains("evaluating") || line_lower.contains("evaluation") {
            let _ = phase_tx.send(DeployPhase::Evaluating).await;
        } else if line_lower.contains("building") || line_lower.contains("these derivations") {
            let _ = phase_tx.send(DeployPhase::Building).await;
        } else if line_lower.contains("copying") || line_lower.contains("copying paths") {
            let _ = phase_tx.send(DeployPhase::Copying).await;
        } else if line_lower.contains("activating") || line_lower.contains("switching to") {
            let _ = phase_tx.send(DeployPhase::Activating).await;
        } else if line_lower.contains("finished") || line_lower.contains("successfully activated") {
            let _ = phase_tx.send(DeployPhase::Complete).await;
        }
    }

    /// Rollback to a previous generation
    pub async fn rollback(
        &self,
        request: &RollbackRequest,
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<i32> {
        // Connect to the target
        let session = SshSession::connect(request.target.clone()).await?;

        // Determine the command based on whether we have a specific generation
        let command = if let Some(generation) = request.generation {
            format!(
                "sudo /nix/var/nix/profiles/system-{}-link/bin/switch-to-configuration switch",
                generation
            )
        } else {
            "sudo nixos-rebuild switch --rollback".to_string()
        };

        let _ = tx.send(OutputLine::stdout(format!("$ {}", command))).await;

        // Execute the rollback command
        let (output_tx, mut output_rx) = mpsc::channel(100);
        let tx_clone = tx.clone();

        // Forward output
        tokio::spawn(async move {
            while let Some(line) = output_rx.recv().await {
                let _ = tx_clone.send(line).await;
            }
        });

        let exit_code = session.execute_streaming(&command, output_tx).await?;

        Ok(exit_code as i32)
    }

    /// List available generations on the target
    pub async fn list_generations(&self, target: &crate::ssh::SshTarget) -> Result<Vec<GenerationInfo>> {
        let session = SshSession::connect(target.clone()).await?;

        // Get generation list
        let result = session
            .execute("nixos-rebuild list-generations --json 2>/dev/null || nix-env --list-generations -p /nix/var/nix/profiles/system")
            .await?;

        if result.exit_code != 0 {
            return Err(NpError::Deployment(format!(
                "Failed to list generations: {}",
                result.stderr
            )));
        }

        // Try to parse as JSON first (newer NixOS)
        if let Ok(generations) = serde_json::from_str::<Vec<GenerationInfo>>(&result.stdout) {
            return Ok(generations);
        }

        // Fall back to parsing text output
        let mut generations = Vec::new();
        for line in result.stdout.lines() {
            if let Some(generation) = Self::parse_generation_line(line) {
                generations.push(generation);
            }
        }

        Ok(generations)
    }

    /// Parse a generation line from nix-env output
    fn parse_generation_line(line: &str) -> Option<GenerationInfo> {
        // Format: "  42   2024-01-15 10:30:00   (current)"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let number: u32 = parts.first()?.parse().ok()?;
        let current = line.contains("(current)");

        // Try to parse date
        let date = if parts.len() >= 3 {
            let date_str = format!("{} {}", parts.get(1)?, parts.get(2)?);
            chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.and_utc())
                .unwrap_or_else(chrono::Utc::now)
        } else {
            chrono::Utc::now()
        };

        Some(GenerationInfo {
            number,
            date,
            current,
            nixos_version: None,
            kernel_version: None,
            configuration_revision: None,
        })
    }

    /// Get the current generation number
    pub async fn get_current_generation(&self, target: &crate::ssh::SshTarget) -> Result<u32> {
        let session = SshSession::connect(target.clone()).await?;

        let result = session
            .execute("readlink /nix/var/nix/profiles/system | grep -oE '[0-9]+' | tail -1")
            .await?;

        if result.exit_code != 0 {
            return Err(NpError::Deployment(
                "Failed to get current generation".to_string(),
            ));
        }

        result
            .stdout
            .trim()
            .parse()
            .map_err(|_| NpError::Deployment("Failed to parse generation number".to_string()))
    }
}

impl Default for DeployExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deploy::DeployAction;
    use crate::ssh::{SshCredentials, SshTarget};

    #[test]
    fn test_build_args() {
        let executor = DeployExecutor::new();

        let request = DeployRequest {
            target: SshTarget {
                host: "192.168.1.100".to_string(),
                port: 22,
                credentials: SshCredentials::with_agent("root"),
            },
            flake_ref: ".#nixosConfigurations.myhost".to_string(),
            action: DeployAction::Switch,
            build_on_target: false,
            use_substitutes: true,
            extra_args: vec![],
            rollback_on_failure: false,
        };

        let args = executor.build_args(&request);
        assert!(args.contains(&"switch".to_string()));
        assert!(args.contains(&"--flake".to_string()));
        assert!(args.contains(&"--target-host".to_string()));
    }

    #[test]
    fn test_parse_generation_line() {
        let line = "  42   2024-01-15 10:30:00   (current)";
        let generation = DeployExecutor::parse_generation_line(line).unwrap();
        assert_eq!(generation.number, 42);
        assert!(generation.current);
    }
}
