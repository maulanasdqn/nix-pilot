//! Flake metadata parser

use crate::error::{NpError, Result};
use crate::nix::NixExecutor;
use std::collections::HashMap;
use std::path::Path;

use super::types::{
    FlakeInput, FlakeInputLocked, FlakeInputRef, FlakeLockNode, FlakeLocks, FlakeMetadata,
    FlakeOutputs, LockNodeInput,
};

/// Parser for flake metadata and outputs
pub struct FlakeParser {
    nix: NixExecutor,
}

impl FlakeParser {
    pub fn new(nix: NixExecutor) -> Self {
        Self { nix }
    }

    /// Parse flake metadata from a flake reference
    pub async fn parse_metadata(&self, flake_ref: &str) -> Result<FlakeMetadata> {
        let json = self.nix.flake_metadata(flake_ref).await?;
        Self::parse_metadata_json(&json, flake_ref)
    }

    /// Parse flake outputs from a flake reference
    pub async fn parse_outputs(&self, flake_ref: &str) -> Result<FlakeOutputs> {
        let json = self.nix.flake_show(flake_ref).await?;
        serde_json::from_value(json).map_err(|e| NpError::FlakeParse(e.to_string()))
    }

    /// Parse metadata from raw JSON value
    pub fn parse_metadata_json(json: &serde_json::Value, flake_ref: &str) -> Result<FlakeMetadata> {
        let description = json.get("description").and_then(|v| v.as_str()).map(String::from);

        let last_modified = json
            .get("lastModified")
            .and_then(|v| v.as_i64())
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let locked_url = json.get("lockedUrl").and_then(|v| v.as_str()).map(String::from);
        let original_url = json.get("originalUrl").and_then(|v| v.as_str()).map(String::from);
        let resolved_url = json.get("resolvedUrl").and_then(|v| v.as_str()).map(String::from);
        let revision = json.get("revision").and_then(|v| v.as_str()).map(String::from);

        let path = json
            .get("path")
            .and_then(|v| v.as_str())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from(flake_ref));

        // Parse locks
        let locks = json.get("locks").and_then(|locks_json| {
            Self::parse_locks(locks_json).ok()
        });

        // Parse inputs from locks
        let inputs = locks
            .as_ref()
            .map(|l| Self::extract_inputs_from_locks(l))
            .unwrap_or_default();

