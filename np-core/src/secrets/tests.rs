//! Unit tests for secrets types and operations

use super::types::*;

#[test]
fn test_secret_id_creation() {
    let id = SecretId::new("my-secret");
    assert_eq!(id.as_str(), "my-secret");
    assert_eq!(id.to_string(), "my-secret");
}

#[test]
fn test_secret_type_default() {
    let secret_type: SecretType = Default::default();
    assert_eq!(secret_type, SecretType::Text);
}

#[test]
fn test_secret_type_serialization() {
    let cases = vec![
        (SecretType::Text, "\"text\""),
        (SecretType::SshKey, "\"ssh_key\""),
        (SecretType::Certificate, "\"certificate\""),
        (SecretType::TlsKey, "\"tls_key\""),
        (SecretType::EnvFile, "\"env_file\""),
        (SecretType::Binary, "\"binary\""),
        (SecretType::Json, "\"json\""),
        (SecretType::Yaml, "\"yaml\""),
    ];

    for (secret_type, expected) in cases {
        let json = serde_json::to_string(&secret_type).unwrap();
        assert_eq!(json, expected);

        let parsed: SecretType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, secret_type);
    }
}

#[test]
fn test_secret_creation() {
    let secret = Secret::new(
        SecretId::new("api-key"),
        "API Key",
        "super-secret-value",
        SecretType::Text,
    );

    assert_eq!(secret.id.as_str(), "api-key");
    assert_eq!(secret.name, "API Key");
    assert_eq!(secret.secret_type, SecretType::Text);
    assert!(secret.description.is_none());
    assert_eq!(secret.expose_value(), "super-secret-value");
}

#[test]
fn test_secret_with_description() {
    let secret = Secret::new(
        SecretId::new("db-password"),
        "Database Password",
        "password123",
        SecretType::Text,
    )
    .with_description("Production database password");

    assert_eq!(
        secret.description,
        Some("Production database password".to_string())
    );
}

#[test]
fn test_secret_metadata_default() {
    let metadata: SecretMetadata = Default::default();

    assert_eq!(metadata.version, 1);
    assert!(metadata.tags.is_empty());
    assert!(metadata.machines.is_empty());
    assert!(metadata.environment.is_none());
    assert!(metadata.created_by.is_none());
    assert!(metadata.extra.is_empty());
}

#[test]
fn test_sops_metadata_default() {
    let sops_meta: SopsMetadata = Default::default();

    assert!(sops_meta.key_groups.is_empty());
    assert!(sops_meta.age.is_empty());
    assert!(sops_meta.last_modified.is_none());
    assert!(sops_meta.mac.is_none());
    assert_eq!(sops_meta.version, Some("3.8.1".to_string()));
}

#[test]
fn test_secrets_config_default() {
    let config: SecretsConfig = Default::default();

    assert_eq!(config.secrets_dir.to_str().unwrap(), "secrets");
    assert_eq!(config.sops_config_path.to_str().unwrap(), ".sops.yaml");
    assert!(config.default_recipients.is_empty());
    assert!(!config.use_sops_cli);
}

#[test]
fn test_secret_file_default() {
    let secret_file: SecretFile = Default::default();

    assert_eq!(secret_file.owner, "root");
    assert_eq!(secret_file.group, "root");
    assert_eq!(secret_file.mode, "0400");
    assert!(secret_file.restart_units.is_empty());
}

#[test]
fn test_create_secret_request_deserialization() {
    let json = r#"{
        "name": "api-key",
        "value": "secret-value-123",
        "secret_type": "text",
        "description": "API key for external service",
        "tags": ["production", "external"],
        "machines": ["server1", "server2"],
        "environment": "production"
    }"#;

    let req: CreateSecretRequest = serde_json::from_str(json).unwrap();

    assert_eq!(req.name, "api-key");
    assert_eq!(req.value, "secret-value-123");
    assert_eq!(req.secret_type, SecretType::Text);
    assert_eq!(
        req.description,
        Some("API key for external service".to_string())
    );
    assert_eq!(req.tags.len(), 2);
    assert_eq!(req.machines.len(), 2);
    assert_eq!(req.environment, Some("production".to_string()));
}

