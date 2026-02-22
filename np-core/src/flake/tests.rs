//! Unit tests for flake types and parsing

use super::parser::FlakeParser;
use super::types::*;

#[test]
fn test_flake_id_generation() {
    let id1 = FlakeId::new();
    let id2 = FlakeId::new();

    // Each ID should be unique
    assert_ne!(id1.0, id2.0);

    // IDs should be valid UUIDs (36 chars with hyphens)
    assert_eq!(id1.0.len(), 36);
}

#[test]
fn test_flake_id_display() {
    let id = FlakeId("test-flake-123".to_string());
    assert_eq!(id.to_string(), "test-flake-123");
}

#[test]
fn test_flake_input_ref_github() {
    let input_ref = FlakeInputRef {
        input_type: "github".to_string(),
        owner: Some("NixOS".to_string()),
        repo: Some("nixpkgs".to_string()),
        path: None,
        url: None,
        git_ref: Some("nixos-24.05".to_string()),
        rev: None,
    };

    assert_eq!(input_ref.to_flake_ref(), "github:NixOS/nixpkgs/nixos-24.05");
}

#[test]
fn test_flake_input_ref_github_with_rev() {
    let input_ref = FlakeInputRef {
        input_type: "github".to_string(),
        owner: Some("nix-community".to_string()),
        repo: Some("home-manager".to_string()),
        path: None,
        url: None,
        git_ref: None,
        rev: Some("abc123def456".to_string()),
    };

    assert_eq!(
        input_ref.to_flake_ref(),
        "github:nix-community/home-manager/abc123def456"
    );
}

#[test]
fn test_flake_input_ref_gitlab() {
    let input_ref = FlakeInputRef {
        input_type: "gitlab".to_string(),
        owner: Some("myorg".to_string()),
        repo: Some("myrepo".to_string()),
        path: None,
        url: None,
        git_ref: None,
        rev: None,
    };

    assert_eq!(input_ref.to_flake_ref(), "gitlab:myorg/myrepo");
}

#[test]
fn test_flake_input_ref_path() {
    let input_ref = FlakeInputRef {
        input_type: "path".to_string(),
        owner: None,
        repo: None,
        path: Some("/home/user/my-flake".to_string()),
        url: None,
        git_ref: None,
        rev: None,
    };

    assert_eq!(input_ref.to_flake_ref(), "path:/home/user/my-flake");
}

#[test]
fn test_flake_input_ref_git() {
    let input_ref = FlakeInputRef {
        input_type: "git".to_string(),
        owner: None,
        repo: None,
        path: None,
        url: Some("https://github.com/owner/repo.git".to_string()),
        git_ref: Some("main".to_string()),
        rev: None,
    };

    assert_eq!(
        input_ref.to_flake_ref(),
        "git+https://github.com/owner/repo.git?ref=main"
    );
}

#[test]
fn test_flake_input_ref_tarball() {
    let input_ref = FlakeInputRef {
        input_type: "tarball".to_string(),
        owner: None,
        repo: None,
        path: None,
        url: Some("https://example.com/archive.tar.gz".to_string()),
        git_ref: None,
        rev: None,
    };

    assert_eq!(
        input_ref.to_flake_ref(),
        "https://example.com/archive.tar.gz"
    );
}

#[test]
fn test_lock_node_input_direct() {
    let input = LockNodeInput::Direct("nixpkgs".to_string());

    let json = serde_json::to_string(&input).unwrap();
    assert_eq!(json, "\"nixpkgs\"");

    let parsed: LockNodeInput = serde_json::from_str(&json).unwrap();
    match parsed {
        LockNodeInput::Direct(name) => assert_eq!(name, "nixpkgs"),
        _ => panic!("Expected Direct variant"),
    }
}

#[test]
fn test_lock_node_input_follows() {
    let input = LockNodeInput::Follows(vec!["root".to_string(), "nixpkgs".to_string()]);

    let json = serde_json::to_string(&input).unwrap();
    assert_eq!(json, "[\"root\",\"nixpkgs\"]");

    let parsed: LockNodeInput = serde_json::from_str(&json).unwrap();
    match parsed {
        LockNodeInput::Follows(path) => {
            assert_eq!(path.len(), 2);
            assert_eq!(path[0], "root");
            assert_eq!(path[1], "nixpkgs");
        }
        _ => panic!("Expected Follows variant"),
    }
}

#[test]
fn test_parse_flake_locks() {
    let json = serde_json::json!({
        "version": 7,
        "root": "root",
        "nodes": {
            "root": {
                "inputs": {
                    "nixpkgs": "nixpkgs"
                }
            },
            "nixpkgs": {
                "locked": {
                    "type": "github",
                    "owner": "NixOS",
                    "repo": "nixpkgs",
                    "rev": "abc123",
                    "narHash": "sha256-xxx"
                },
                "original": {
                    "type": "github",
                    "owner": "NixOS",
                    "repo": "nixpkgs",
                    "ref": "nixos-24.05"
                }
            }
        }
    });

    let locks = FlakeParser::parse_locks(&json).unwrap();

    assert_eq!(locks.version, 7);
    assert_eq!(locks.root, "root");
    assert_eq!(locks.nodes.len(), 2);

    // Check nixpkgs node
    let nixpkgs = locks.nodes.get("nixpkgs").unwrap();
    assert!(nixpkgs.locked.is_some());
    assert!(nixpkgs.original.is_some());

    let locked = nixpkgs.locked.as_ref().unwrap();
    assert_eq!(locked.input_type, "github");
    assert_eq!(locked.owner, Some("NixOS".to_string()));
    assert_eq!(locked.rev, Some("abc123".to_string()));
}

