//! Flake management operations

use crate::error::{NpError, Result};
use crate::nix::{NixExecutor, OutputLine};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

use super::parser::FlakeParser;
use super::types::{
    CreateFlakeRequest, FlakeId, FlakeMetadata, FlakeOutputs, RegisteredFlake, UpdateFlakeRequest,
    UpdateInputRequest,
};

/// Manager for flake operations
pub struct FlakeManager {
    data_dir: PathBuf,
    parser: FlakeParser,
    nix: NixExecutor,
}

impl FlakeManager {
    pub fn new(data_dir: PathBuf, nix: NixExecutor) -> Self {
        let parser = FlakeParser::new(nix.clone());
        Self {
            data_dir,
            parser,
            nix,
        }
    }

    /// Get the path to the flakes storage file
    fn flakes_file(&self) -> PathBuf {
        self.data_dir.join("flakes.json")
    }

    /// Load all registered flakes
    async fn load_flakes(&self) -> Result<HashMap<FlakeId, RegisteredFlake>> {
        let path = self.flakes_file();
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let flakes: HashMap<FlakeId, RegisteredFlake> = serde_json::from_str(&content)?;
        Ok(flakes)
    }

    /// Save all registered flakes
    async fn save_flakes(&self, flakes: &HashMap<FlakeId, RegisteredFlake>) -> Result<()> {
        tokio::fs::create_dir_all(&self.data_dir).await?;
        let content = serde_json::to_string_pretty(flakes)?;
        tokio::fs::write(self.flakes_file(), content).await?;
        Ok(())
    }

    /// List all registered flakes
    pub async fn list(&self) -> Result<Vec<RegisteredFlake>> {
        let flakes = self.load_flakes().await?;
        let mut list: Vec<_> = flakes.into_values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(list)
    }

    /// Get a specific flake by ID
    pub async fn get(&self, id: &FlakeId) -> Result<RegisteredFlake> {
        let flakes = self.load_flakes().await?;
        flakes
            .get(id)
            .cloned()
            .ok_or_else(|| NpError::FlakeNotFound(PathBuf::from(&id.0)))
    }

    /// Register a new flake
    pub async fn create(&self, request: CreateFlakeRequest) -> Result<RegisteredFlake> {
        // Verify the flake exists and is valid
        if !request.path.exists() {
            return Err(NpError::FlakeNotFound(request.path.clone()));
        }

        let flake_nix = request.path.join("flake.nix");
        if !flake_nix.exists() {
            return Err(NpError::FlakeParse(format!(
                "No flake.nix found at {:?}",
                request.path
            )));
        }

        // Parse initial metadata
        let flake_ref = request.path.to_string_lossy().to_string();
        let metadata = self.parser.parse_metadata(&flake_ref).await.ok();

        let now = Utc::now();
        let flake = RegisteredFlake {
            id: FlakeId::new(),
            name: request.name,
            path: request.path,
            description: request.description,
            metadata,
            created_at: now,
            updated_at: now,
        };

        let mut flakes = self.load_flakes().await?;
        flakes.insert(flake.id.clone(), flake.clone());
        self.save_flakes(&flakes).await?;

        Ok(flake)
    }

    /// Update a registered flake
    pub async fn update(&self, id: &FlakeId, request: UpdateFlakeRequest) -> Result<RegisteredFlake> {
        let mut flakes = self.load_flakes().await?;

        let flake = flakes
            .get_mut(id)
            .ok_or_else(|| NpError::FlakeNotFound(PathBuf::from(&id.0)))?;

        if let Some(name) = request.name {
            flake.name = name;
        }
        if let Some(desc) = request.description {
            flake.description = Some(desc);
        }
        flake.updated_at = Utc::now();

        let result = flake.clone();
        self.save_flakes(&flakes).await?;
        Ok(result)
    }

    /// Delete a registered flake
    pub async fn delete(&self, id: &FlakeId) -> Result<()> {
        let mut flakes = self.load_flakes().await?;
        if flakes.remove(id).is_none() {
            return Err(NpError::FlakeNotFound(PathBuf::from(&id.0)));
        }
        self.save_flakes(&flakes).await?;
        Ok(())
    }

    /// Refresh metadata for a registered flake
    pub async fn refresh_metadata(&self, id: &FlakeId) -> Result<RegisteredFlake> {
        let mut flakes = self.load_flakes().await?;

        let flake = flakes
            .get_mut(id)
            .ok_or_else(|| NpError::FlakeNotFound(PathBuf::from(&id.0)))?;

        let flake_ref = flake.path.to_string_lossy().to_string();
        let metadata = self.parser.parse_metadata(&flake_ref).await?;

        flake.metadata = Some(metadata);
        flake.updated_at = Utc::now();

        let result = flake.clone();
        self.save_flakes(&flakes).await?;
        Ok(result)
    }

    /// Get flake outputs
    pub async fn get_outputs(&self, id: &FlakeId) -> Result<FlakeOutputs> {
        let flake = self.get(id).await?;
        let flake_ref = flake.path.to_string_lossy().to_string();
        self.parser.parse_outputs(&flake_ref).await
    }

    /// Get metadata for a flake path (not necessarily registered)
    pub async fn get_metadata(&self, flake_ref: &str) -> Result<FlakeMetadata> {
        self.parser.parse_metadata(flake_ref).await
    }

