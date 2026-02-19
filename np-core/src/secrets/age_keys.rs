//! Age key generation and management
//!
//! Provides functionality for generating and managing age key pairs
//! used for SOPS encryption.

use age::secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::error::{NpError, Result};

/// An age key pair (public + private key)
#[derive(Debug, Clone)]
pub struct AgeKeyPair {
    /// The public key (starts with "age1...")
    pub public_key: String,
    /// The private key (starts with "AGE-SECRET-KEY-1...")
    private_key: String,
    /// Optional comment/label
    pub comment: Option<String>,
}

impl AgeKeyPair {
    /// Generate a new age key pair
    pub fn generate() -> Result<Self> {
        let identity = age::x25519::Identity::generate();
        let public_key = identity.to_public().to_string();
        let private_key = identity.to_string().expose_secret().to_string();

        Ok(Self {
            public_key,
            private_key,
            comment: None,
        })
    }

    /// Generate a new age key pair with a comment
    pub fn generate_with_comment(comment: impl Into<String>) -> Result<Self> {
        let mut key_pair = Self::generate()?;
        key_pair.comment = Some(comment.into());
        Ok(key_pair)
    }

    /// Parse a key pair from a private key string
    pub fn from_private_key(private_key: &str) -> Result<Self> {
        let private_key = private_key.trim();

        // Parse the identity
        let identity: age::x25519::Identity = private_key
            .parse()
            .map_err(|e| NpError::Config(format!("Invalid age private key: {}", e)))?;

        let public_key = identity.to_public().to_string();

        Ok(Self {
            public_key,
            private_key: private_key.to_string(),
            comment: None,
        })
    }

    /// Get the private key (use sparingly)
    pub fn expose_private_key(&self) -> &str {
        &self.private_key
    }

    /// Format for storage in keys.txt file
    pub fn to_keys_file_format(&self) -> String {
        let mut output = String::new();

        // Add comment if present
        if let Some(comment) = &self.comment {
            output.push_str(&format!("# {}\n", comment));
        }

        // Add public key as comment
        output.push_str(&format!("# public key: {}\n", self.public_key));

        // Add private key
        output.push_str(&self.private_key);
        output.push('\n');

        output
    }
}

/// Information about a stored key (without the private key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeKeyInfo {
    /// The public key
    pub public_key: String,
    /// Optional comment/label
    pub comment: Option<String>,
    /// Path to the key file (if applicable)
    pub file_path: Option<PathBuf>,
    /// Whether this is the default key
    pub is_default: bool,
}

/// Manager for age keys
pub struct AgeKeyManager {
    /// Directory where keys are stored
    keys_dir: PathBuf,
    /// Path to the default keys file
    keys_file: PathBuf,
}

impl AgeKeyManager {
    /// Create a new key manager with default paths
    pub fn new() -> Result<Self> {
        let base_dirs = directories::BaseDirs::new()
            .ok_or_else(|| NpError::Config("Could not determine home directory".to_string()))?;

        let keys_dir = base_dirs.config_dir().join("sops/age");
        let keys_file = keys_dir.join("keys.txt");

        Ok(Self {
            keys_dir,
            keys_file,
        })
    }

    /// Create a new key manager with custom paths
    pub fn with_paths(keys_dir: impl Into<PathBuf>, keys_file: impl Into<PathBuf>) -> Self {
        Self {
            keys_dir: keys_dir.into(),
            keys_file: keys_file.into(),
        }
    }

    /// Get the keys directory
    pub fn keys_dir(&self) -> &Path {
        &self.keys_dir
    }

    /// Get the keys file path
    pub fn keys_file(&self) -> &Path {
        &self.keys_file
    }

    /// Ensure the keys directory exists
    pub fn ensure_keys_dir(&self) -> Result<()> {
        if !self.keys_dir.exists() {
            fs::create_dir_all(&self.keys_dir).map_err(|e| {
                NpError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to create keys directory: {}", e),
                ))
            })?;

