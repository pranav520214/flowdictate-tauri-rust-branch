//! # Bounded Ring Buffer
//!
//! Preallocated SPSC (Single-Producer Single-Consumer) ring buffer for
//! passing audio samples from the real-time capture callback to the
//! processor thread without allocation.
//!
//! ## Design
//!
//! - Producer: OS audio callback thread (writes captured samples)
//! - Consumer: Processor thread (reads samples for resampling/VAD)
//! - Overflow policy: Drop oldest frames (NEVER block the callback)
//! - Size: Preallocated for ~5 seconds at native sample rate
//!
//! ## Size Calculation Example
//!
//! ```text
//! 48,000 Hz × 1 channel × 4 bytes (f32) × 5 seconds = 960,000 bytes ≈ 938 KB
//! ```

// TODO(milestone-1): Implement preallocated ring buffer wrapper around ringbuf
// TODO(milestone-1): Implement overflow detection and metrics
// TODO(milestone-1): Add tests for bounded behavior under sustained input
