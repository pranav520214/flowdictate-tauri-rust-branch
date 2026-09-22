//! # Voice Activity Detection
//!
//! Lightweight VAD using WebRTC VAD to detect speech boundaries.
//!
//! ## State Machine
//!
//! ```text
//! Silence → SpeechStart → SpeechContinue → ShortPause → UtteranceComplete
//!                              ↓                              ↓
//!                         ShortPause ──→ SpeechContinue    LongSilence
//! ```
//!
//! ## Safety
//!
//! - Failed VAD state MUST NOT create unbounded audio buffers
//! - Configurable silence threshold for utterance completion
//! - Maximum utterance duration limit to prevent runaway buffering

// TODO(milestone-1): Implement VAD state machine
// TODO(milestone-1): Implement configurable silence/speech thresholds
// TODO(milestone-1): Add maximum duration safety limit
// TODO(milestone-1): Add tests for state transitions
