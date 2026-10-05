//! # flowdictate-core
//!
//! Central pipeline orchestrator for FlowDictate.
//!
//! Coordinates the complete dictation flow:
//!
//! ```text
//! Hotkey Press → Audio Capture → VAD → Streaming ASR → Refinement → Text Injection
//! ```
//!
//! ## Thread Architecture
//!
//! - Audio callback thread (OS-managed, zero-alloc) → ring buffer
//! - Audio processor thread (resample, VAD) → crossbeam channel
//! - ASR inference thread (whisper.cpp) → crossbeam channel
//! - Refinement thread (deterministic + optional LLM) → crossbeam channel
//! - Main/UI thread (Tauri event loop)
//!
//! ## Performance Modes
//!
//! - **Eco**: whisper-tiny, deterministic-only, models unload after inactivity
//! - **Balanced**: whisper-base, tiny LLM warm when memory permits
//! - **Quality**: larger models if installed, still entirely offline

pub mod config;
pub mod error;
pub mod metrics;
pub mod pipeline;
pub mod session;
