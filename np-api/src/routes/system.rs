//! Local system management routes
//!
//! These routes manage the local NixOS/nix-darwin system (the machine running this API).

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// Detected system type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SystemType {
    NixOS,
    NixDarwin,
    Unknown,
}

/// Detect the system type
async fn detect_system_type() -> SystemType {
    // Check if we're on macOS
    let os = std::env::consts::OS;

    if os == "macos" {
        // Check if nix-darwin is installed
        let darwin_check = Command::new("darwin-rebuild")
            .arg("--version")
            .output()
            .await;

        if darwin_check.is_ok() && darwin_check.unwrap().status.success() {
            return SystemType::NixDarwin;
        }

        // Also check for darwin-version
        let version_check = Command::new("sh")
            .args(["-c", "command -v darwin-rebuild"])
            .output()
            .await;

        if version_check.is_ok() {
            let output = version_check.unwrap();
            if output.status.success() && !output.stdout.is_empty() {
                return SystemType::NixDarwin;
            }
        }

        return SystemType::Unknown;
    }

    // Check for NixOS
    let nixos_check = Command::new("nixos-version")
        .output()
        .await;

    if nixos_check.is_ok() && nixos_check.unwrap().status.success() {
        return SystemType::NixOS;
    }

    SystemType::Unknown
}

/// System information response
#[derive(Debug, Serialize)]
pub struct SystemInfoResponse {
    pub hostname: String,
    pub nixos_version: String,
    pub kernel_version: String,
    pub uptime: String,
    pub system_type: SystemType,
    pub os: String,
}

/// Get local system information
pub async fn get_system_info(
    State(_state): State<AppState>,
) -> ApiResult<Json<SystemInfoResponse>> {
    let system_type = detect_system_type().await;
    let os = std::env::consts::OS.to_string();

    // Get hostname
    let hostname = Command::new("hostname")
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Get version based on system type
    let nixos_version = match system_type {
        SystemType::NixOS => {
            Command::new("nixos-version")
                .output()
                .await
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|_| "Unknown".to_string())
        }
        SystemType::NixDarwin => {
            // Try to get nix-darwin generation info
            let generation = Command::new("sh")
                .args(["-c", "darwin-rebuild --list-generations 2>/dev/null | tail -1 | awk '{print $1, $2, $3}'"])
                .output()
                .await
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();

            if generation.is_empty() {
                // Try sw_vers for macOS version
                let macos_ver = Command::new("sw_vers")
                    .args(["-productVersion"])
                    .output()
                    .await
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .unwrap_or_else(|_| "Unknown".to_string());
                format!("nix-darwin (macOS {})", macos_ver)
            } else {
                format!("nix-darwin {}", generation)
            }
        }
        SystemType::Unknown => "Not a Nix system".to_string(),
    };

    // Get kernel version
    let kernel_version = Command::new("uname")
        .arg("-r")
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    // Get uptime (different command on macOS vs Linux)
    let uptime = if os == "macos" {
        // macOS uptime format
        let output = Command::new("uptime")
            .output()
            .await
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        // Parse macOS uptime output: "14:50  up 2 days, 3:45, 2 users, load averages: 1.23 1.45 1.67"
        if let Some(up_idx) = output.find("up ") {
            let rest = &output[up_idx + 3..];
            if let Some(users_idx) = rest.find(" user") {
                // Find the comma before "user"
                if let Some(comma_idx) = rest[..users_idx].rfind(',') {
                    rest[..comma_idx].trim().to_string()
                } else {
                    rest[..users_idx].trim().to_string()
                }
            } else {
                rest.split(',').next().unwrap_or(rest).trim().to_string()
            }
        } else {
            output
        }
    } else {
        // Linux uptime -p
        Command::new("uptime")
            .arg("-p")
            .output()
            .await
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "Unknown".to_string())
    };

    Ok(Json(SystemInfoResponse {
        hostname,
        nixos_version,
        kernel_version,
        uptime,
        system_type,
        os,
    }))
}

/// Service information
#[derive(Debug, Clone, Serialize)]
pub struct LocalServiceInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub enabled: bool,
}

