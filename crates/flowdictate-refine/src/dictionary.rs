//! # User Dictionary
//!
//! Manages user-defined vocabulary for improved transcription accuracy.
//!
//! ## Supported Entries
//!
//! - Names (personal, organizational, product)
//! - Acronyms and abbreviations
//! - Programming identifiers
//! - Scientific/technical terms
//! - Preferred spellings
//! - Pronunciation hints
//!
//! ## Privacy
//!
//! Dictionary data is stored encrypted in the local SQLCipher database.
//! Dictionary contents MUST NEVER appear in logs.
//!
//! ## Pipeline Position
//!
//! Dictionary matching happens BEFORE LLM refinement (Stage A),
//! reducing the need for expensive model inference.

// TODO(milestone-3): Implement dictionary data structures
// TODO(milestone-3): Implement matching and replacement
// TODO(milestone-7): Implement import/export
// TODO(milestone-7): Implement pronunciation hints
