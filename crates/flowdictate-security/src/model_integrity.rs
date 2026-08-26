//! # Model Integrity Verification
//!
//! Implements the model supply-chain security gate described in `docs/MODEL_SECURITY.md`.
//!
//! Every model file must pass integrity verification before loading:
//!
//! 1. Validate path (no traversal, within allowed model directory)
//! 2. Validate file size (matches manifest ±tolerance)
//! 3. Calculate SHA-256 hash of the entire file
//! 4. Compare hash against the compiled-in/signed allowlist manifest
//!
//! **If ANY check fails → DO NOT LOAD. Fail closed.**
//!
//! Unknown hashes are rejected. There is no "warn and continue" path.
//!
//! ## Manifest Format
//!
//! The manifest is a JSON document containing model metadata and expected hashes.
//! It can be compiled into the binary or loaded from a verified location.

use serde::{Deserialize, Serialize};

/// A single model entry in the integrity manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifestEntry {
    /// Unique model identifier (e.g., "whisper-base-q5_1")
    pub id: String,
    /// Model architecture (e.g., "whisper", "qwen2")
    pub architecture: String,
    /// Model variant (e.g., "base", "0.5B-Instruct")
    pub variant: String,
    /// Whether the model supports multiple languages
    pub multilingual: bool,
    /// Quantization format (e.g., "q5_1", "Q4_K_M")
    pub quantization: String,
    /// Human-readable parameter count (e.g., "74M")
    pub parameters: String,
    /// Expected SHA-256 hex digest of the model file
    pub sha256: String,
    /// Expected file size in bytes
    pub file_size_bytes: u64,
    /// Expected filename
    pub filename: String,
    /// Required runtime (e.g., "whisper.cpp", "llama.cpp")
    pub runtime: String,
    /// Minimum compatible runtime version
    pub runtime_version_min: String,
}

/// The complete model integrity manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifest {
    /// Manifest format version
    pub version: u32,
    /// List of allowed models
    pub models: Vec<ModelManifestEntry>,
}

/// Result of model integrity verification.
#[derive(Debug)]
pub enum VerificationResult {
    /// Model passed all integrity checks
    Valid,
    /// Path validation failed (traversal attempt, symlink, etc.)
    InvalidPath(String),
    /// File size does not match manifest
    SizeMismatch { expected: u64, actual: u64 },
    /// SHA-256 hash does not match any allowed model
    HashMismatch { expected: String, actual: String },
    /// Model ID not found in manifest
    UnknownModel(String),
    /// File could not be read
    IoError(String),
}

// TODO(milestone-2): Implement verify_model_file()
// TODO(milestone-2): Implement load_manifest()
// TODO(milestone-2): Implement compiled-in manifest embedding