        Ok(FlakeMetadata {
            description,
            last_modified,
            locked_url,
            original_url,
            path,
            resolved_url,
            revision,
            inputs,
            locks,
        })
    }

    /// Parse flake.lock content
    pub fn parse_locks(json: &serde_json::Value) -> Result<FlakeLocks> {
        let version = json.get("version").and_then(|v| v.as_i64()).unwrap_or(7) as i32;
        let root = json
            .get("root")
            .and_then(|v| v.as_str())
            .unwrap_or("root")
            .to_string();

        let nodes = json
            .get("nodes")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(name, node)| {
                        Self::parse_lock_node(node).ok().map(|n| (name.clone(), n))
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(FlakeLocks {
            version,
            root,
            nodes,
        })
    }

    /// Parse a single lock node
    fn parse_lock_node(json: &serde_json::Value) -> Result<FlakeLockNode> {
        let inputs = json.get("inputs").and_then(|v| v.as_object()).map(|obj| {
            obj.iter()
                .filter_map(|(name, value)| {
                    let input = if let Some(s) = value.as_str() {
                        LockNodeInput::Direct(s.to_string())
                    } else if let Some(arr) = value.as_array() {
                        let follows: Vec<String> = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        LockNodeInput::Follows(follows)
                    } else {
                        return None;
                    };
                    Some((name.clone(), input))
                })
                .collect()
        });

        let locked = json
            .get("locked")
            .and_then(|v| Self::parse_input_locked(v).ok());

        let original = json
            .get("original")
            .and_then(|v| Self::parse_input_ref(v).ok());

        Ok(FlakeLockNode {
            inputs,
            locked,
            original,
        })
    }

    /// Parse locked input info
    fn parse_input_locked(json: &serde_json::Value) -> Result<FlakeInputLocked> {
        Ok(FlakeInputLocked {
            input_type: json
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            owner: json.get("owner").and_then(|v| v.as_str()).map(String::from),
            repo: json.get("repo").and_then(|v| v.as_str()).map(String::from),
            rev: json.get("rev").and_then(|v| v.as_str()).map(String::from),
            nar_hash: json.get("narHash").and_then(|v| v.as_str()).map(String::from),
            last_modified: json.get("lastModified").and_then(|v| v.as_i64()),
        })
    }

    /// Parse original input reference
    fn parse_input_ref(json: &serde_json::Value) -> Result<FlakeInputRef> {
        Ok(FlakeInputRef {
            input_type: json
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            owner: json.get("owner").and_then(|v| v.as_str()).map(String::from),
            repo: json.get("repo").and_then(|v| v.as_str()).map(String::from),
            path: json.get("path").and_then(|v| v.as_str()).map(String::from),
            url: json.get("url").and_then(|v| v.as_str()).map(String::from),
            git_ref: json.get("ref").and_then(|v| v.as_str()).map(String::from),
            rev: json.get("rev").and_then(|v| v.as_str()).map(String::from),
        })
    }

    /// Extract structured inputs from parsed locks
    fn extract_inputs_from_locks(locks: &FlakeLocks) -> HashMap<String, FlakeInput> {
        let mut inputs = HashMap::new();

        // Get the root node's inputs
        if let Some(root_node) = locks.nodes.get(&locks.root) {
            if let Some(root_inputs) = &root_node.inputs {
                for (input_name, input_ref) in root_inputs {
                    let node_name = match input_ref {
                        LockNodeInput::Direct(name) => name.clone(),
                        LockNodeInput::Follows(path) => {
                            // For follows, we still want to show the input
                            let input = FlakeInput {
                                name: input_name.clone(),
                                original: FlakeInputRef {
                                    input_type: "follows".to_string(),
                                    owner: None,
                                    repo: None,
                                    path: None,
                                    url: None,
                                    git_ref: None,
                                    rev: None,
                                },
                                locked: None,
                                follows: Some(path.clone()),
                            };
                            inputs.insert(input_name.clone(), input);
                            continue;
                        }
                    };

                    if let Some(node) = locks.nodes.get(&node_name) {
                        let input = FlakeInput {
                            name: input_name.clone(),
                            original: node.original.clone().unwrap_or(FlakeInputRef {
                                input_type: "unknown".to_string(),
                                owner: None,
                                repo: None,
                                path: None,
                                url: None,
                                git_ref: None,
                                rev: None,
                            }),
                            locked: node.locked.clone(),
                            follows: None,
                        };
                        inputs.insert(input_name.clone(), input);
                    }
                }
            }
        }

        inputs
    }

    /// Read and parse flake.lock file directly
    pub async fn parse_lock_file(lock_path: &Path) -> Result<FlakeLocks> {
        if !lock_path.exists() {
            return Err(NpError::FlakeNotFound(lock_path.to_path_buf()));
        }

        let content = tokio::fs::read_to_string(lock_path).await?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        Self::parse_locks(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_input_ref() {
        let json = serde_json::json!({
            "type": "github",
            "owner": "NixOS",
            "repo": "nixpkgs",
            "ref": "nixos-24.05"
        });

        let input_ref = FlakeParser::parse_input_ref(&json).unwrap();
        assert_eq!(input_ref.input_type, "github");
        assert_eq!(input_ref.owner, Some("NixOS".to_string()));
        assert_eq!(input_ref.repo, Some("nixpkgs".to_string()));
        assert_eq!(input_ref.to_flake_ref(), "github:NixOS/nixpkgs/nixos-24.05");
    }

    #[test]
    fn test_parse_locked_input() {
        let json = serde_json::json!({
            "type": "github",
            "owner": "NixOS",
            "repo": "nixpkgs",
            "rev": "abc123",
            "narHash": "sha256-xxx",
            "lastModified": 1700000000
        });

        let locked = FlakeParser::parse_input_locked(&json).unwrap();
        assert_eq!(locked.rev, Some("abc123".to_string()));
        assert_eq!(locked.last_modified, Some(1700000000));
    }
}
