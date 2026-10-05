//! # flowdictate-asr
//!
//! Abstract speech recognition interface for FlowDictate.
//!
//! ## Architecture (§5)
//!
//! This crate defines:
//! - The `Recognizer` trait — abstract ASR interface
//! - `TranscriptEvent` — streaming event protocol (Partial/Committed/Final)
//! - `NoopRecognizer` — degradation fallback
//!
//! The actual NeMo-Speech.cpp integration lives in the ASR worker process,
//! not in this crate. This separation maintains the trust boundary between
//! the Rust core and the native C++ inference runtime.
//!
//! ## Pipeline Position
//!
//! ```text
//! [flowdictate-audio: VAD speech frames]
//!     → Recognizer::feed_audio()
//!     → TranscriptEvent::Partial (UI overlay)
//!     → TranscriptEvent::Committed/Final (refinement pipeline)
//!     → [flowdictate-refine: text cleanup]
//! ```
//!
//! ## Model Loading
//!
//! Models are NEVER loaded without passing the integrity gate:
//! `flowdictate-security::model_integrity::verify_model_file()`

pub mod engine;
pub mod transcript;

// These modules retain their existing stub documentation for future work.
pub mod consensus;
pub mod model;
pub mod streaming;