            // Set restrictive permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&self.keys_dir, fs::Permissions::from_mode(0o700))
                    .map_err(|e| NpError::Io(e))?;
            }
        }
        Ok(())
    }

    /// Generate a new key pair and save it
    pub fn generate_key(&self, comment: Option<&str>) -> Result<AgeKeyPair> {
        self.ensure_keys_dir()?;

        let key_pair = match comment {
            Some(c) => AgeKeyPair::generate_with_comment(c)?,
            None => AgeKeyPair::generate()?,
        };

        // Append to keys file
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.keys_file)
            .map_err(|e| NpError::Io(e))?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.keys_file, fs::Permissions::from_mode(0o600))
                .map_err(|e| NpError::Io(e))?;
        }

        write!(file, "{}", key_pair.to_keys_file_format()).map_err(|e| NpError::Io(e))?;

        Ok(key_pair)
    }

    /// List all keys in the keys file
    pub fn list_keys(&self) -> Result<Vec<AgeKeyInfo>> {
        if !self.keys_file.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&self.keys_file).map_err(|e| NpError::Io(e))?;
        let reader = BufReader::new(file);

        let mut keys = Vec::new();
        let mut current_comment: Option<String> = None;
        let mut current_public_key: Option<String> = None;

        for line in reader.lines() {
            let line = line.map_err(|e| NpError::Io(e))?;
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if line.starts_with("# public key: ") {
                current_public_key = Some(line.strip_prefix("# public key: ").unwrap().to_string());
            } else if line.starts_with('#') {
                // This is a comment
                let comment = line.strip_prefix('#').unwrap().trim();
                if !comment.starts_with("public key:") && !comment.is_empty() {
                    current_comment = Some(comment.to_string());
                }
            } else if line.starts_with("AGE-SECRET-KEY-") {
                // This is a private key, extract public key from it
                if let Ok(key_pair) = AgeKeyPair::from_private_key(line) {
                    keys.push(AgeKeyInfo {
                        public_key: current_public_key
                            .take()
                            .unwrap_or_else(|| key_pair.public_key),
                        comment: current_comment.take(),
                        file_path: Some(self.keys_file.clone()),
                        is_default: keys.is_empty(), // First key is default
                    });
                }
            }
        }

        Ok(keys)
    }

    /// Get the default (first) key
    pub fn get_default_key(&self) -> Result<Option<AgeKeyPair>> {
        if !self.keys_file.exists() {
            return Ok(None);
        }

        let file = fs::File::open(&self.keys_file).map_err(|e| NpError::Io(e))?;
        let reader = BufReader::new(file);

        let mut current_comment: Option<String> = None;

        for line in reader.lines() {
            let line = line.map_err(|e| NpError::Io(e))?;
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if line.starts_with('#') && !line.starts_with("# public key:") {
                let comment = line.strip_prefix('#').unwrap().trim();
                if !comment.is_empty() {
                    current_comment = Some(comment.to_string());
                }
            } else if line.starts_with("AGE-SECRET-KEY-") {
                let mut key_pair = AgeKeyPair::from_private_key(line)?;
                key_pair.comment = current_comment;
                return Ok(Some(key_pair));
            }
        }

        Ok(None)
    }

    /// Get all key pairs (including private keys - use carefully)
    pub fn get_all_keys(&self) -> Result<Vec<AgeKeyPair>> {
        if !self.keys_file.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&self.keys_file).map_err(|e| NpError::Io(e))?;
        let reader = BufReader::new(file);

        let mut keys = Vec::new();
        let mut current_comment: Option<String> = None;

        for line in reader.lines() {
            let line = line.map_err(|e| NpError::Io(e))?;
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if line.starts_with('#') && !line.starts_with("# public key:") {
                let comment = line.strip_prefix('#').unwrap().trim();
                if !comment.is_empty() {
                    current_comment = Some(comment.to_string());
                }
            } else if line.starts_with("AGE-SECRET-KEY-") {
                if let Ok(mut key_pair) = AgeKeyPair::from_private_key(line) {
                    key_pair.comment = current_comment.take();
                    keys.push(key_pair);
                }
            }
        }

        Ok(keys)
    }

    /// Check if any keys exist
    pub fn has_keys(&self) -> bool {
        self.keys_file.exists()
            && fs::read_to_string(&self.keys_file)
                .map(|s| s.contains("AGE-SECRET-KEY-"))
                .unwrap_or(false)
    }

    /// Import a private key
    pub fn import_key(&self, private_key: &str, comment: Option<&str>) -> Result<AgeKeyPair> {
        self.ensure_keys_dir()?;

        let mut key_pair = AgeKeyPair::from_private_key(private_key)?;
        key_pair.comment = comment.map(|s| s.to_string());

        // Append to keys file
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.keys_file)
            .map_err(|e| NpError::Io(e))?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.keys_file, fs::Permissions::from_mode(0o600))
                .map_err(|e| NpError::Io(e))?;
        }

        write!(file, "{}", key_pair.to_keys_file_format()).map_err(|e| NpError::Io(e))?;

        Ok(key_pair)
    }

    /// Get all public keys (for .sops.yaml configuration)
    pub fn get_public_keys(&self) -> Result<Vec<String>> {
        Ok(self
            .list_keys()?
            .into_iter()
            .map(|k| k.public_key)
            .collect())
    }
}

impl Default for AgeKeyManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default AgeKeyManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_key_pair() {
        let key_pair = AgeKeyPair::generate().unwrap();
        assert!(key_pair.public_key.starts_with("age1"));
        assert!(key_pair.private_key.starts_with("AGE-SECRET-KEY-1"));
    }

    #[test]
    fn test_key_pair_round_trip() {
        let original = AgeKeyPair::generate().unwrap();
        let restored = AgeKeyPair::from_private_key(&original.private_key).unwrap();
        assert_eq!(original.public_key, restored.public_key);
    }

    #[test]
    fn test_key_manager() {
        let temp_dir = TempDir::new().unwrap();
        let keys_dir = temp_dir.path().join("age");
        let keys_file = keys_dir.join("keys.txt");

        let manager = AgeKeyManager::with_paths(&keys_dir, &keys_file);

        // Generate a key
        let key = manager.generate_key(Some("test key")).unwrap();
        assert!(key.public_key.starts_with("age1"));

        // List keys
        let keys = manager.list_keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].comment, Some("test key".to_string()));

        // Generate another key
        manager.generate_key(Some("second key")).unwrap();

        let keys = manager.list_keys().unwrap();
        assert_eq!(keys.len(), 2);
    }
}