/// Response for service list
#[derive(Debug, Serialize)]
pub struct LocalServiceListResponse {
    pub services: Vec<LocalServiceInfo>,
    pub total: usize,
    pub system_type: SystemType,
}

/// List all local services
pub async fn list_local_services(
    State(_state): State<AppState>,
) -> ApiResult<Json<LocalServiceListResponse>> {
    let system_type = detect_system_type().await;
    let os = std::env::consts::OS;

    let services = if os == "macos" {
        list_launchd_services().await?
    } else {
        list_systemd_services().await?
    };

    let total = services.len();
    Ok(Json(LocalServiceListResponse { services, total, system_type }))
}

/// List systemd services (Linux/NixOS)
async fn list_systemd_services() -> Result<Vec<LocalServiceInfo>, ApiError> {
    let output = Command::new("systemctl")
        .args(["list-units", "--type=service", "--all", "--no-pager", "--plain", "--no-legend"])
        .output()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to execute systemctl: {}", e)))?;

    if !output.status.success() {
        return Err(ApiError::Internal(format!(
            "systemctl failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut services = Vec::new();

    for line in stdout.lines() {
        if let Some(info) = parse_systemd_service_line(line) {
            services.push(info);
        }
    }

    Ok(services)
}

/// List launchd services (macOS/nix-darwin)
async fn list_launchd_services() -> Result<Vec<LocalServiceInfo>, ApiError> {
    // Get user services
    let user_output = Command::new("launchctl")
        .args(["list"])
        .output()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to execute launchctl: {}", e)))?;

    let mut services = Vec::new();

    // Parse user services
    if user_output.status.success() {
        let stdout = String::from_utf8_lossy(&user_output.stdout);
        for line in stdout.lines().skip(1) { // Skip header
            if let Some(info) = parse_launchd_service_line(line, false) {
                services.push(info);
            }
        }
    }

    // Try to get system services (may require sudo)
    let system_output = Command::new("sudo")
        .args(["launchctl", "list"])
        .output()
        .await;

    if let Ok(output) = system_output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                if let Some(info) = parse_launchd_service_line(line, true) {
                    // Avoid duplicates
                    if !services.iter().any(|s| s.name == info.name) {
                        services.push(info);
                    }
                }
            }
        }
    }

    // Sort by name
    services.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(services)
}

fn parse_systemd_service_line(line: &str) -> Option<LocalServiceInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let name = parts[0].to_string();
    let load_state = parts[1].to_string();
    let active_state = parts[2].to_string();
    let sub_state = parts[3].to_string();
    let description = if parts.len() > 4 {
        Some(parts[4..].join(" "))
    } else {
        None
    };

    Some(LocalServiceInfo {
        name,
        description,
        load_state,
        active_state,
        sub_state,
        enabled: false,
    })
}

fn parse_launchd_service_line(line: &str, is_system: bool) -> Option<LocalServiceInfo> {
    // launchctl list format: PID Status Label
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let pid = parts[0];
    let status = parts[1];
    let label = parts[2..].join(" ");

    // Skip Apple internal services unless they're nix-related
    if label.starts_with("com.apple.") && !label.contains("nix") {
        return None;
    }

    let (active_state, sub_state) = if pid == "-" {
        ("inactive".to_string(), "stopped".to_string())
    } else {
        ("active".to_string(), format!("running (PID {})", pid))
    };

    let load_state = if status == "0" || pid != "-" {
        "loaded".to_string()
    } else {
        format!("error ({})", status)
    };

    let description = if is_system {
        Some("System service".to_string())
    } else {
        Some("User service".to_string())
    };

    Some(LocalServiceInfo {
        name: label,
        description,
        load_state,
        active_state,
        sub_state,
        enabled: true, // launchd services are enabled if they appear in list
    })
}

/// Service detail response
#[derive(Debug, Serialize)]
pub struct LocalServiceDetailResponse {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_file_path: Option<String>,
    pub requires: Vec<String>,
    pub wanted_by: Vec<String>,
    pub after: Vec<String>,
    pub before: Vec<String>,
    pub recent_logs: Vec<String>,
    pub system_type: SystemType,
}

