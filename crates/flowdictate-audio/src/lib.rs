//! # flowdictate-audio
//!
//! Audio capture and preprocessing subsystem for FlowDictate.
//!
//! This crate handles the real-time audio pipeline from microphone to
//! processed speech frames ready for ASR inference:
//!
//! ```text
//! Microphone (cpal) → Ring Buffer (SPSC) → Processor Thread
//!     → Channel Conversion → Resampling (16kHz) → Normalization (f32)
//!     → VAD → Speech Frames → [ASR Engine]
//! ```
//!
//! ## Safety Invariants
//!
//! - The audio capture callback performs **no heap allocation**
//! - The audio capture callback performs **no disk I/O**
//! - The audio capture callback performs **no blocking locks**
//! - The ring buffer is **bounded and preallocated**
//! - VAD failure cannot create **unbounded buffers**
//! - Audio data exists only in **volatile memory**

pub mod capture;
pub mod codec;
pub mod normalize;
pub mod resample;
pub mod ringbuffer;
pub mod vad;