    /// Update flake lock with streaming output
    pub async fn update_lock(
        &self,
        flake_path: &Path,
        input: Option<&str>,
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<i32> {
        let path_str = flake_path.to_string_lossy().to_string();
        self.nix.flake_update(&path_str, input, tx).await
    }

    /// Update a flake input by modifying flake.nix
    pub async fn update_input(
        &self,
        flake_path: &Path,
        input_name: &str,
        request: UpdateInputRequest,
    ) -> Result<()> {
        let flake_nix = flake_path.join("flake.nix");
        if !flake_nix.exists() {
            return Err(NpError::FlakeNotFound(flake_path.to_path_buf()));
        }

        let content = tokio::fs::read_to_string(&flake_nix).await?;

        let new_content = if let Some(new_ref) = request.flake_ref {
            self.update_input_ref(&content, input_name, &new_ref)?
        } else if let Some(follows) = request.follows {
            self.update_input_follows(&content, input_name, &follows)?
        } else {
            return Ok(()); // Nothing to update
        };

        tokio::fs::write(&flake_nix, new_content).await?;
        Ok(())
    }

    /// Update an input's URL/ref in flake.nix content
    fn update_input_ref(&self, content: &str, input_name: &str, new_ref: &str) -> Result<String> {
        // Look for patterns like:
        //   inputName.url = "...";
        //   inputName = { url = "..."; };
        //   inputs.inputName.url = "...";

        // Pattern 1: inputName.url = "...";
        let pattern1 = format!(
            r#"(\b{}\.url\s*=\s*)"[^"]*""#,
            regex::escape(input_name)
        );
        let re1 = regex::Regex::new(&pattern1).map_err(|e| NpError::FlakeParse(e.to_string()))?;

        if re1.is_match(content) {
            let replacement = format!(r#"${{1}}"{}""#, new_ref);
            return Ok(re1.replace(content, replacement.as_str()).to_string());
        }

        // Pattern 2: inputs.inputName.url = "...";
        let pattern2 = format!(
            r#"(inputs\.{}\.url\s*=\s*)"[^"]*""#,
            regex::escape(input_name)
        );
        let re2 = regex::Regex::new(&pattern2).map_err(|e| NpError::FlakeParse(e.to_string()))?;

        if re2.is_match(content) {
            let replacement = format!(r#"${{1}}"{}""#, new_ref);
            return Ok(re2.replace(content, replacement.as_str()).to_string());
        }

        // Pattern 3: inputName = "..."; (shorthand)
        let pattern3 = format!(
            r#"(\b{}\s*=\s*)"[^"]*""#,
            regex::escape(input_name)
        );
        let re3 = regex::Regex::new(&pattern3).map_err(|e| NpError::FlakeParse(e.to_string()))?;

        if re3.is_match(content) {
            let replacement = format!(r#"${{1}}"{}""#, new_ref);
            return Ok(re3.replace(content, replacement.as_str()).to_string());
        }

        Err(NpError::FlakeParse(format!(
            "Could not find input '{}' in flake.nix",
            input_name
        )))
    }

    /// Update an input to follow another input
    fn update_input_follows(
        &self,
        content: &str,
        input_name: &str,
        follows: &[String],
    ) -> Result<String> {
        // This is more complex as we need to add/modify the follows attribute
        // For now, we'll handle the simple case of replacing url with follows

        let follows_str = follows.join(".");

        // Look for inputName.url = "..."; and replace with inputName.follows = "...";
        let pattern = format!(
            r#"({}\.)(url)(\s*=\s*)"[^"]*""#,
            regex::escape(input_name)
        );
        let re = regex::Regex::new(&pattern).map_err(|e| NpError::FlakeParse(e.to_string()))?;

        if re.is_match(content) {
            let replacement = format!(r#"${{1}}follows${{3}}"{}""#, follows_str);
            return Ok(re.replace(content, replacement.as_str()).to_string());
        }

        // Try inputs.inputName.url pattern
        let pattern2 = format!(
            r#"(inputs\.{}\.)(url)(\s*=\s*)"[^"]*""#,
            regex::escape(input_name)
        );
        let re2 = regex::Regex::new(&pattern2).map_err(|e| NpError::FlakeParse(e.to_string()))?;

        if re2.is_match(content) {
            let replacement = format!(r#"${{1}}follows${{3}}"{}""#, follows_str);
            return Ok(re2.replace(content, replacement.as_str()).to_string());
        }

        Err(NpError::FlakeParse(format!(
            "Could not find input '{}' URL to convert to follows",
            input_name
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_input_ref_simple() {
        let manager = FlakeManager::new(
            PathBuf::from("/tmp"),
            NixExecutor::from_path().unwrap(),
        );

        let content = r#"
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
  };
}
"#;

        let result = manager
            .update_input_ref(content, "nixpkgs", "github:NixOS/nixpkgs/nixos-24.05")
            .unwrap();

        assert!(result.contains("github:NixOS/nixpkgs/nixos-24.05"));
        assert!(!result.contains("nixos-23.11"));
    }

    #[test]
    fn test_update_input_follows() {
        let manager = FlakeManager::new(
            PathBuf::from("/tmp"),
            NixExecutor::from_path().unwrap(),
        );

        let content = r#"
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    home-manager.url = "github:nix-community/home-manager";
  };
}
"#;

        let result = manager
            .update_input_follows(content, "home-manager", &["nixpkgs".to_string()])
            .unwrap();

        assert!(result.contains("home-manager.follows"));
        assert!(result.contains("\"nixpkgs\""));
    }
}