/// Get detailed status of a local service
pub async fn get_local_service(
    State(_state): State<AppState>,
    Path(service): Path<String>,
) -> ApiResult<Json<LocalServiceDetailResponse>> {
    let system_type = detect_system_type().await;
    let os = std::env::consts::OS;

    if os == "macos" {
        get_launchd_service_detail(&service, system_type).await
    } else {
        get_systemd_service_detail(&service, system_type).await
    }
}

async fn get_systemd_service_detail(service: &str, system_type: SystemType) -> ApiResult<Json<LocalServiceDetailResponse>> {
    let service_name = if service.ends_with(".service") {
        service.to_string()
    } else {
        format!("{}.service", service)
    };

    // Get service properties
    let output = Command::new("systemctl")
        .args(["show", &service_name, "--no-pager"])
        .output()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to execute systemctl: {}", e)))?;

    if !output.status.success() {
        return Err(ApiError::NotFound(format!("Service not found: {}", service_name)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut info = parse_systemd_service_properties(&service_name, &stdout, system_type);

    // Get dependencies
    let deps_output = Command::new("systemctl")
        .args(["show", &service_name, "--property=Requires,WantedBy,After,Before", "--no-pager"])
        .output()
        .await
        .ok();

    if let Some(deps) = deps_output {
        let deps_stdout = String::from_utf8_lossy(&deps.stdout);
        for line in deps_stdout.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let items: Vec<String> = value
                    .split_whitespace()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();

                match key {
                    "Requires" => info.requires = items,
                    "WantedBy" => info.wanted_by = items,
                    "After" => info.after = items,
                    "Before" => info.before = items,
                    _ => {}
                }
            }
        }
    }

    // Get recent logs
    let logs_output = Command::new("journalctl")
        .args(["-u", &service_name, "-n", "20", "--no-pager", "--output=short-iso"])
        .output()
        .await
        .ok();

    if let Some(logs) = logs_output {
        info.recent_logs = String::from_utf8_lossy(&logs.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();
    }

    Ok(Json(info))
}

async fn get_launchd_service_detail(service: &str, system_type: SystemType) -> ApiResult<Json<LocalServiceDetailResponse>> {
    // Try user domain first, then system domain
    let user_output = Command::new("launchctl")
        .args(["list", service])
        .output()
        .await;

    let (is_loaded, pid, status) = if let Ok(output) = &user_output {
        if output.status.success() {
            parse_launchctl_list_output(&String::from_utf8_lossy(&output.stdout))
        } else {
            // Try system domain
            let system_output = Command::new("sudo")
                .args(["launchctl", "list", service])
                .output()
                .await;

            if let Ok(output) = system_output {
                if output.status.success() {
                    parse_launchctl_list_output(&String::from_utf8_lossy(&output.stdout))
                } else {
                    (false, None, None)
                }
            } else {
                (false, None, None)
            }
        }
    } else {
        (false, None, None)
    };

    if !is_loaded {
        return Err(ApiError::NotFound(format!("Service not found: {}", service)));
    }

    let (active_state, sub_state) = if let Some(p) = pid {
        ("active".to_string(), format!("running (PID {})", p))
    } else {
        ("inactive".to_string(), "stopped".to_string())
    };

    let load_state = match status {
        Some(0) | None => "loaded".to_string(),
        Some(code) => format!("error (exit code {})", code),
    };

    // Try to find the plist file
    let plist_paths = [
        format!("/Library/LaunchDaemons/{}.plist", service),
        format!("/Library/LaunchAgents/{}.plist", service),
        format!("{}/Library/LaunchAgents/{}.plist", std::env::var("HOME").unwrap_or_default(), service),
        format!("/System/Library/LaunchDaemons/{}.plist", service),
    ];

    let unit_file_path = plist_paths.iter()
        .find(|p| std::path::Path::new(p).exists())
        .cloned();

    // Get logs from system log
    let logs_output = Command::new("log")
        .args(["show", "--predicate", &format!("subsystem == '{}'", service), "--last", "5m", "--style", "compact"])
        .output()
        .await
        .ok();

    let recent_logs = if let Some(logs) = logs_output {
        String::from_utf8_lossy(&logs.stdout)
            .lines()
            .take(20)
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    };

    Ok(Json(LocalServiceDetailResponse {
        name: service.to_string(),
        description: Some("launchd service".to_string()),
        load_state,
        active_state,
        sub_state,
        enabled: true,
        main_pid: pid,
        memory_bytes: None,
        started_at: None,
        unit_file_path,
        requires: Vec::new(),
        wanted_by: Vec::new(),
        after: Vec::new(),
        before: Vec::new(),
        recent_logs,
        system_type,
    }))
}

fn parse_launchctl_list_output(output: &str) -> (bool, Option<u32>, Option<i32>) {
    // launchctl list <label> output format:
    // {
    //     "PID" = 1234;
    //     "LastExitStatus" = 0;
    //     ...
    // }
    let mut pid = None;
    let mut status = None;

    for line in output.lines() {
        let line = line.trim();
        if line.contains("\"PID\"") {
            if let Some(val) = line.split('=').nth(1) {
                pid = val.trim().trim_end_matches(';').trim().parse().ok();
            }
        } else if line.contains("\"LastExitStatus\"") {
            if let Some(val) = line.split('=').nth(1) {
                status = val.trim().trim_end_matches(';').trim().parse().ok();
            }
        }
    }

    (true, pid, status)
}

fn parse_systemd_service_properties(name: &str, output: &str, system_type: SystemType) -> LocalServiceDetailResponse {
    let mut info = LocalServiceDetailResponse {
        name: name.to_string(),
        description: None,
        load_state: "unknown".to_string(),
        active_state: "unknown".to_string(),
        sub_state: "unknown".to_string(),
        enabled: false,
        main_pid: None,
        memory_bytes: None,
        started_at: None,
        unit_file_path: None,
        requires: Vec::new(),
        wanted_by: Vec::new(),
        after: Vec::new(),
        before: Vec::new(),
        recent_logs: Vec::new(),
        system_type,
    };

    for line in output.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key {
                "Description" => info.description = Some(value.to_string()),
                "LoadState" => info.load_state = value.to_string(),
                "ActiveState" => info.active_state = value.to_string(),
                "SubState" => info.sub_state = value.to_string(),
                "UnitFileState" => {
                    info.enabled = value == "enabled" || value == "enabled-runtime"
                }
                "MainPID" => info.main_pid = value.parse().ok().filter(|&p| p > 0),
                "MemoryCurrent" => {
                    info.memory_bytes = value.parse().ok().filter(|&m| m < u64::MAX)
                }
                "ActiveEnterTimestamp" => {
                    if !value.is_empty() && value != "n/a" {
                        info.started_at = Some(value.to_string());
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

    info
}

/// Service action request
#[derive(Debug, Deserialize)]
pub struct LocalServiceActionRequest {
    pub action: String,
}

/// Service action response
#[derive(Debug, Serialize)]
pub struct LocalServiceActionResponse {
    pub success: bool,
    pub message: String,
    pub system_type: SystemType,
}

/// Perform an action on a local service
pub async fn local_service_action(
    State(_state): State<AppState>,
    Path(service): Path<String>,
    Json(body): Json<LocalServiceActionRequest>,
) -> ApiResult<Json<LocalServiceActionResponse>> {
    let system_type = detect_system_type().await;
    let os = std::env::consts::OS;

    if os == "macos" {
        launchd_service_action(&service, &body.action, system_type).await
    } else {
        systemd_service_action(&service, &body.action, system_type).await
    }
}

async fn systemd_service_action(service: &str, action: &str, system_type: SystemType) -> ApiResult<Json<LocalServiceActionResponse>> {
    let service_name = if service.ends_with(".service") {
        service.to_string()
    } else {
        format!("{}.service", service)
    };

    let valid_actions = ["start", "stop", "restart", "reload", "enable", "disable"];
    if !valid_actions.contains(&action) {
        return Err(ApiError::BadRequest(format!(
            "Invalid action: {}. Valid actions: {:?}",
            action, valid_actions
        )));
    }

    let output = Command::new("sudo")
        .args(["systemctl", action, &service_name])
        .output()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to execute systemctl: {}", e)))?;

    let success = output.status.success();
    let message = if success {
        format!("Successfully {}ed {}", action, service_name)
    } else {
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    };

    Ok(Json(LocalServiceActionResponse { success, message, system_type }))
}

async fn launchd_service_action(service: &str, action: &str, system_type: SystemType) -> ApiResult<Json<LocalServiceActionResponse>> {
    let valid_actions = ["start", "stop", "restart", "enable", "disable"];
    if !valid_actions.contains(&action) {
        return Err(ApiError::BadRequest(format!(
            "Invalid action: {}. Valid actions for launchd: {:?}",
            action, valid_actions
        )));
    }

    let service_target = format!("system/{}", service);

    // Handle enable/disable separately as they need plist paths
    if action == "enable" || action == "disable" {
        let plist_paths = [
            format!("/Library/LaunchDaemons/{}.plist", service),
            format!("/Library/LaunchAgents/{}.plist", service),
        ];

        let plist = plist_paths.iter().find(|p| std::path::Path::new(p).exists());

        if let Some(path) = plist {
            let launchctl_action = if action == "enable" { "load" } else { "unload" };
            let output = Command::new("sudo")
                .args(["launchctl", launchctl_action, "-w", path])
                .output()
                .await
                .map_err(|e| ApiError::Internal(format!("Failed to execute launchctl: {}", e)))?;

            let success = output.status.success();
            let message = if success {
                format!("Successfully {}d {}", action, service)
            } else {
                String::from_utf8_lossy(&output.stderr).trim().to_string()
            };
            return Ok(Json(LocalServiceActionResponse { success, message, system_type }));
        } else {
            return Err(ApiError::NotFound(format!("Plist not found for service: {}", service)));
        }
    }

    // Handle start/stop/restart
    let args: Vec<String> = match action {
        "start" | "restart" => vec![
            "launchctl".to_string(),
            "kickstart".to_string(),
            "-k".to_string(),
            service_target,
        ],
        "stop" => vec![
            "launchctl".to_string(),
            "kill".to_string(),
            "SIGTERM".to_string(),
            service_target,
        ],
        _ => unreachable!(),
    };

    let output = Command::new("sudo")
        .args(&args)
        .output()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to execute launchctl: {}", e)))?;

    let success = output.status.success();
    let message = if success {
        format!("Successfully {}ed {}", action, service)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            stderr
        }
    };

    Ok(Json(LocalServiceActionResponse { success, message, system_type }))
}

/// Query params for log fetching
#[derive(Debug, Deserialize)]
pub struct LocalLogQueryParams {
    #[serde(default)]
    pub lines: Option<u32>,
}

/// Log response
#[derive(Debug, Serialize)]
pub struct LocalLogResponse {
    pub logs: Vec<String>,
    pub service: String,
    pub system_type: SystemType,
}

/// Get logs for a local service
pub async fn get_local_service_logs(
    State(_state): State<AppState>,
    Path(service): Path<String>,
    Query(params): Query<LocalLogQueryParams>,
) -> ApiResult<Json<LocalLogResponse>> {
    let system_type = detect_system_type().await;
    let os = std::env::consts::OS;
    let lines = params.lines.unwrap_or(100);

    let logs = if os == "macos" {
        get_macos_service_logs(&service, lines).await?
    } else {
        get_journalctl_logs(&service, lines).await?
    };

    Ok(Json(LocalLogResponse { logs, service, system_type }))
}

async fn get_journalctl_logs(service: &str, lines: u32) -> Result<Vec<String>, ApiError> {
    let service_name = if service.ends_with(".service") {
        service.to_string()
    } else {
        format!("{}.service", service)
    };

    let output = Command::new("journalctl")
        .args(["-u", &service_name, "-n", &lines.to_string(), "--no-pager", "--output=short-iso"])
        .output()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to execute journalctl: {}", e)))?;

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect())
}

async fn get_macos_service_logs(service: &str, lines: u32) -> Result<Vec<String>, ApiError> {
    // Try unified logging system first
    let output = Command::new("log")
        .args([
            "show",
            "--predicate", &format!("subsystem == '{}' OR processImagePath CONTAINS '{}'", service, service),
            "--last", "1h",
            "--style", "compact",
        ])
        .output()
        .await;

    if let Ok(out) = output {
        if out.status.success() {
            let logs: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .take(lines as usize)
                .map(|s| s.to_string())
                .collect();

            if !logs.is_empty() {
                return Ok(logs);
            }
        }
    }

    // Fall back to syslog for this service
    let output = Command::new("log")
        .args([
            "show",
            "--predicate", &format!("eventMessage CONTAINS '{}'", service),
            "--last", "1h",
            "--style", "compact",
        ])
        .output()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to execute log command: {}", e)))?;

    let logs: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .take(lines as usize)
        .map(|s| s.to_string())
        .collect();

    if logs.is_empty() {
        // Try system.log as last resort
        let output = Command::new("grep")
            .args(["-i", service, "/var/log/system.log"])
            .output()
            .await;

        if let Ok(out) = output {
            return Ok(String::from_utf8_lossy(&out.stdout)
                .lines()
                .rev()
                .take(lines as usize)
                .map(|s| s.to_string())
                .collect());
        }
    }

    Ok(logs)
}

/// Rebuild request
#[derive(Debug, Deserialize)]
pub struct RebuildRequest {
    pub action: String,
    #[serde(default)]
    pub flake_path: Option<String>,
}

/// Rebuild response
#[derive(Debug, Serialize)]
pub struct RebuildResponse {
    pub success: bool,
    pub job_id: Option<String>,
    pub message: String,
    pub system_type: SystemType,
    pub command: String,
}

/// Start a NixOS/nix-darwin rebuild
pub async fn start_rebuild(
    State(_state): State<AppState>,
    Json(body): Json<RebuildRequest>,
) -> ApiResult<Json<RebuildResponse>> {
    let system_type = detect_system_type().await;

    // Different valid actions for NixOS vs nix-darwin
    let (rebuild_cmd, valid_actions): (&str, &[&str]) = match system_type {
        SystemType::NixOS => ("nixos-rebuild", &["switch", "boot", "test", "build", "dry-build", "dry-activate"]),
        SystemType::NixDarwin => ("darwin-rebuild", &["switch", "build", "check", "changelog"]),
        SystemType::Unknown => {
            return Ok(Json(RebuildResponse {
                success: false,
                job_id: None,
                message: "Cannot rebuild: System is neither NixOS nor nix-darwin".to_string(),
                system_type,
                command: "".to_string(),
            }));
        }
    };

    if !valid_actions.contains(&body.action.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Invalid action for {}: {}. Valid actions: {:?}",
            rebuild_cmd, body.action, valid_actions
        )));
    }

    // Build the command
    let mut args = vec![rebuild_cmd.to_string(), body.action.clone()];

    if let Some(flake_path) = &body.flake_path {
        args.push("--flake".to_string());
        args.push(flake_path.clone());
    }

    let command_str = format!("sudo {}", args.join(" "));

    // Generate a job ID
    let job_id = uuid::Uuid::new_v4().to_string();

    // For now, just start the process and return immediately
    // In production, you'd want to track this job and stream output
    let mut command = Command::new("sudo");
    command.args(&args);

    // Spawn the process in the background
    match command.spawn() {
        Ok(_) => Ok(Json(RebuildResponse {
            success: true,
            job_id: Some(job_id),
            message: format!("{} ({}) started", rebuild_cmd, body.action),
            system_type,
            command: command_str,
        })),
        Err(e) => Ok(Json(RebuildResponse {
            success: false,
            job_id: None,
            message: format!("Failed to start {}: {}", rebuild_cmd, e),
            system_type,
            command: command_str,
        })),
    }
}
