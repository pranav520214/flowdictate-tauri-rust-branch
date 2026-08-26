//! # PCM Normalization
//!
//! Converts audio samples from various hardware formats to the canonical
//! internal representation used by the inference pipeline.
//!
//! ## Canonical Format
//!
//! - Mono (single channel)
//! - 16,000 Hz sample rate (after resampling)
//! - f32 samples in [-1.0, 1.0] range
//!
//! ## Input Formats Handled
//!
//! - i16 PCM → f32 (divide by 32768.0)
//! - i32 PCM → f32 (divide by 2147483648.0)
//! - f32 PCM → f32 (passthrough with clamping)
//! - Stereo → mono (average channels)
//! - Multi-channel → mono (average all channels)

// TODO(milestone-1): Implement sample format conversion
// TODO(milestone-1): Implement channel downmixing
// TODO(milestone-1): Add tests with known input/output pairs
