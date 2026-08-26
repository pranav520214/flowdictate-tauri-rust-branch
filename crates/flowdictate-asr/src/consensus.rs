//! # Hypothesis Consensus & Commit
//!
//! Implements a consensus strategy to separate stable committed text from
//! unstable in-progress hypotheses.
//!
//! Whisper produces hypotheses that can change as more audio arrives.
//! This module ensures that only text with sufficient stability is committed
//! to the output, preventing the jarring rewrite behavior seen in naive
//! streaming implementations.
//!
//! ## Strategy
//!
//! - Track hypothesis history over multiple inference passes
//! - Text that remains stable across N consecutive passes is committed
//! - Committed text is never rewritten
//! - Unstable text is shown as provisional in the UI overlay

// TODO(milestone-2): Implement consensus tracking
// TODO(milestone-2): Implement stability threshold configuration
// TODO(milestone-2): Add tests with simulated hypothesis sequences