#[test]
fn test_parse_metadata_json() {
    let json = serde_json::json!({
        "description": "My awesome flake",
        "lastModified": 1700000000,
        "lockedUrl": "git+file:///etc/nixos",
        "originalUrl": "path:/etc/nixos",
        "resolvedUrl": "path:/etc/nixos",
        "revision": "abc123def456",
        "path": "/etc/nixos",
        "locks": {
            "version": 7,
            "root": "root",
            "nodes": {
                "root": {
                    "inputs": {}
                }
            }
        }
    });

    let metadata = FlakeParser::parse_metadata_json(&json, "/etc/nixos").unwrap();

    assert_eq!(metadata.description, Some("My awesome flake".to_string()));
    assert!(metadata.last_modified.is_some());
    assert_eq!(
        metadata.locked_url,
        Some("git+file:///etc/nixos".to_string())
    );
    assert_eq!(metadata.revision, Some("abc123def456".to_string()));
    assert!(metadata.locks.is_some());
}

#[test]
fn test_flake_outputs_deserialization() {
    let json = r#"{
        "nixosConfigurations": {
            "myhost": {
                "type": "nixos-configuration"
            }
        },
        "darwinConfigurations": {
            "macbook": {
                "type": "darwin-configuration"
            }
        },
        "packages": {
            "x86_64-linux": {
                "default": {
                    "type": "derivation",
                    "name": "my-package",
                    "description": "My package"
                }
            }
        },
        "devShells": {
            "x86_64-linux": {
                "default": {
                    "type": "derivation"
                }
            }
        }
    }"#;

    let outputs: FlakeOutputs = serde_json::from_str(json).unwrap();

    assert!(outputs.nixos_configurations.is_some());
    assert!(outputs.darwin_configurations.is_some());
    assert!(outputs.packages.is_some());
    assert!(outputs.dev_shells.is_some());

    let nixos_configs = outputs.nixos_configurations.unwrap();
    assert!(nixos_configs.contains_key("myhost"));

    let darwin_configs = outputs.darwin_configurations.unwrap();
    assert!(darwin_configs.contains_key("macbook"));
}

#[test]
fn test_create_flake_request() {
    let json = r#"{
        "name": "my-flake",
        "path": "/etc/nixos",
        "description": "My NixOS configuration"
    }"#;

    let req: CreateFlakeRequest = serde_json::from_str(json).unwrap();

    assert_eq!(req.name, "my-flake");
    assert_eq!(req.path.to_str().unwrap(), "/etc/nixos");
    assert_eq!(req.description, Some("My NixOS configuration".to_string()));
}

#[test]
fn test_update_flake_request() {
    let json = r#"{
        "name": "new-name"
    }"#;

    let req: UpdateFlakeRequest = serde_json::from_str(json).unwrap();

    assert_eq!(req.name, Some("new-name".to_string()));
    assert!(req.description.is_none());
}

#[test]
fn test_update_input_request() {
    let json = r#"{
        "flake_ref": "github:NixOS/nixpkgs/nixos-unstable"
    }"#;

    let req: UpdateInputRequest = serde_json::from_str(json).unwrap();

    assert_eq!(
        req.flake_ref,
        Some("github:NixOS/nixpkgs/nixos-unstable".to_string())
    );
    assert!(req.follows.is_none());
}

#[test]
fn test_update_input_request_follows() {
    let json = r#"{
        "follows": ["nixpkgs"]
    }"#;

    let req: UpdateInputRequest = serde_json::from_str(json).unwrap();

    assert!(req.flake_ref.is_none());
    assert_eq!(req.follows, Some(vec!["nixpkgs".to_string()]));
}

#[test]
fn test_flake_input_locked_serialization() {
    let locked = FlakeInputLocked {
        input_type: "github".to_string(),
        owner: Some("NixOS".to_string()),
        repo: Some("nixpkgs".to_string()),
        rev: Some("abc123".to_string()),
        nar_hash: Some("sha256-xxx".to_string()),
        last_modified: Some(1700000000),
    };

    let json = serde_json::to_string(&locked).unwrap();
    assert!(json.contains("\"type\":\"github\""));
    assert!(json.contains("\"owner\":\"NixOS\""));
    assert!(json.contains("\"narHash\":\"sha256-xxx\""));
    assert!(json.contains("\"lastModified\":1700000000"));

    // Deserialize back
    let parsed: FlakeInputLocked = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.input_type, "github");
    assert_eq!(parsed.rev, Some("abc123".to_string()));
}

#[test]
fn test_registered_flake_creation() {
    let now = chrono::Utc::now();
    let flake = RegisteredFlake {
        id: FlakeId::new(),
        name: "my-config".to_string(),
        path: std::path::PathBuf::from("/etc/nixos"),
        description: Some("My NixOS config".to_string()),
        metadata: None,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(flake.name, "my-config");
    assert_eq!(flake.path.to_str().unwrap(), "/etc/nixos");
    assert!(flake.description.is_some());
    assert!(flake.metadata.is_none());
}
