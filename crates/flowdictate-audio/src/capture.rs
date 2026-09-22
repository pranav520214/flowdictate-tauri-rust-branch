//! # Microphone Capture
//!
//! Cross-platform microphone enumeration and audio capture using `cpal`.
//!
//! ## Callback Contract
//!
//! The audio capture callback MUST:
//! - Copy samples into the preallocated ring buffer
//! - Return immediately
//!
//! The audio capture callback MUST NOT:
//! - Perform heap allocation
//! - Perform disk I/O
//! - Perform database operations
//! - Perform network operations
//! - Perform model inference
//! - Acquire blocking locks
//!
//! Its job is approximately: capture → normalize/copy → bounded buffer → return

// TODO(milestone-1): Implement microphone enumeration
// TODO(milestone-1): Implement capture stream with allocation-free callback
// TODO(milestone-1): Implement device selection and permission handling
