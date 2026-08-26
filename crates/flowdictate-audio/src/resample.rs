//! # Audio Resampling
//!
//! Converts audio from the microphone's native sample rate to the 16 kHz
//! required by whisper.cpp, using the `rubato` sinc resampler.
//!
//! ## Pipeline Position
//!
//! ```text
//! Ring Buffer → Channel Conversion → **Resampling** → Normalization → VAD
//! ```
//!
//! ## Requirements
//!
//! - Input: native sample rate (commonly 44.1 kHz or 48 kHz)
//! - Output: 16,000 Hz mono
//! - Must occur outside the real-time capture callback
//! - Must be benchmarked for: latency, CPU usage, allocations, signal quality

// TODO(milestone-1): Implement rubato sinc resampler wrapper
// TODO(milestone-1): Support 44.1kHz and 48kHz input rates
// TODO(milestone-1): Benchmark resampling latency and CPU usage
