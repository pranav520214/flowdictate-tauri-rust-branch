//! # Codec Abstraction Layer
//!
//! Manages audio format conversion for non-live-pipeline use cases.
//!
//! ## Live Pipeline
//!
//! The live inference path uses ONLY normalized PCM — no codec encoding/decoding.
//! This module is NOT in the live audio path.
//!
//! ## Offline/Utility Use Cases
//!
//! - WAV/PCM: Test fixtures, debugging, diagnostic exports (via `hound`)
//! - FLAC: Optional lossless training data import (via `claxon`, feature-gated)
//! - Opus: Optional compressed recording (via `audiopus`, feature-gated)
//!
//! ## Security
//!
//! All external audio files are UNTRUSTED INPUT. Before decoding:
//! - Validate file size (max limit)
//! - Validate container type / magic bytes
//! - Validate channel count (reject > 16)
//! - Validate sample rate (reject > 384kHz)
//! - Validate duration (reject > configurable max)
//! - Bound decompression output size

// TODO(milestone-1): Implement WAV reading for test fixtures
// TODO(milestone-1): Implement codec validation gates
// TODO(milestone-9): Fuzz codec parsers
