//! Unit tests for machine types and operations

use super::types::*;
use crate::ssh::{SshAuthMethod, SshCredentials, SshTarget};

#[test]
fn test_machine_id_generation() {
    let id1 = MachineId::new();
    let id2 = MachineId::new();

    // Each ID should be unique
    assert_ne!(id1.0, id2.0);

    // IDs should be valid UUIDs (36 chars with hyphens)
    assert_eq!(id1.0.len(), 36);
    assert!(id1.0.contains('-'));
}

#[test]
fn test_machine_id_from_string() {
    let id = MachineId::from_string("test-machine-123");
    assert_eq!(id.0, "test-machine-123");
    assert_eq!(id.to_string(), "test-machine-123");
    assert_eq!(id.as_ref(), "test-machine-123");
}

#[test]
fn test_machine_status_default() {
    let status: MachineStatus = Default::default();
    assert_eq!(status, MachineStatus::Unknown);
}

#[test]
fn test_machine_status_serialization() {
    let cases = vec![
        (MachineStatus::Unknown, "\"unknown\""),
        (MachineStatus::Online, "\"online\""),
        (MachineStatus::Offline, "\"offline\""),
        (MachineStatus::AuthFailed, "\"authfailed\""),
    ];

    for (status, expected) in cases {
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, expected);

        let parsed: MachineStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, status);
    }
}

fn create_test_target() -> SshTarget {
    SshTarget {
        host: "192.168.1.100".to_string(),
        port: 22,
        credentials: SshCredentials::with_agent("root"),
    }
}

#[test]
fn test_machine_creation() {
    let target = create_test_target();

    let machine = Machine::new("test-server", target);

    assert!(!machine.id.0.is_empty());
    assert_eq!(machine.name, "test-server");
    assert_eq!(machine.target.host, "192.168.1.100");
    assert_eq!(machine.target.port, 22);
    assert_eq!(machine.status, MachineStatus::Unknown);
    assert!(machine.description.is_none());
    assert!(machine.tags.is_empty());
}

#[test]
fn test_machine_with_description() {
    let target = create_test_target();

    let machine = Machine::new("test-server", target)
        .with_description("A test server for development");

    assert_eq!(
        machine.description,
        Some("A test server for development".to_string())
    );
}

#[test]
fn test_machine_with_tags() {
    let target = create_test_target();

    let machine = Machine::new("test-server", target)
        .with_tags(vec!["production".to_string(), "web".to_string()]);

    assert_eq!(machine.tags.len(), 2);
    assert!(machine.tags.contains(&"production".to_string()));
    assert!(machine.tags.contains(&"web".to_string()));
}

#[test]
fn test_machine_update_status_online() {
    let target = create_test_target();

    let mut machine = Machine::new("test-server", target);
    let original_updated_at = machine.updated_at;

    // Small delay to ensure timestamps differ
    std::thread::sleep(std::time::Duration::from_millis(10));

    machine.update_status(MachineStatus::Online, Some("NixOS 24.05".to_string()));

    assert_eq!(machine.status, MachineStatus::Online);
    assert!(machine.last_seen.is_some());
    assert_eq!(machine.system_info, Some("NixOS 24.05".to_string()));
    assert!(machine.updated_at > original_updated_at);
}

#[test]
fn test_machine_update_status_offline() {
    let target = create_test_target();

    let mut machine = Machine::new("test-server", target);

    // First set to online
    machine.update_status(MachineStatus::Online, Some("NixOS 24.05".to_string()));
    let last_seen_when_online = machine.last_seen;

    // Then set to offline
    machine.update_status(MachineStatus::Offline, None);

    assert_eq!(machine.status, MachineStatus::Offline);
    // last_seen should remain from when it was online
    assert_eq!(machine.last_seen, last_seen_when_online);
}

#[test]
fn test_machine_serialization() {
    let target = create_test_target();

    let machine = Machine::new("test-server", target)
        .with_description("Test machine")
        .with_tags(vec!["test".to_string()]);

    // Serialize to JSON
    let json = serde_json::to_string(&machine).unwrap();
    assert!(json.contains("\"name\":\"test-server\""));
    assert!(json.contains("\"host\":\"192.168.1.100\""));

    // Deserialize back
    let parsed: Machine = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, machine.name);
    assert_eq!(parsed.target.host, machine.target.host);
}

