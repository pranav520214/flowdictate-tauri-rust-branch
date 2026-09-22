//! # ASR Engine Interface
//!
//! Defines the abstract `Recognizer` trait that all ASR backends implement.
//!
//! ## Trust Boundary
//!
//! The `Recognizer` trait is the trust boundary between the core pipeline
//! and the native ASR runtime. Implementations may communicate with an
//! out-of-process worker (process isolation) or wrap an in-process FFI
//! binding (with documented compensating controls per §4).
//!
//! ## Architecture
//!
//! ```text
//! Primary:   NemotronRecognizer → NeMo-Speech.cpp (via IPC or FFI)
//! Fallback:  NoopRecognizer     → returns empty (for testing / degradation)
//! ```

use crate::transcript::TranscriptEvent;
use thiserror::Error;

/// Errors from ASR engine operations.
#[derive(Debug, Error)]
pub enum AsrError {
    #[error("model not loaded")]
    ModelNotLoaded,

    #[error("model verification failed")]
    ModelVerificationFailed,

    #[error("inference failed: {reason}")]
    InferenceFailed { reason: String },

    #[error("worker process crashed")]
    WorkerCrashed,

    #[error("worker communication timeout")]
    WorkerTimeout,

    #[error("transcript validation failed")]
    InvalidTranscript,

    #[error("backend unavailable: {backend}")]
    BackendUnavailable { backend: String },
}

/// Abstract speech recognition interface.
///
/// Implementations must:
/// - Validate all output from native runtimes before returning
/// - Handle native runtime crashes without panicking
/// - Bound all returned transcript text per §17
/// - Never access the network
pub trait Recognizer: Send {
    /// Feed a chunk of mono 16kHz f32 audio to the recognizer.
    ///
    /// Returns zero or more transcript events. Partial events may
    /// update the UI; only Committed/Final events are authoritative.
    ///
    /// The chunk size is determined by the audio pipeline (typically
    /// 30ms = 480 samples at 16kHz).
    fn feed_audio(&mut self, chunk: &[f32]) -> Result<Vec<TranscriptEvent>, AsrError>;

    /// Reset the recognizer state for a new utterance.
    fn reset(&mut self);

    /// Returns true if the recognizer has a model loaded and is ready.
    fn is_ready(&self) -> bool;
}

/// No-op recognizer for testing and graceful degradation.
///
/// Returns no transcript events. The application remains functional
/// (hotkey works, UI displays, injection path works) but produces
/// no speech recognition output.
pub struct NoopRecognizer;

impl Recognizer for NoopRecognizer {
    fn feed_audio(&mut self, _chunk: &[f32]) -> Result<Vec<TranscriptEvent>, AsrError> {
        Ok(Vec::new())
    }

    fn reset(&mut self) {}

    fn is_ready(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_recognizer_returns_empty() {
        let mut r = NoopRecognizer;
        let events = r.feed_audio(&[0.0; 480]).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn noop_recognizer_is_not_ready() {
        let r = NoopRecognizer;
        assert!(!r.is_ready());
    }

    #[test]
    fn noop_recognizer_reset_is_safe() {
        let mut r = NoopRecognizer;
        r.reset(); // Must not panic
    }
}
