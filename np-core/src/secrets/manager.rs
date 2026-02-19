//! SOPS-compatible secrets manager
//!
//! Provides functionality for encrypting and decrypting secrets using age,
//! compatible with SOPS format for use with sops-nix.

use age::armor::{ArmoredReader, ArmoredWriter, Format};
use age::{Decryptor, Encryptor};
use secrecy::SecretString;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use super::age_keys::{AgeKeyManager, AgeKeyPair};
use super::types::*;
use crate::error::{NpError, Result};

/// Manager for SOPS-compatible secrets
pub struct SecretsManager {
    /// Configuration
    config: SecretsConfig,
    /// Age key manager
    key_manager: AgeKeyManager,
    /// Cached identities for decryption
    identities: Vec<age::x25519::Identity>,
}

impl SecretsManager {
    /// Create a new secrets manager with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(SecretsConfig::default())
    }

    /// Create a new secrets manager with custom configuration
    pub fn with_config(config: SecretsConfig) -> Result<Self> {
        let key_manager = AgeKeyManager::with_paths(
            config.age_key_path.parent().unwrap_or(Path::new(".")),
            &config.age_key_path,
        );

        let identities = Self::load_identities(&key_manager)?;

        Ok(Self {
            config,
            key_manager,
            identities,
        })
    }

    /// Load age identities from the key manager
    fn load_identities(key_manager: &AgeKeyManager) -> Result<Vec<age::x25519::Identity>> {
        let keys = key_manager.get_all_keys()?;
        let mut identities = Vec::new();

        for key in keys {
            match key.expose_private_key().parse() {
                Ok(identity) => identities.push(identity),
                Err(e) => {
                    warn!("Failed to parse age key: {}", e);
                }
            }
        }

        Ok(identities)
    }

    /// Reload identities from disk
    pub fn reload_keys(&mut self) -> Result<()> {
        self.identities = Self::load_identities(&self.key_manager)?;
        Ok(())
    }

    /// Get the key manager
    pub fn key_manager(&self) -> &AgeKeyManager {
        &self.key_manager
    }

    /// Get the configuration
    pub fn config(&self) -> &SecretsConfig {
        &self.config
    }

    /// Ensure a key exists, generating one if needed
    pub fn ensure_key(&self, comment: Option<&str>) -> Result<AgeKeyPair> {
        if self.key_manager.has_keys() {
            self.key_manager
                .get_default_key()?
                .ok_or_else(|| NpError::Config("No age keys found".to_string()))
        } else {
            info!("No age keys found, generating new key");
            self.key_manager.generate_key(comment)
        }
    }

    /// Encrypt a secret value using age
    pub fn encrypt(&self, plaintext: &str, recipients: &[String]) -> Result<String> {
        if recipients.is_empty() {
            return Err(NpError::Config(
                "No recipients specified for encryption".to_string(),
            ));
        }

        // Parse recipients
        let parsed_recipients: Vec<age::x25519::Recipient> = recipients
            .iter()
            .map(|r| {
                r.parse()
                    .map_err(|e| NpError::Config(format!("Invalid age recipient '{}': {}", r, e)))
            })
            .collect::<Result<Vec<_>>>()?;

        let recipients_refs: Vec<&dyn age::Recipient> = parsed_recipients
            .iter()
            .map(|r| r as &dyn age::Recipient)
            .collect();

        // Create encryptor
        let encryptor = Encryptor::with_recipients(recipients_refs.into_iter())
            .map_err(|e| NpError::Encryption(format!("Failed to create encryptor: {}", e)))?;

        // Encrypt to armored format
        let mut encrypted = Vec::new();
        let armor_writer = ArmoredWriter::wrap_output(&mut encrypted, Format::AsciiArmor)
            .map_err(|e| NpError::Encryption(format!("Failed to create armor writer: {}", e)))?;

        let mut writer = encryptor
            .wrap_output(armor_writer)
            .map_err(|e| NpError::Encryption(format!("Failed to wrap output: {}", e)))?;

        writer
            .write_all(plaintext.as_bytes())
            .map_err(|e| NpError::Io(e))?;

        writer
            .finish()
            .and_then(|w| w.finish())
            .map_err(|e| NpError::Encryption(format!("Failed to finish encryption: {}", e)))?;

        Ok(String::from_utf8_lossy(&encrypted).to_string())
    }

    /// Decrypt a secret value using age
    pub fn decrypt(&self, ciphertext: &str) -> Result<String> {
        if self.identities.is_empty() {
            return Err(NpError::Config(
                "No age identities available for decryption".to_string(),
            ));
        }

        let armor_reader = ArmoredReader::new(ciphertext.as_bytes());

        let decryptor = Decryptor::new(armor_reader)
            .map_err(|e| NpError::Decryption(format!("Failed to create decryptor: {}", e)))?;

        // Check if it's passphrase encrypted
        if decryptor.is_scrypt() {
            return Err(NpError::Decryption(
                "Passphrase-encrypted secrets not supported".to_string(),
            ));
        }

        let identity_refs: Vec<&dyn age::Identity> = self
            .identities
            .iter()
            .map(|i| i as &dyn age::Identity)
            .collect();

        let mut reader = decryptor
            .decrypt(identity_refs.into_iter())
            .map_err(|e| NpError::Decryption(format!("Failed to decrypt: {}", e)))?;

        let mut plaintext = String::new();
        reader
            .read_to_string(&mut plaintext)
            .map_err(|e| NpError::Io(e))?;

        Ok(plaintext)
    }

    /// Create a SOPS-compatible encrypted file
    pub fn create_secret_file(
        &self,
        path: &Path,
        data: &HashMap<String, String>,
        recipients: &[String],
    ) -> Result<()> {
        let recipients = if recipients.is_empty() {
            &self.config.default_recipients
        } else {
            recipients
        };

        if recipients.is_empty() {
            return Err(NpError::Config(
                "No recipients specified and no defaults configured".to_string(),
            ));
        }

        // Encrypt each value
        let mut encrypted_data: HashMap<String, serde_yaml::Value> = HashMap::new();

        for (key, value) in data {
            let encrypted = self.encrypt(value, recipients)?;
            encrypted_data.insert(key.clone(), serde_yaml::Value::String(encrypted));
        }

        // Create SOPS metadata
        let sops_metadata = SopsMetadata {
            age: recipients
                .iter()
                .map(|r| AgeRecipient {
                    recipient: r.clone(),
                    enc: None,
                })
                .collect(),
            last_modified: Some(chrono::Utc::now().to_rfc3339()),
            version: Some("3.8.1".to_string()),
            ..Default::default()
        };

        // Combine into SOPS format
        encrypted_data.insert(
            "sops".to_string(),
            serde_yaml::to_value(&sops_metadata)
                .map_err(|e| NpError::Encryption(format!("Failed to serialize SOPS metadata: {}", e)))?,
        );

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| NpError::Io(e))?;
        }

        // Write to file
        let yaml = serde_yaml::to_string(&encrypted_data)
            .map_err(|e| NpError::Encryption(format!("Failed to serialize to YAML: {}", e)))?;

        fs::write(path, yaml).map_err(|e| NpError::Io(e))?;

        info!("Created encrypted secret file: {}", path.display());
        Ok(())
    }

    /// Read and decrypt a SOPS-compatible file
    pub fn read_secret_file(&self, path: &Path) -> Result<HashMap<String, String>> {
        let content = fs::read_to_string(path).map_err(|e| NpError::Io(e))?;

        let data: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(&content)
            .map_err(|e| NpError::Decryption(format!("Failed to parse YAML: {}", e)))?;

        let mut decrypted: HashMap<String, String> = HashMap::new();

        for (key, value) in data {
            // Skip SOPS metadata
            if key == "sops" {
                continue;
            }

            if let serde_yaml::Value::String(encrypted) = value {
                // Check if it's actually encrypted (age armored format)
                if encrypted.starts_with("-----BEGIN AGE ENCRYPTED FILE-----") {
                    match self.decrypt(&encrypted) {
                        Ok(plaintext) => {
                            decrypted.insert(key, plaintext);
                        }
                        Err(e) => {
                            warn!("Failed to decrypt key '{}': {}", key, e);
                            // Include as-is if decryption fails
                            decrypted.insert(key, encrypted);
                        }
                    }
                } else {
                    // Not encrypted, include as-is
                    decrypted.insert(key, encrypted);
                }
            }
        }

        Ok(decrypted)
    }

    /// Create a new secret and save it
    pub fn create_secret(&self, request: &CreateSecretRequest) -> Result<SecretSummary> {
        let id = SecretId::new(uuid::Uuid::new_v4().to_string());

        // Determine file path based on secret type and environment
        let file_name = format!("{}.yaml", sanitize_filename(&request.name));
        let relative_path = match &request.environment {
            Some(env) => PathBuf::from("env").join(env).join(&file_name),
            None => PathBuf::from(&file_name),
        };
        let file_path = self.config.secrets_dir.join(&relative_path);

        // Prepare data
        let mut data = HashMap::new();
        data.insert("value".to_string(), request.value.clone());

        // Include metadata
        let metadata = SecretMetadata {
            tags: request.tags.clone(),
            machines: request.machines.clone(),
            environment: request.environment.clone(),
            ..Default::default()
        };
        data.insert(
            "_metadata".to_string(),
            serde_json::to_string(&metadata)
                .map_err(|e| NpError::Encryption(format!("Failed to serialize metadata: {}", e)))?,
        );

        // Get recipients
        let recipients = self.key_manager.get_public_keys()?;
        if recipients.is_empty() {
            return Err(NpError::Config(
                "No age keys available. Generate a key first.".to_string(),
            ));
        }

        // Create the encrypted file
        self.create_secret_file(&file_path, &data, &recipients)?;

        Ok(SecretSummary {
            id,
            name: request.name.clone(),
            description: request.description.clone(),
            secret_type: request.secret_type,
            metadata,
            file_path,
        })
    }

    /// List all secrets in the secrets directory
    pub fn list_secrets(&self) -> Result<Vec<SecretSummary>> {
        let secrets_dir = &self.config.secrets_dir;

        if !secrets_dir.exists() {
            return Ok(Vec::new());
        }

        let mut secrets = Vec::new();

        for entry in walkdir::WalkDir::new(secrets_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
                if let Ok(summary) = self.read_secret_summary(path) {
                    secrets.push(summary);
                }
            }
        }

        Ok(secrets)
    }

    /// Read secret summary (metadata only, not the value)
    fn read_secret_summary(&self, path: &Path) -> Result<SecretSummary> {
        let content = fs::read_to_string(path).map_err(|e| NpError::Io(e))?;

        let data: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(&content)
            .map_err(|e| NpError::Decryption(format!("Failed to parse YAML: {}", e)))?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let id = SecretId::new(format!(
            "{}",
            path.strip_prefix(&self.config.secrets_dir)
                .unwrap_or(path)
                .display()
        ));

        // Try to extract metadata if it exists
        let metadata = if let Some(serde_yaml::Value::String(encrypted_meta)) =
            data.get("_metadata")
        {
            if encrypted_meta.starts_with("-----BEGIN AGE") {
                // Encrypted, try to decrypt
                self.decrypt(encrypted_meta.as_str())
                    .ok()
                    .and_then(|json| serde_json::from_str(&json).ok())
                    .unwrap_or_default()
            } else {
                serde_json::from_str(encrypted_meta.as_str()).unwrap_or_default()
            }
        } else {
            SecretMetadata::default()
        };

        Ok(SecretSummary {
            id,
            name,
            description: None,
            secret_type: SecretType::Text,
            metadata,
            file_path: path.to_path_buf(),
        })
    }

    /// Get a secret by ID
    pub fn get_secret(&self, id: &SecretId) -> Result<Secret> {
        let file_path = self.config.secrets_dir.join(id.as_str());

        if !file_path.exists() {
            return Err(NpError::SecretNotFound(id.to_string()));
        }

        let decrypted = self.read_secret_file(&file_path)?;

        let value = decrypted
            .get("value")
            .cloned()
            .unwrap_or_default();

        let name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Secret {
            id: id.clone(),
            name,
            description: None,
            secret_type: SecretType::Text,
            value: SecretString::from(value),
            metadata: SecretMetadata::default(),
        })
    }

    /// Delete a secret
    pub fn delete_secret(&self, id: &SecretId) -> Result<()> {
        let file_path = self.config.secrets_dir.join(id.as_str());

        if !file_path.exists() {
            return Err(NpError::SecretNotFound(id.to_string()));
        }

        fs::remove_file(&file_path).map_err(|e| NpError::Io(e))?;

        info!("Deleted secret: {}", id);
        Ok(())
    }

    /// Update .sops.yaml with current public keys
    pub fn update_sops_config(&self) -> Result<()> {
        let public_keys = self.key_manager.get_public_keys()?;

        if public_keys.is_empty() {
            warn!("No public keys to add to .sops.yaml");
            return Ok(());
        }

        // Read existing config
        let config_path = &self.config.sops_config_path;
        let content = if config_path.exists() {
            fs::read_to_string(config_path).map_err(|e| NpError::Io(e))?
        } else {
            include_str!("../../.sops.yaml.template").to_string()
        };

        // Parse and update (simplified - in production would properly parse YAML)
        let updated = content.replace(
            "age: []",
            &format!(
                "age:\n          - {}",
                public_keys.join("\n          - ")
            ),
        );

        fs::write(config_path, updated).map_err(|e| NpError::Io(e))?;

        info!(
            "Updated {} with {} public keys",
            config_path.display(),
            public_keys.len()
        );
        Ok(())
    }

    /// Generate sops-nix configuration for a secret
    pub fn generate_sops_nix_config(&self, secret: &SecretFile) -> String {
        format!(
            r#"sops.secrets."{}" = {{
  sopsFile = {};
  owner = "{}";
  group = "{}";
  mode = "{}";
  {}
}};"#,
            secret.target_path.display(),
            secret.source_path.display(),
            secret.owner,
            secret.group,
            secret.mode,
            if secret.restart_units.is_empty() {
                String::new()
            } else {
                format!(
                    "restartUnits = [ {} ];",
                    secret
                        .restart_units
                        .iter()
                        .map(|u| format!("\"{}\"", u))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
        )
    }
}

