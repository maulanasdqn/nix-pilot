//! Systemd service manager implementation

use crate::error::{NpError, Result};
use crate::nix::OutputLine;
use crate::ssh::{SshSession, SshTarget};
use tokio::sync::mpsc;

use super::types::{
    LoadState, LogOptions, ServiceActionRequest, ServiceActionResult, ServiceInfo, ServiceState,
    ServiceStatus, ServiceSubState,
};

/// Manager for systemd services via SSH
pub struct SystemdManager;

impl SystemdManager {
    pub fn new() -> Self {
        Self
    }

    /// List all services on a target
    pub async fn list_services(&self, target: &SshTarget) -> Result<Vec<ServiceInfo>> {
        let session = SshSession::connect(target.clone()).await?;

        // Use systemctl to list all services with their status
        let result = session
            .execute("systemctl list-units --type=service --all --no-pager --plain --no-legend")
            .await?;

        if result.exit_code != 0 {
            return Err(NpError::ServiceOperation {
                service: "*".to_string(),
                message: format!("Failed to list services: {}", result.stderr),
            });
        }

        let mut services = Vec::new();
        for line in result.stdout.lines() {
            if let Some(info) = self.parse_service_line(line) {
                services.push(info);
            }
        }

        Ok(services)
    }

    /// Parse a line from systemctl list-units output
    fn parse_service_line(&self, line: &str) -> Option<ServiceInfo> {
        // Format: UNIT LOAD ACTIVE SUB DESCRIPTION
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }

        let name = parts[0].to_string();
        let load_state = LoadState::from_str(parts[1]);
        let active_state = ServiceState::from_str(parts[2]);
        let sub_state = ServiceSubState::from_str(parts[3]);
        let description = if parts.len() > 4 {
            Some(parts[4..].join(" "))
        } else {
            None
        };

