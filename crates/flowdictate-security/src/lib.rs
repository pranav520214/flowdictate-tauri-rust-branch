//! # flowdictate-security
//!
//! Security primitives for the FlowDictate application.
//!
//! This crate provides foundational security functionality that other crates depend on:
//!
//! - **Model integrity verification** — SHA-256 manifest-based model allowlisting
//! - **Memory security** — Zeroization wrappers for sensitive buffers
//! - **Path validation** — Traversal prevention and symlink checking
//! - **Audit trail** — Local-only audit logging for security events
//!
//! ## Security Boundaries
//!
//! This crate does NOT process audio, transcripts, or user content directly.
//! It provides validation gates that other crates must pass through before
//! loading models, accessing paths, or handling sensitive memory.

pub mod audit;
pub mod memory;
pub mod model_integrity;
pub mod path;
