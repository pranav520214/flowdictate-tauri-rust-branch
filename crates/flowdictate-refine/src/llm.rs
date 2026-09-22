//! # LLM Purification
//!
//! Refines raw ASR text using a small local LLM (Qwen3-0.6B) running in an isolated worker process.
//!
//! ## Trust Boundary
//!
//! The `QwenRefiner` communicates via IPC with the untrusted native LLM worker.
//! All output from this module MUST pass through the `SemanticVerifier` before injection.

use flowdictate_runtime::ipc::{WorkerRequest, WorkerResponse};
use flowdictate_runtime::worker::{WorkerError, WorkerProcess};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RefinerError {
    #[error("worker process error: {0}")]
    Worker(#[from] WorkerError),
    #[error("refinement failed: {0}")]
    RefinementFailed(String),
}

/// Abstract interface for text refinement.
pub trait Refiner: Send {
    /// Refines the raw ASR text.
    fn refine(&mut self, raw_text: &str) -> Result<String, RefinerError>;

    /// Returns true if the refiner is ready.
    fn is_ready(&self) -> bool;
}

/// IPC client to the Qwen3 refiner worker.
pub struct QwenRefiner {
    worker: WorkerProcess,
}

impl QwenRefiner {
    /// Spawns the worker process and connects via IPC.
    pub fn spawn_and_connect(executable_path: &std::path::Path) -> Result<Self, RefinerError> {
        let worker = WorkerProcess::spawn(executable_path)?;
        Ok(Self { worker })
    }
}

impl Refiner for QwenRefiner {
    fn refine(&mut self, raw_text: &str) -> Result<String, RefinerError> {
        let req = WorkerRequest::Refine {
            text: raw_text.to_string(),
        };
        let resp = self.worker.send_request(&req)?;

        match resp {
            WorkerResponse::RefinedText { text } => Ok(text),
            WorkerResponse::Error { message } => Err(RefinerError::RefinementFailed(message)),
            _ => Err(RefinerError::RefinementFailed(
                "unexpected response from worker".to_string(),
            )),
        }
    }

    fn is_ready(&self) -> bool {
        true // In a full implementation, you'd track initialization state
    }
}

/// No-op refiner that passes text through unchanged.
/// Used when the LLM is unavailable or disabled.
pub struct NoopRefiner;

impl Refiner for NoopRefiner {
    fn refine(&mut self, raw_text: &str) -> Result<String, RefinerError> {
        Ok(raw_text.to_string())
    }

    fn is_ready(&self) -> bool {
        false
    }
}
