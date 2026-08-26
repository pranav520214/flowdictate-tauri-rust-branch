//! # Core Error Types
//!
//! Aggregates errors from all sub-crates into a unified error type.
//! Error messages MUST NOT contain transcript text, audio data, or secrets.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlowDictateError {
    #[error("audio subsystem error: {0}")]
    Audio(String),

    #[error("ASR engine error: {0}")]
    Asr(String),

    #[error("refinement error: {0}")]
    Refinement(String),

    #[error("text injection error: {0}")]
    Injection(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("security error: {0}")]
    Security(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("model integrity verification failed")]
    ModelIntegrityFailure,
}
