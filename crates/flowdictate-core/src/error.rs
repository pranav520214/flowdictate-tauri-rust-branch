//! # Core Error Types
//!
//! Aggregates errors from all sub-crates into a unified error type.
//!
//! ## Privacy Policy
//!
//! Error messages MUST NOT contain:
//! - transcript text
//! - audio data
//! - LLM prompts or responses
//! - encryption keys
//! - user data of any kind
//!
//! Use structural error categories (e.g., "edit_ratio_exceeded")
//! instead of embedding user content in error strings.

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

    #[error("runtime error: {0}")]
    Runtime(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("transcript verifier rejected candidate: {reason}")]
    VerifierRejection { reason: String },

    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("model integrity verification failed")]
    ModelIntegrityFailure,

    #[error("resource exhaustion: {resource}")]
    ResourceExhausted { resource: String },

    #[error("worker process failed: {0}")]
    WorkerFailed(String),

    #[error("backend unavailable: {0}")]
    BackendUnavailable(String),
}