        Some(ServiceInfo {
            name,
            description,
            load_state,
            active_state,
            sub_state,
            enabled: false, // Will be filled by get_service_status
            main_pid: None,
            memory_bytes: None,
            cpu_time_usec: None,
            started_at: None,
            unit_file_path: None,
        })
    }

    /// Get detailed status of a specific service
    pub async fn get_service_status(
        &self,
        target: &SshTarget,
        service: &str,
    ) -> Result<ServiceStatus> {
        let session = SshSession::connect(target.clone()).await?;

        // Normalize service name
        let service_name = if service.ends_with(".service") {
            service.to_string()
        } else {
            format!("{}.service", service)
        };

        // Get service properties
        let result = session
            .execute(&format!(
                "systemctl show '{}' --no-pager",
                service_name
            ))
            .await?;

        if result.exit_code != 0 {
            return Err(NpError::ServiceOperation {
                service: service_name.clone(),
                message: format!("Failed to get service status: {}", result.stderr),
            });
        }

        let info = self.parse_service_properties(&service_name, &result.stdout)?;

        // Get recent logs
        let logs_result = session
            .execute(&format!(
                "journalctl -u '{}' -n 20 --no-pager --output=short-iso",
                service_name
            ))
            .await?;

        let recent_logs: Vec<String> = logs_result
            .stdout
            .lines()
            .map(|s| s.to_string())
            .collect();

        // Get dependencies
        let deps_result = session
            .execute(&format!(
                "systemctl show '{}' --property=Requires,WantedBy,After,Before --no-pager",
                service_name
            ))
            .await?;

        let mut requires = Vec::new();
        let mut wanted_by = Vec::new();
        let mut after = Vec::new();
        let mut before = Vec::new();

        for line in deps_result.stdout.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let items: Vec<String> = value
                    .split_whitespace()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();

                match key {
                    "Requires" => requires = items,
                    "WantedBy" => wanted_by = items,
                    "After" => after = items,
                    "Before" => before = items,
                    _ => {}
                }
            }
        }

        Ok(ServiceStatus {
            info,
            recent_logs,
            triggered_by: None,
            triggers: None,
            requires,
            wanted_by,
            after,
            before,
        })
    }

    /// Parse systemctl show output into ServiceInfo
    fn parse_service_properties(&self, name: &str, output: &str) -> Result<ServiceInfo> {
        let mut info = ServiceInfo {
            name: name.to_string(),
            description: None,
            load_state: LoadState::Unknown,
            active_state: ServiceState::Unknown,
            sub_state: ServiceSubState::Unknown,
            enabled: false,
            main_pid: None,
            memory_bytes: None,
            cpu_time_usec: None,
            started_at: None,
            unit_file_path: None,
        };

        for line in output.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "Description" => info.description = Some(value.to_string()),
                    "LoadState" => info.load_state = LoadState::from_str(value),
                    "ActiveState" => info.active_state = ServiceState::from_str(value),
                    "SubState" => info.sub_state = ServiceSubState::from_str(value),
                    "UnitFileState" => {
                        info.enabled = value == "enabled" || value == "enabled-runtime"
                    }
                    "MainPID" => info.main_pid = value.parse().ok().filter(|&p| p > 0),
                    "MemoryCurrent" => {
                        info.memory_bytes = value.parse().ok().filter(|&m| m < u64::MAX)
                    }
                    "CPUUsageNSec" => {
                        info.cpu_time_usec = value.parse::<u64>().ok().map(|n| n / 1000)
                    }
                    "ActiveEnterTimestamp" => {
                        if !value.is_empty() && value != "n/a" {
                            // Parse timestamp like "Mon 2024-01-15 10:30:00 UTC"
                            info.started_at = chrono::DateTime::parse_from_str(
                                value,
                                "%a %Y-%m-%d %H:%M:%S %Z",
                            )
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc));
                        }
                    }
                    "FragmentPath" => {
                        if !value.is_empty() {
                            info.unit_file_path = Some(value.to_string())
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(info)
    }

    /// Perform an action on a service
    pub async fn service_action(
        &self,
        target: &SshTarget,
        request: &ServiceActionRequest,
    ) -> Result<ServiceActionResult> {
        let session = SshSession::connect(target.clone()).await?;

        // Normalize service name
        let service_name = if request.service.ends_with(".service") {
            request.service.clone()
        } else {
            format!("{}.service", request.service)
        };

        let command = format!(
            "sudo systemctl {} '{}'",
            request.action.as_str(),
            service_name
        );

        let result = session.execute(&command).await?;

        Ok(ServiceActionResult {
            service: service_name,
            action: request.action,
            success: result.exit_code == 0,
            message: if result.exit_code == 0 {
                format!("Successfully {}ed service", request.action.as_str())
            } else {
                result.stderr.trim().to_string()
            },
            exit_code: result.exit_code as i32,
        })
    }

    /// Get logs for a service
    pub async fn get_logs(
        &self,
        target: &SshTarget,
        service: &str,
        options: &LogOptions,
    ) -> Result<Vec<String>> {
        let session = SshSession::connect(target.clone()).await?;

        // Normalize service name
        let service_name = if service.ends_with(".service") {
            service.to_string()
        } else {
            format!("{}.service", service)
        };

        let mut args = vec![format!("-u '{}'", service_name), "--no-pager".to_string()];

        if let Some(lines) = options.lines {
            args.push(format!("-n {}", lines));
        } else {
            args.push("-n 100".to_string());
        }

        if let Some(since) = &options.since {
            args.push(format!("--since='{}'", since.format("%Y-%m-%d %H:%M:%S")));
        }

        if let Some(until) = &options.until {
            args.push(format!("--until='{}'", until.format("%Y-%m-%d %H:%M:%S")));
        }

        if let Some(output) = &options.output {
            args.push(format!("--output={}", output.as_str()));
        } else {
            args.push("--output=short-iso".to_string());
        }

        if options.full {
            args.push("--full".to_string());
        }

        let command = format!("journalctl {}", args.join(" "));
        let result = session.execute(&command).await?;

        if result.exit_code != 0 {
            return Err(NpError::ServiceOperation {
                service: service_name,
                message: format!("Failed to get logs: {}", result.stderr),
            });
        }

        Ok(result.stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Stream logs for a service (using journalctl -f)
    pub async fn stream_logs(
        &self,
        target: &SshTarget,
        service: &str,
        options: &LogOptions,
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<()> {
        let session = SshSession::connect(target.clone()).await?;

        // Normalize service name
        let service_name = if service.ends_with(".service") {
            service.to_string()
        } else {
            format!("{}.service", service)
        };

        let mut args = vec![
            format!("-u '{}'", service_name),
            "--no-pager".to_string(),
            "-f".to_string(), // Follow
        ];

        if let Some(lines) = options.lines {
            args.push(format!("-n {}", lines));
        } else {
            args.push("-n 50".to_string());
        }

        if let Some(output) = &options.output {
            args.push(format!("--output={}", output.as_str()));
        } else {
            args.push("--output=short-iso".to_string());
        }

        let command = format!("journalctl {}", args.join(" "));

        // Execute with streaming
        session.execute_streaming(&command, tx).await?;

        Ok(())
    }

    /// Get system-wide journal logs
    pub async fn get_system_logs(
        &self,
        target: &SshTarget,
        options: &LogOptions,
    ) -> Result<Vec<String>> {
        let session = SshSession::connect(target.clone()).await?;

        let mut args = vec!["--no-pager".to_string()];

        if let Some(lines) = options.lines {
            args.push(format!("-n {}", lines));
        } else {
            args.push("-n 100".to_string());
        }

        if let Some(since) = &options.since {
            args.push(format!("--since='{}'", since.format("%Y-%m-%d %H:%M:%S")));
        }

        if let Some(output) = &options.output {
            args.push(format!("--output={}", output.as_str()));
        } else {
            args.push("--output=short-iso".to_string());
        }

        let command = format!("journalctl {}", args.join(" "));
        let result = session.execute(&command).await?;

        if result.exit_code != 0 {
            return Err(NpError::ServiceOperation {
                service: "system".to_string(),
                message: format!("Failed to get system logs: {}", result.stderr),
            });
        }

        Ok(result.stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Check if a service exists
    pub async fn service_exists(&self, target: &SshTarget, service: &str) -> Result<bool> {
        let session = SshSession::connect(target.clone()).await?;

        let service_name = if service.ends_with(".service") {
            service.to_string()
        } else {
            format!("{}.service", service)
        };

        let result = session
            .execute(&format!(
                "systemctl list-unit-files '{}' --no-pager --no-legend | wc -l",
                service_name
            ))
            .await?;

        Ok(result.stdout.trim() != "0")
    }

    /// Get failed services
    pub async fn get_failed_services(&self, target: &SshTarget) -> Result<Vec<ServiceInfo>> {
        let session = SshSession::connect(target.clone()).await?;

        let result = session
            .execute("systemctl list-units --type=service --state=failed --no-pager --plain --no-legend")
            .await?;

        if result.exit_code != 0 {
            return Err(NpError::ServiceOperation {
                service: "*".to_string(),
                message: format!("Failed to list failed services: {}", result.stderr),
            });
        }

        let mut services = Vec::new();
        for line in result.stdout.lines() {
            if let Some(info) = self.parse_service_line(line) {
                services.push(info);
            }
        }

        Ok(services)
    }
}

impl Default for SystemdManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_service_line() {
        let manager = SystemdManager::new();

        let line = "nginx.service loaded active running A high performance web server";
        let info = manager.parse_service_line(line).unwrap();

        assert_eq!(info.name, "nginx.service");
        assert_eq!(info.load_state, LoadState::Loaded);
        assert_eq!(info.active_state, ServiceState::Active);
        assert_eq!(info.sub_state, ServiceSubState::Running);
        assert_eq!(
            info.description,
            Some("A high performance web server".to_string())
        );
    }
}
