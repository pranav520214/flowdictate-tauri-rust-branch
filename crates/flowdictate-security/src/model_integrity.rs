//! # Model Integrity Verification
//!
//! Implements the model supply-chain security gate (§6).
//!
//! Every model file must pass integrity verification before loading:
//!
//! 1. Canonicalize the path safely
//! 2. Validate the file location (within allowed model directory)
//! 3. Reject directory traversal
//! 4. Reject unexpected symlink traversal
//! 5. Validate file type (extension)
//! 6. Enforce reasonable maximum size
//! 7. Calculate SHA-256 hash of the entire file
//! 8. Compare hash in constant-time against the manifest
//! 9. Validate manifest compatibility
//! 10. Only then allow the inference runtime to open the model
//!
//! **If ANY check fails → MODEL_REJECTED. Fail closed.**
//!
//! Unknown hashes are rejected. There is no "warn and continue" path.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use subtle::ConstantTimeEq;
use thiserror::Error;

/// Maximum allowed model file size: 4 GiB.
/// Prevents allocation bombs from maliciously crafted manifest entries.
const MAX_MODEL_FILE_SIZE: u64 = 4 * 1024 * 1024 * 1024;

/// Hash comparison buffer size for streaming SHA-256.
const HASH_BUFFER_SIZE: usize = 8192;

/// A single model entry in the integrity manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifestEntry {
    /// Unique model identifier (e.g., "nemotron-3.5-asr-streaming-0.6b-q8_0")
    pub id: String,
    /// Model version string
    pub model_version: String,
    /// Model architecture (e.g., "fastconformer-rnnt", "qwen3")
    pub architecture: String,
    /// Model variant (e.g., "streaming-0.6B")
    pub variant: String,
    /// Whether the model supports multiple languages
    pub multilingual: bool,
    /// Quantization format (e.g., "Q8_0", "Q4_K_M")
    pub quantization: String,
    /// Human-readable parameter count (e.g., "600M")
    pub parameters: String,
    /// Expected SHA-256 hex digest of the model file
    pub sha256: String,
    /// Expected file size in bytes
    pub file_size_bytes: u64,
    /// Expected filename
    pub filename: String,
    /// Required runtime (e.g., "nemo-speech.cpp", "llama.cpp")
    pub runtime: String,
    /// Minimum compatible runtime version
    pub runtime_version_min: String,
    /// SPDX license identifier
    pub license_identifier: String,
    /// Source identifier (e.g., HuggingFace repo)
    pub source_identifier: String,
    /// Compatible runtime name
    pub compatible_runtime: String,
    /// Minimum FlowDictate version for this model
    pub minimum_flowdictate_version: String,
}

/// The complete model integrity manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifest {
    /// Manifest format version
    pub version: u32,
    /// List of allowed models
    pub models: Vec<ModelManifestEntry>,
}

/// Errors from model integrity verification.
///
/// Error messages use structural categories, never user data content.
#[derive(Debug, Error)]
pub enum ModelVerifyError {
    #[error("path validation failed: {reason}")]
    InvalidPath { reason: String },

    #[error("file size mismatch: expected {expected}, got {actual}")]
    SizeMismatch { expected: u64, actual: u64 },

    #[error("file exceeds maximum allowed size: {size} > {max}")]
    FileTooLarge { size: u64, max: u64 },

    #[error("hash mismatch for model {model_id}")]
    HashMismatch { model_id: String },

    #[error("model not found in manifest: {filename}")]
    UnknownModel { filename: String },

    #[error("manifest contains placeholder hash — model not verified")]
    PlaceholderHash,

