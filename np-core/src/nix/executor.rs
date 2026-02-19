use crate::error::{NpError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

/// Output stream type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// A single line of command output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputLine {
    pub stream: OutputStream,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl OutputLine {
    pub fn stdout(content: impl Into<String>) -> Self {
        Self {
            stream: OutputStream::Stdout,
            content: content.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn stderr(content: impl Into<String>) -> Self {
        Self {
            stream: OutputStream::Stderr,
            content: content.into(),
            timestamp: Utc::now(),
        }
    }
}

/// Result of a completed command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Executor for running nix commands
#[derive(Debug, Clone)]
pub struct NixExecutor {
    nix_path: PathBuf,
    timeout_secs: u64,
}

impl NixExecutor {
    /// Create a new executor with the given nix path
    pub fn new(nix_path: PathBuf) -> Self {
        Self {
            nix_path,
            timeout_secs: 300,
        }
    }

    /// Create executor that finds nix in PATH
    pub fn from_path() -> Result<Self> {
        Ok(Self::new(PathBuf::from("nix")))
    }

    /// Set the command timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Execute a nix command and wait for completion
    pub async fn execute(&self, args: &[&str]) -> Result<CommandOutput> {
        let output = Command::new(&self.nix_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let stdout = String::from_utf8(output.stdout).map_err(|_| NpError::InvalidUtf8)?;
        let stderr = String::from_utf8(output.stderr).map_err(|_| NpError::InvalidUtf8)?;
        let exit_code = output.status.code().unwrap_or(-1);

        Ok(CommandOutput {
            exit_code,
            stdout,
            stderr,
        })
    }

    /// Execute a nix command with streaming output
    pub async fn execute_streaming(
        &self,
        args: &[&str],
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<i32> {
        let mut child = Command::new(&self.nix_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().expect("stdout not captured");
        let stderr = child.stderr.take().expect("stderr not captured");

        let stdout_tx = tx.clone();
        let stderr_tx = tx;

        // Spawn tasks to read stdout and stderr concurrently
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

        // Wait for the process to complete with timeout
        let timeout = std::time::Duration::from_secs(self.timeout_secs);
        let status = tokio::time::timeout(timeout, child.wait()).await.map_err(|_| {
            NpError::NixTimeout {
                command: format!("nix {}", args.join(" ")),
                timeout_secs: self.timeout_secs,
            }
        })??;

        // Wait for output tasks to complete
        let _ = stdout_handle.await;
        let _ = stderr_handle.await;

        Ok(status.code().unwrap_or(-1))
    }

    /// Check if nix is available
    pub async fn check_available(&self) -> Result<String> {
        let output = self.execute(&["--version"]).await?;
        if output.exit_code != 0 {
            return Err(NpError::NixNotFound {
                path: Some(self.nix_path.clone()),
            });
        }
        Ok(output.stdout.trim().to_string())
    }

    /// Get flake metadata as JSON
    pub async fn flake_metadata(&self, flake_ref: &str) -> Result<serde_json::Value> {
        let output = self
            .execute(&["flake", "metadata", "--json", flake_ref])
            .await?;

        if output.exit_code != 0 {
            return Err(NpError::NixCommand {
                code: output.exit_code,
                stderr: output.stderr,
            });
        }

        let json: serde_json::Value = serde_json::from_str(&output.stdout)?;
        Ok(json)
    }

    /// Show flake outputs as JSON
    pub async fn flake_show(&self, flake_ref: &str) -> Result<serde_json::Value> {
        let output = self
            .execute(&["flake", "show", "--json", flake_ref])
            .await?;

        if output.exit_code != 0 {
            return Err(NpError::NixCommand {
                code: output.exit_code,
                stderr: output.stderr,
            });
        }

        let json: serde_json::Value = serde_json::from_str(&output.stdout)?;
        Ok(json)
    }

    /// Update flake lock file with streaming output
    pub async fn flake_update(
        &self,
        flake_path: &str,
        input: Option<&str>,
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<i32> {
        let mut args = vec!["flake", "update"];
        if let Some(input_name) = input {
            args.push(input_name);
        }
        args.push("--flake");
        args.push(flake_path);

        self.execute_streaming(&args, tx).await
    }

    /// Build a flake output with streaming output
    pub async fn build(
        &self,
        flake_ref: &str,
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<i32> {
        let args = vec!["build", flake_ref, "--print-build-logs"];
        self.execute_streaming(&args, tx).await
    }

    /// Evaluate a nix expression
    pub async fn eval(&self, expr: &str) -> Result<serde_json::Value> {
        let output = self.execute(&["eval", "--json", "--expr", expr]).await?;

        if output.exit_code != 0 {
            return Err(NpError::NixCommand {
                code: output.exit_code,
                stderr: output.stderr,
            });
        }

        let json: serde_json::Value = serde_json::from_str(&output.stdout)?;
        Ok(json)
    }

    /// Run garbage collection with streaming output
    pub async fn gc(&self, delete_older_than: Option<&str>, tx: mpsc::Sender<OutputLine>) -> Result<i32> {
        let mut args = vec!["store", "gc", "-v"];
        if let Some(older) = delete_older_than {
            args.push("--delete-older-than");
            args.push(older);
        }
        self.execute_streaming(&args, tx).await
    }

    /// Get nix store info
    pub async fn store_info(&self) -> Result<StoreInfo> {
        let output = self.execute(&["store", "info", "--json"]).await?;

        if output.exit_code != 0 {
            return Err(NpError::NixCommand {
                code: output.exit_code,
                stderr: output.stderr,
            });
        }

        let json: serde_json::Value = serde_json::from_str(&output.stdout)?;

        Ok(StoreInfo {
            url: json["url"].as_str().unwrap_or("").to_string(),
            version: json["version"].as_str().map(|s| s.to_string()),
            trusted: json["trusted"].as_bool(),
        })
    }

    /// Get info about a store path
    pub async fn path_info(&self, path: &str) -> Result<PathInfo> {
        let output = self.execute(&["path-info", "--json", path]).await?;

        if output.exit_code != 0 {
            return Err(NpError::NixCommand {
                code: output.exit_code,
                stderr: output.stderr,
            });
        }

        let json: serde_json::Value = serde_json::from_str(&output.stdout)?;

        // path-info returns an array of path info objects
        let info = json
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| NpError::Other("No path info returned".into()))?;

        Ok(PathInfo {
            path: info["path"].as_str().unwrap_or("").to_string(),
            nar_size: info["narSize"].as_u64().unwrap_or(0),
            nar_hash: info["narHash"].as_str().unwrap_or("").to_string(),
            deriver: info["deriver"].as_str().map(|s| s.to_string()),
            references: info["references"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            registration_time: info["registrationTime"].as_u64(),
            signatures: info["signatures"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            valid: info["valid"].as_bool().unwrap_or(true),
        })
    }

    /// Run flake check with streaming output
    pub async fn flake_check(
        &self,
        flake_ref: &str,
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<i32> {
        let args = vec!["flake", "check", flake_ref, "-L"];
        self.execute_streaming(&args, tx).await
    }

    /// Optimise the nix store (deduplication) with streaming output
    pub async fn store_optimise(&self, tx: mpsc::Sender<OutputLine>) -> Result<i32> {
        let args = vec!["store", "optimise", "-v"];
        self.execute_streaming(&args, tx).await
    }

    /// Verify store integrity with streaming output
    pub async fn store_verify(
        &self,
        paths: Option<&[&str]>,
        check_contents: bool,
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<i32> {
        let mut args = vec!["store", "verify"];
        if check_contents {
            args.push("--check-contents");
        }
        if let Some(ps) = paths {
            args.extend(ps.iter());
        } else {
            args.push("--all");
        }
        self.execute_streaming(&args, tx).await
    }

    /// Show derivation details
    pub async fn derivation_show(&self, path: &str) -> Result<serde_json::Value> {
        let output = self.execute(&["derivation", "show", path]).await?;

        if output.exit_code != 0 {
            return Err(NpError::NixCommand {
                code: output.exit_code,
                stderr: output.stderr,
            });
        }

        let json: serde_json::Value = serde_json::from_str(&output.stdout)?;
        Ok(json)
    }

    /// Show why one path depends on another
    pub async fn why_depends(&self, package: &str, dependency: &str) -> Result<Vec<String>> {
        let output = self
            .execute(&["why-depends", package, dependency])
            .await?;

        if output.exit_code != 0 {
            return Err(NpError::NixCommand {
                code: output.exit_code,
                stderr: output.stderr,
            });
        }

        Ok(output.stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Search for packages
    pub async fn search(&self, query: &str, flake_ref: Option<&str>) -> Result<Vec<SearchResult>> {
        let flake = flake_ref.unwrap_or("nixpkgs");
        let output = self
            .execute(&["search", flake, query, "--json"])
            .await?;

        if output.exit_code != 0 {
            return Err(NpError::NixCommand {
                code: output.exit_code,
                stderr: output.stderr,
            });
        }

        let json: serde_json::Value = serde_json::from_str(&output.stdout)?;
        let mut results = Vec::new();

        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                results.push(SearchResult {
                    attr_path: key.clone(),
                    name: value["pname"].as_str().unwrap_or("").to_string(),
                    version: value["version"].as_str().unwrap_or("").to_string(),
                    description: value["description"].as_str().map(|s| s.to_string()),
                });
            }
        }

        Ok(results)
    }

    /// Get the closure size of a path
    pub async fn closure_size(&self, path: &str) -> Result<ClosureInfo> {
        let output = self
            .execute(&["path-info", "--json", "-S", "-s", path])
            .await?;

        if output.exit_code != 0 {
            return Err(NpError::NixCommand {
                code: output.exit_code,
                stderr: output.stderr,
            });
        }

        let json: serde_json::Value = serde_json::from_str(&output.stdout)?;
        let info = json
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| NpError::Other("No path info returned".into()))?;

        Ok(ClosureInfo {
            path: info["path"].as_str().unwrap_or("").to_string(),
            nar_size: info["narSize"].as_u64().unwrap_or(0),
            closure_size: info["closureSize"].as_u64().unwrap_or(0),
        })
    }

    /// List generations for a profile
    pub async fn profile_list(&self, profile: Option<&str>) -> Result<Vec<ProfileGeneration>> {
        let profile_path = profile.unwrap_or("/nix/var/nix/profiles/system");
        let output = self
            .execute(&["profile", "history", "--profile", profile_path, "--json"])
            .await?;

        if output.exit_code != 0 {
            // Fall back to non-JSON format if JSON isn't supported
            return self.profile_list_legacy(profile_path).await;
        }

        let json: serde_json::Value = serde_json::from_str(&output.stdout)?;
        let mut generations = Vec::new();

        if let Some(arr) = json.as_array() {
            for item in arr {
                generations.push(ProfileGeneration {
                    number: item["generation"].as_u64().unwrap_or(0) as u32,
                    date: item["date"].as_str().map(|s| s.to_string()),
                    current: item["current"].as_bool().unwrap_or(false),
                });
            }
        }

        Ok(generations)
    }

    /// Legacy profile list using profile-list command
    async fn profile_list_legacy(&self, profile: &str) -> Result<Vec<ProfileGeneration>> {
        // Try to list profile generations by reading the profile directory
        let output = Command::new("ls")
            .args(["-la", profile])
            .output()
            .await?;

        // Just return empty for now if the command fails
        if !output.status.success() {
            return Ok(Vec::new());
        }

        // Parse basic info
        Ok(Vec::new())
    }

    /// Delete old generations
    pub async fn profile_delete_generations(
        &self,
        profile: Option<&str>,
        generations: &[u32],
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<i32> {
        let profile_path = profile.unwrap_or("/nix/var/nix/profiles/system");
        let mut args = vec!["profile", "wipe-history", "--profile", profile_path];

        // Build generation args
        let gen_strs: Vec<String> = generations.iter().map(|g| g.to_string()).collect();
        for g in &gen_strs {
            args.push("--older-than");
            args.push(g);
        }

        self.execute_streaming(&args, tx).await
    }

    /// Repair a store path
    pub async fn store_repair(&self, path: &str, tx: mpsc::Sender<OutputLine>) -> Result<i32> {
        let args = vec!["store", "repair", path];
        self.execute_streaming(&args, tx).await
    }
}

/// Information about the nix store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreInfo {
    pub url: String,
    pub version: Option<String>,
    pub trusted: Option<bool>,
}

/// Information about a store path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathInfo {
    pub path: String,
    pub nar_size: u64,
    pub nar_hash: String,
    pub deriver: Option<String>,
    pub references: Vec<String>,
    pub registration_time: Option<u64>,
    pub signatures: Vec<String>,
    pub valid: bool,
}

/// Information about a closure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureInfo {
    pub path: String,
    pub nar_size: u64,
    pub closure_size: u64,
}

/// Package search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub attr_path: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

/// Profile generation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileGeneration {
    pub number: u32,
    pub date: Option<String>,
    pub current: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nix_version() {
        let executor = NixExecutor::from_path().unwrap();
        // This test will only pass if nix is installed
        if let Ok(version) = executor.check_available().await {
            assert!(version.contains("nix"));
        }
    }
}