#[test]
fn test_create_machine_request_deserialization() {
    let json = r#"{
        "name": "my-server",
        "host": "10.0.0.1",
        "port": 2222,
        "username": "admin",
        "auth_method": { "type": "agent" },
        "tags": ["production", "web"]
    }"#;

    let req: CreateMachineRequest = serde_json::from_str(json).unwrap();

    assert_eq!(req.name, "my-server");
    assert_eq!(req.host, "10.0.0.1");
    assert_eq!(req.port, 2222);
    assert_eq!(req.username, "admin");
    assert_eq!(req.tags.len(), 2);
}

#[test]
fn test_create_machine_request_default_port() {
    let json = r#"{
        "name": "my-server",
        "host": "10.0.0.1",
        "username": "admin"
    }"#;

    let req: CreateMachineRequest = serde_json::from_str(json).unwrap();

    assert_eq!(req.port, 22); // Default port
}

#[test]
fn test_create_machine_auth_method_key_file() {
    let json = r#"{
        "name": "my-server",
        "host": "10.0.0.1",
        "username": "admin",
        "auth_method": {
            "type": "key_file",
            "path": "/home/user/.ssh/id_rsa",
            "passphrase": "secret123"
        }
    }"#;

    let req: CreateMachineRequest = serde_json::from_str(json).unwrap();

    match req.auth_method {
        CreateMachineAuthMethod::KeyFile { path, passphrase } => {
            assert_eq!(path, "/home/user/.ssh/id_rsa");
            assert_eq!(passphrase, Some("secret123".to_string()));
        }
        _ => panic!("Expected KeyFile auth method"),
    }
}

#[test]
fn test_create_machine_auth_method_password() {
    let json = r#"{
        "name": "my-server",
        "host": "10.0.0.1",
        "username": "admin",
        "auth_method": {
            "type": "password",
            "password": "mypassword"
        }
    }"#;

    let req: CreateMachineRequest = serde_json::from_str(json).unwrap();

    match req.auth_method {
        CreateMachineAuthMethod::Password { password } => {
            assert_eq!(password, "mypassword");
        }
        _ => panic!("Expected Password auth method"),
    }
}

#[test]
fn test_update_machine_request_partial() {
    let json = r#"{
        "name": "new-name",
        "description": "Updated description"
    }"#;

    let req: UpdateMachineRequest = serde_json::from_str(json).unwrap();

    assert_eq!(req.name, Some("new-name".to_string()));
    assert_eq!(req.description, Some("Updated description".to_string()));
    assert!(req.host.is_none());
    assert!(req.port.is_none());
    assert!(req.username.is_none());
    assert!(req.auth_method.is_none());
    assert!(req.tags.is_none());
}

#[test]
fn test_ssh_credentials_with_agent() {
    let creds = SshCredentials::with_agent("root");

    assert_eq!(creds.username, "root");
    match creds.auth {
        SshAuthMethod::Agent => {}
        _ => panic!("Expected Agent auth method"),
    }
}

#[test]
fn test_ssh_credentials_with_key_file() {
    let creds = SshCredentials::with_key_file(
        "admin",
        std::path::PathBuf::from("/home/admin/.ssh/id_ed25519"),
        Some("passphrase123".to_string()),
    );

    assert_eq!(creds.username, "admin");
    match creds.auth {
        SshAuthMethod::KeyFile { path, passphrase } => {
            assert_eq!(path.to_str().unwrap(), "/home/admin/.ssh/id_ed25519");
            assert_eq!(passphrase, Some("passphrase123".to_string()));
        }
        _ => panic!("Expected KeyFile auth method"),
    }
}

#[test]
fn test_ssh_credentials_with_password() {
    let creds = SshCredentials::with_password("user", "secret");

    assert_eq!(creds.username, "user");
    match creds.auth {
        SshAuthMethod::Password { password } => {
            assert_eq!(password, "secret");
        }
        _ => panic!("Expected Password auth method"),
    }
}

#[test]
fn test_ssh_auth_method_default() {
    let auth: SshAuthMethod = Default::default();
    match auth {
        SshAuthMethod::Agent => {}
        _ => panic!("Expected default to be Agent"),
    }
}

#[test]
fn test_ssh_target_serialization() {
    let target = SshTarget {
        host: "192.168.1.50".to_string(),
        port: 2222,
        credentials: SshCredentials::with_agent("deploy"),
    };

    let json = serde_json::to_string(&target).unwrap();
    assert!(json.contains("\"host\":\"192.168.1.50\""));
    assert!(json.contains("\"port\":2222"));
    assert!(json.contains("\"username\":\"deploy\""));

    let parsed: SshTarget = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.host, "192.168.1.50");
    assert_eq!(parsed.port, 2222);
    assert_eq!(parsed.credentials.username, "deploy");
}