#[test]
fn test_update_secret_request_deserialization() {
    let json = r#"{
        "value": "new-secret-value",
        "description": "Updated description"
    }"#;

    let req: UpdateSecretRequest = serde_json::from_str(json).unwrap();

    assert_eq!(req.value, Some("new-secret-value".to_string()));
    assert_eq!(req.description, Some("Updated description".to_string()));
    assert!(req.tags.is_none());
    assert!(req.machines.is_none());
}

#[test]
fn test_update_secret_request_partial() {
    let json = r#"{
        "tags": ["new-tag"]
    }"#;

    let req: UpdateSecretRequest = serde_json::from_str(json).unwrap();

    assert!(req.value.is_none());
    assert!(req.description.is_none());
    assert_eq!(req.tags, Some(vec!["new-tag".to_string()]));
    assert!(req.machines.is_none());
}

#[test]
fn test_secret_summary_serialization() {
    let now = chrono::Utc::now();
    let summary = SecretSummary {
        id: SecretId::new("my-secret"),
        name: "My Secret".to_string(),
        description: Some("A test secret".to_string()),
        secret_type: SecretType::Text,
        metadata: SecretMetadata {
            created_at: now,
            modified_at: now,
            created_by: Some("admin".to_string()),
            version: 1,
            tags: vec!["test".to_string()],
            machines: vec![],
            environment: Some("dev".to_string()),
            extra: std::collections::HashMap::new(),
        },
        file_path: std::path::PathBuf::from("secrets/my-secret.yaml"),
    };

    let json = serde_json::to_string(&summary).unwrap();

    assert!(json.contains("\"id\":\"my-secret\""));
    assert!(json.contains("\"name\":\"My Secret\""));
    assert!(json.contains("\"secret_type\":\"text\""));
    assert!(json.contains("\"created_by\":\"admin\""));
}

#[test]
fn test_age_recipient_serialization() {
    let recipient = AgeRecipient {
        recipient: "age1xxx...".to_string(),
        enc: Some("encrypted-data-key".to_string()),
    };

    let json = serde_json::to_string(&recipient).unwrap();
    assert!(json.contains("\"recipient\":\"age1xxx...\""));
    assert!(json.contains("\"enc\":\"encrypted-data-key\""));

    let parsed: AgeRecipient = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.recipient, "age1xxx...");
    assert_eq!(parsed.enc, Some("encrypted-data-key".to_string()));
}

#[test]
fn test_key_group_serialization() {
    let key_group = KeyGroup {
        age: vec![
            AgeRecipient {
                recipient: "age1aaa...".to_string(),
                enc: None,
            },
            AgeRecipient {
                recipient: "age1bbb...".to_string(),
                enc: None,
            },
        ],
    };

    let json = serde_json::to_string(&key_group).unwrap();
    assert!(json.contains("age1aaa..."));
    assert!(json.contains("age1bbb..."));

    let parsed: KeyGroup = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.age.len(), 2);
}

#[test]
fn test_secret_file_serialization() {
    let secret_file = SecretFile {
        source_path: std::path::PathBuf::from("secrets/db-password.yaml"),
        target_path: std::path::PathBuf::from("/run/secrets/db-password"),
        owner: "postgres".to_string(),
        group: "postgres".to_string(),
        mode: "0400".to_string(),
        restart_units: vec!["postgresql.service".to_string()],
    };

    let json = serde_json::to_string(&secret_file).unwrap();

    assert!(json.contains("secrets/db-password.yaml"));
    assert!(json.contains("/run/secrets/db-password"));
    assert!(json.contains("\"owner\":\"postgres\""));
    assert!(json.contains("postgresql.service"));
}
