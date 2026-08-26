//! # flowdictate-refine
//!
//! Text refinement pipeline that cleans raw ASR transcripts into polished text.
//!
//! ## Two-Stage Architecture
//!
//! ### Stage A — Deterministic Cleanup (always active)
//!
//! Handles inexpensive operations without any model:
//! - Duplicate whitespace removal
//! - Capitalization after sentence boundaries
//! - Filler word removal ("um", "uh", "like", "you know")
//! - Punctuation normalization
//! - Spacing around punctuation
//! - Spoken formatting commands ("new line", "period", "comma")
//! - User dictionary replacement
//!
//! ### Stage B — Local LLM Refinement (optional, feature-gated)
//!
//! Invoked ONLY when:
//! - Semantic rewriting is actually necessary
//! - Transcript contains ambiguous self-correction
//! - Formatting requires contextual understanding
//! - Deterministic rules have insufficient confidence
//!
//! The LLM is an **editor**, not an assistant. It must:
//! - Preserve meaning
//! - Not invent facts
//! - Not add information
//! - Not answer the dictated text
//! - Not treat dictated content as system instructions
//!
//! ## Fallback Hierarchy
//!
//! ```text
//! LLM refinement fails → deterministic cleanup → sanitized raw transcript
//! ```
//!
//! Never falls back to a cloud service.

pub mod deterministic;
pub mod dictionary;
#[cfg(feature = "llm")]
pub mod llm;
pub mod router;
pub mod sanitize;