/// Sanitize a filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

// We need walkdir for directory traversal
// This is a simple inline implementation to avoid adding dependency
mod walkdir {
    use std::fs;
    use std::path::{Path, PathBuf};

    pub struct WalkDir {
        root: PathBuf,
        follow_links: bool,
    }

    impl WalkDir {
        pub fn new(root: impl Into<PathBuf>) -> Self {
            Self {
                root: root.into(),
                follow_links: false,
            }
        }

        pub fn follow_links(mut self, _yes: bool) -> Self {
            self.follow_links = true;
            self
        }

        pub fn into_iter(self) -> impl Iterator<Item = Result<DirEntry, std::io::Error>> {
            WalkDirIter {
                stack: vec![self.root],
            }
        }
    }

    pub struct DirEntry {
        path: PathBuf,
    }

    impl DirEntry {
        pub fn path(&self) -> &Path {
            &self.path
        }
    }

    struct WalkDirIter {
        stack: Vec<PathBuf>,
    }

    impl Iterator for WalkDirIter {
        type Item = Result<DirEntry, std::io::Error>;

        fn next(&mut self) -> Option<Self::Item> {
            while let Some(path) = self.stack.pop() {
                if path.is_dir() {
                    match fs::read_dir(&path) {
                        Ok(entries) => {
                            for entry in entries.flatten() {
                                self.stack.push(entry.path());
                            }
                        }
                        Err(e) => return Some(Err(e)),
                    }
                } else {
                    return Some(Ok(DirEntry { path }));
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_manager() -> (SecretsManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let keys_dir = temp_dir.path().join("keys");
        let secrets_dir = temp_dir.path().join("secrets");

        fs::create_dir_all(&keys_dir).unwrap();
        fs::create_dir_all(&secrets_dir).unwrap();

        let config = SecretsConfig {
            secrets_dir,
            sops_config_path: temp_dir.path().join(".sops.yaml"),
            age_key_path: keys_dir.join("keys.txt"),
            default_recipients: Vec::new(),
            use_sops_cli: false,
        };

        let manager = SecretsManager::with_config(config).unwrap();

        // Generate a test key
        manager.key_manager().generate_key(Some("test")).unwrap();

        // Reload keys
        let mut manager = manager;
        manager.reload_keys().unwrap();

        (manager, temp_dir)
    }

    #[test]
    fn test_encrypt_decrypt() {
        let (manager, _temp_dir) = setup_test_manager();

        let recipients = manager.key_manager().get_public_keys().unwrap();
        assert!(!recipients.is_empty());

        let plaintext = "super secret value";
        let encrypted = manager.encrypt(plaintext, &recipients).unwrap();

        assert!(encrypted.contains("-----BEGIN AGE ENCRYPTED FILE-----"));

        let decrypted = manager.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_create_and_read_secret_file() {
        let (manager, _temp_dir) = setup_test_manager();

        let mut data = HashMap::new();
        data.insert("password".to_string(), "secret123".to_string());
        data.insert("api_key".to_string(), "key456".to_string());

        let recipients = manager.key_manager().get_public_keys().unwrap();
        let file_path = manager.config().secrets_dir.join("test.yaml");

        manager
            .create_secret_file(&file_path, &data, &recipients)
            .unwrap();

        assert!(file_path.exists());

        let decrypted = manager.read_secret_file(&file_path).unwrap();
        assert_eq!(decrypted.get("password"), Some(&"secret123".to_string()));
        assert_eq!(decrypted.get("api_key"), Some(&"key456".to_string()));
    }
}
