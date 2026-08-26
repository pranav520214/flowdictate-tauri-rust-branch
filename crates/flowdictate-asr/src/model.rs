//! # Model Loading with Integrity Verification
//!
//! Loads whisper.cpp models ONLY after they pass the integrity gate
//! in `flowdictate-security::model_integrity`.
//!
//! ## Loading Flow
//!
//! ```text
//! Model path → validate_path() → check_file_size() → compute_sha256()
//!     → compare_manifest() → [PASS] → whisper_rs::WhisperContext::new()
//!                          → [FAIL] → return error, DO NOT LOAD
//! ```

// TODO(milestone-2): Implement verified model loading
// TODO(milestone-2): Implement model info queries (languages, size, etc.)