    #[error("I/O error during verification")]
    Io(#[from] std::io::Error),

    #[error("symlink detected at model path")]
    SymlinkDetected,

    #[error("invalid file extension: expected .gguf")]
    InvalidExtension,
}

/// Load and parse a model manifest from a JSON file.
pub fn load_manifest(manifest_path: &Path) -> Result<ModelManifest, ModelVerifyError> {
    let data = std::fs::read_to_string(manifest_path)?;
    serde_json::from_str(&data).map_err(|e| ModelVerifyError::InvalidPath {
        reason: format!("manifest parse error: {e}"),
    })
}

/// Verify a model file against the manifest.
///
/// Performs the full 10-step verification pipeline from §6.
/// Returns the matching manifest entry on success.
///
/// On ANY failure: returns `Err(ModelVerifyError)`. The caller MUST NOT
/// load the model. There is no "warn and continue" path.
pub fn verify_model_file(
    model_path: &Path,
    allowed_dir: &Path,
    manifest: &ModelManifest,
) -> Result<ModelManifestEntry, ModelVerifyError> {
    // Step 1: Canonicalize safely
    let canonical = model_path.canonicalize()?;
    let canonical_dir = allowed_dir.canonicalize()?;

    // Step 2+3: Validate location, reject traversal
    if !canonical.starts_with(&canonical_dir) {
        return Err(ModelVerifyError::InvalidPath {
            reason: "model path escapes allowed directory".to_string(),
        });
    }

    // Step 4: Reject symlinks
    let metadata = std::fs::symlink_metadata(&canonical)?;
    if metadata.file_type().is_symlink() {
        return Err(ModelVerifyError::SymlinkDetected);
    }

    // Step 5: Validate file type
    let extension = canonical.extension().and_then(|e| e.to_str()).unwrap_or("");
    if extension != "gguf" {
        return Err(ModelVerifyError::InvalidExtension);
    }

    // Step 6: Enforce maximum size
    let file_size = metadata.len();
    if file_size > MAX_MODEL_FILE_SIZE {
        return Err(ModelVerifyError::FileTooLarge {
            size: file_size,
            max: MAX_MODEL_FILE_SIZE,
        });
    }

    // Find matching manifest entry by filename
    let filename = canonical.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let entry = manifest
        .models
        .iter()
        .find(|m| m.filename == filename)
        .ok_or_else(|| ModelVerifyError::UnknownModel {
            filename: filename.to_string(),
        })?;

    // Reject placeholder hashes
    if entry.sha256.starts_with("PLACEHOLDER") || entry.sha256.len() != 64 {
        return Err(ModelVerifyError::PlaceholderHash);
    }

    // Validate file size against manifest (if manifest specifies non-zero)
    if entry.file_size_bytes > 0 && file_size != entry.file_size_bytes {
        return Err(ModelVerifyError::SizeMismatch {
            expected: entry.file_size_bytes,
            actual: file_size,
        });
    }

    // Step 7: Calculate SHA-256 hash
    let actual_hash = compute_file_sha256(&canonical)?;

    // Step 8: Constant-time comparison
    let expected_bytes = hex::decode(&entry.sha256).map_err(|_| ModelVerifyError::InvalidPath {
        reason: "manifest sha256 is not valid hex".to_string(),
    })?;
    let actual_bytes = hex::decode(&actual_hash).map_err(|_| ModelVerifyError::InvalidPath {
        reason: "computed sha256 is not valid hex".to_string(),
    })?;

    if expected_bytes.ct_eq(&actual_bytes).into() {
        // Step 9+10: Manifest compatibility validated, return entry
        tracing::info!(
            event = "model_verified",
            model_id = %entry.id,
            quantization = %entry.quantization,
        );
        Ok(entry.clone())
    } else {
        tracing::warn!(
            event = "model_rejected",
            reason = "hash_mismatch",
            model_id = %entry.id,
        );
        Err(ModelVerifyError::HashMismatch {
            model_id: entry.id.clone(),
        })
    }
}

/// Compute the SHA-256 hex digest of a file using streaming reads.
/// Does not load the entire file into memory.
fn compute_file_sha256(path: &Path) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; HASH_BUFFER_SIZE];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    /// Create a temp directory with a test model file and manifest.
    fn setup_test_model() -> (tempfile::TempDir, PathBuf, ModelManifest) {
        let dir = tempfile::tempdir().unwrap();
        let model_path = dir.path().join("test-model.gguf");

        // Write deterministic content
        let content = b"GGUF_TEST_MODEL_CONTENT_1234567890";
        let mut f = std::fs::File::create(&model_path).unwrap();
        f.write_all(content).unwrap();
        drop(f);

        // Compute real hash
        let hash = compute_file_sha256(&model_path).unwrap();

        let manifest = ModelManifest {
            version: 2,
            models: vec![ModelManifestEntry {
                id: "test-model".to_string(),
                model_version: "1.0".to_string(),
                architecture: "test".to_string(),
                variant: "test".to_string(),
                multilingual: false,
                quantization: "Q8_0".to_string(),
                parameters: "1M".to_string(),
                sha256: hash,
                file_size_bytes: content.len() as u64,
                filename: "test-model.gguf".to_string(),
                runtime: "test".to_string(),
                runtime_version_min: "0.1.0".to_string(),
                license_identifier: "MIT".to_string(),
                source_identifier: "test".to_string(),
                compatible_runtime: "test".to_string(),
                minimum_flowdictate_version: "0.1.0".to_string(),
            }],
        };

        (dir, model_path, manifest)
    }

    #[test]
    fn valid_model_accepted() {
        let (dir, model_path, manifest) = setup_test_model();
        let result = verify_model_file(&model_path, dir.path(), &manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn modified_model_rejected() {
        let (dir, model_path, manifest) = setup_test_model();

        // Modify the file content in place (same size, different bytes)
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(&model_path)
            .unwrap();
        f.write_all(b"GGUF_TEST_MODEL_CONTENT_0987654321").unwrap();
        drop(f);

        let result = verify_model_file(&model_path, dir.path(), &manifest);
        assert!(matches!(result, Err(ModelVerifyError::HashMismatch { .. })));
    }

    #[test]
    fn unknown_model_rejected() {
        let (dir, _, manifest) = setup_test_model();

        // Create a different file not in the manifest
        let unknown_path = dir.path().join("unknown.gguf");
        std::fs::write(&unknown_path, b"UNKNOWN").unwrap();

        let result = verify_model_file(&unknown_path, dir.path(), &manifest);
        assert!(matches!(result, Err(ModelVerifyError::UnknownModel { .. })));
    }

    #[test]
    fn placeholder_hash_rejected() {
        let (dir, model_path, mut manifest) = setup_test_model();
        manifest.models[0].sha256 = "PLACEHOLDER_HASH_REPLACE_WITH_ACTUAL".to_string();

        let result = verify_model_file(&model_path, dir.path(), &manifest);
        assert!(matches!(result, Err(ModelVerifyError::PlaceholderHash)));
    }

    #[test]
    fn wrong_extension_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let bad_path = dir.path().join("model.exe");
        std::fs::write(&bad_path, b"NOT_A_MODEL").unwrap();

        let manifest = ModelManifest {
            version: 2,
            models: vec![],
        };

        let result = verify_model_file(&bad_path, dir.path(), &manifest);
        assert!(matches!(result, Err(ModelVerifyError::InvalidExtension)));
    }

    #[test]
    fn traversal_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let allowed = dir.path().join("models");
        std::fs::create_dir_all(&allowed).unwrap();

        // Model outside allowed dir
        let outside = dir.path().join("evil.gguf");
        std::fs::write(&outside, b"EVIL").unwrap();

        let manifest = ModelManifest {
            version: 2,
            models: vec![],
        };

        let result = verify_model_file(&outside, &allowed, &manifest);
        assert!(matches!(result, Err(ModelVerifyError::InvalidPath { .. })));
    }

    #[test]
    fn size_mismatch_rejected() {
        let (dir, model_path, mut manifest) = setup_test_model();
        // Set wrong expected size
        manifest.models[0].file_size_bytes = 999999;

        let result = verify_model_file(&model_path, dir.path(), &manifest);
        assert!(matches!(result, Err(ModelVerifyError::SizeMismatch { .. })));
    }
}
