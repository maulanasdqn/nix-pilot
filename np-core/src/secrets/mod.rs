//! SOPS-based secret management module
//!
//! Provides functionality for:
//! - Age key generation and management
//! - Secret encryption/decryption using age
//! - SOPS-compatible file handling
//! - Secure secret storage

pub mod age_keys;
pub mod manager;
pub mod types;

#[cfg(test)]
mod tests;

pub use age_keys::{AgeKeyInfo, AgeKeyManager, AgeKeyPair};
pub use manager::SecretsManager;
pub use types::{
    CreateSecretRequest, EncryptedSecret, Secret, SecretFile, SecretId, SecretMetadata,
    SecretSummary, SecretType, SecretsConfig, SopsMetadata, UpdateSecretRequest,
};
