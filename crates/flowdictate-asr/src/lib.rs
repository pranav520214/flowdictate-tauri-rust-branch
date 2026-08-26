//! # flowdictate-asr
//!
//! Streaming automatic speech recognition using whisper.cpp.
//!
//! This crate wraps `whisper-rs` to provide:
//!
//! - **Streaming transcription** — live partial hypotheses as the user speaks
//! - **Consensus commit** — stable text separated from unstable hypotheses
//! - **Model integrity** — delegates to `flowdictate-security` for SHA-256 verification
//! - **Bounded decoding** — rolling window prevents unbounded memory growth
//!
//! ## Pipeline Position
//!
//! ```text
//! [flowdictate-audio: VAD speech frames]
//!     → Rolling ASR Window
//!     → Partial Hypotheses (sent to UI overlay)
//!     → Consensus Commit (stable text)
//!     → [flowdictate-refine: text cleanup]
//! ```
//!
//! ## Model Loading
//!
//! Models are NEVER loaded without passing the integrity gate:
//! `flowdictate-security::model_integrity::verify_model_file()`

pub mod consensus;
pub mod engine;
pub mod model;
pub mod streaming;
