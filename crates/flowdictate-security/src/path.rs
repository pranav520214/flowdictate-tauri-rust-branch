//! # Path Validation & Traversal Prevention
//!
//! All filesystem paths derived from user input or external sources must be
//! validated before use. This module prevents:
//!
//! - `../` directory traversal attacks
//! - Symlink-based escapes from allowed directories
//! - Unsafe temporary file creation
//! - Arbitrary file overwrites
//! - Extension confusion attacks
//!
//! ## Usage
//!
//! Before loading a model file, importing a profile, or writing any file based
//! on user-provided names, validate the path through this module.

use std::path::PathBuf;
use thiserror::Error;

/// Errors from path validation.
#[derive(Debug, Error)]
pub enum PathValidationError {
    #[error("path contains traversal component: {0}")]
    TraversalDetected(String),

    #[error("path is a symlink: {0}")]
    SymlinkDetected(PathBuf),

    #[error("path escapes allowed directory: {path} is not within {allowed_root}")]
    EscapesAllowedDirectory {
        path: PathBuf,
        allowed_root: PathBuf,
    },

    #[error("path contains null bytes")]
    NullByte,

    #[error("path is not valid UTF-8")]
    InvalidUtf8,

    #[error("IO error validating path: {0}")]
    Io(#[from] std::io::Error),
}

// TODO(milestone-2): Implement validate_model_path()
// TODO(milestone-7): Implement validate_profile_import_path()
// TODO(milestone-1): Implement safe_temp_file() using OS-provided temp directory
