//! # Confidence/Complexity Router
//!
//! Decides whether a transcript needs LLM refinement or can be handled
//! by deterministic cleanup alone.
//!
//! ## Decision Criteria
//!
//! Route to deterministic-only (Stage A) when:
//! - Text is simple declarative sentences
//! - Only whitespace/punctuation/capitalization fixes needed
//! - Filler words are clearly identifiable
//! - User dictionary covers all specialized terms
//!
//! Route to LLM refinement (Stage B) when:
//! - Ambiguous self-corrections detected ("actually no", "I mean")
//! - Complex sentence restructuring needed
//! - Contextual formatting required
//! - Confidence in deterministic output is below threshold

// TODO(milestone-3): Implement routing heuristics
// TODO(milestone-3): Implement confidence scoring
// TODO(milestone-10): Tune thresholds based on benchmark results
