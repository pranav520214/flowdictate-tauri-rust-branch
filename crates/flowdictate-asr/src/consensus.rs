//! # Hypothesis Consensus & Partial Stabilization (§6)
//!
//! Implements a partial stabilization layer maintaining:
//! - `committed_prefix`: stable text confirmed across decoder passes
//! - `unstable_suffix`: changing provisional hypothesis
//!
//! Prevents visual flickering and jarring rewrites in the UI overlay.

use std::collections::VecDeque;

/// A stabilized view of the current streaming transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StabilizedView {
    /// Stable prefix confirmed across passes.
    pub committed_prefix: String,
    /// Provisional unstable suffix currently being decoded.
    pub unstable_suffix: String,
    /// Combined full display text.
    pub display_text: String,
}

/// Partial transcript stabilizer tracking consensus across hypotheses.
#[derive(Debug, Clone)]
pub struct TranscriptStabilizer {
    committed_tokens: Vec<String>,
    history: VecDeque<Vec<String>>,
    stability_threshold: usize,
}

impl Default for TranscriptStabilizer {
    fn default() -> Self {
        Self::new(2) // 2 consecutive confirmations required for commit
    }
}

impl TranscriptStabilizer {
    /// Create a new stabilizer with a given stability threshold count.
    pub fn new(stability_threshold: usize) -> Self {
        Self {
            committed_tokens: Vec::new(),
            history: VecDeque::with_capacity(stability_threshold + 1),
            stability_threshold: stability_threshold.max(1),
        }
    }

    /// Reset the stabilizer state for a new dictation session.
    pub fn reset(&mut self) {
        self.committed_tokens.clear();
        self.history.clear();
    }

    /// Process a new raw hypothesis from the ASR engine.
    pub fn update(&mut self, hypothesis: &str) -> StabilizedView {
        let tokens: Vec<String> = hypothesis.split_whitespace().map(String::from).collect();

        if tokens.is_empty() {
            return self.current_view();
        }

        // If hypothesis begins after already committed tokens, track the new tokens
        let mut new_tokens = Vec::new();
        if tokens.len() > self.committed_tokens.len() {
            new_tokens = tokens[self.committed_tokens.len()..].to_vec();
        }

        if self.history.len() >= self.stability_threshold {
            self.history.pop_front();
        }
        self.history.push_back(new_tokens);

        // Find longest common prefix across all history frames
        if self.history.len() >= self.stability_threshold {
            let common_prefix = find_common_prefix(&self.history);
            if !common_prefix.is_empty() {
                self.committed_tokens.extend(common_prefix.clone());

                // Remove the newly committed prefix from all history frames
                for frame in self.history.iter_mut() {
                    if frame.len() >= common_prefix.len() {
                        frame.drain(0..common_prefix.len());
                    }
                }
            }
        }

        self.current_view_with_hypothesis(&tokens)
    }

    /// Force commit all remaining tokens at end-of-speech.
    pub fn finalize(&mut self, final_hypothesis: &str) -> String {
        let tokens: Vec<String> = final_hypothesis
            .split_whitespace()
            .map(String::from)
            .collect();

        self.committed_tokens = tokens;
        self.history.clear();
        self.committed_tokens.join(" ")
    }

    fn current_view(&self) -> StabilizedView {
        let prefix = self.committed_tokens.join(" ");
        StabilizedView {
            committed_prefix: prefix.clone(),
            unstable_suffix: String::new(),
            display_text: prefix,
        }
    }

    fn current_view_with_hypothesis(&self, full_hypothesis: &[String]) -> StabilizedView {
        let prefix = self.committed_tokens.join(" ");
        let suffix = if full_hypothesis.len() > self.committed_tokens.len() {
            full_hypothesis[self.committed_tokens.len()..].join(" ")
        } else {
            String::new()
        };

        let display_text = if prefix.is_empty() {
            suffix.clone()
        } else if suffix.is_empty() {
            prefix.clone()
        } else {
            format!("{prefix} {suffix}")
        };

        StabilizedView {
            committed_prefix: prefix,
            unstable_suffix: suffix,
            display_text,
        }
    }
}

/// Helper finding matching prefix across multiple hypothesis snapshots.
fn find_common_prefix(frames: &VecDeque<Vec<String>>) -> Vec<String> {
    if frames.is_empty() {
        return Vec::new();
    }

    let first = &frames[0];
    let mut common = Vec::new();

    for (i, token) in first.iter().enumerate() {
        let mut all_match = true;
        for frame in frames.iter().skip(1) {
            if i >= frame.len() || &frame[i] != token {
                all_match = false;
                break;
            }
        }
        if all_match {
            common.push(token.clone());
        } else {
            break;
        }
    }

    common
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stabilizer_promotes_stable_prefix() {
        let mut stabilizer = TranscriptStabilizer::new(2);

        // Update 1
        let v1 = stabilizer.update("I need to send");
        assert_eq!(v1.committed_prefix, "");
        assert_eq!(v1.unstable_suffix, "I need to send");

        // Update 2: "I need to send" confirmed second time
        let v2 = stabilizer.update("I need to send the report");
        assert_eq!(v2.committed_prefix, "I need to send");
        assert_eq!(v2.unstable_suffix, "the report");

        // Update 3: Suffix changes to "a report", but "I need to send" stays committed!
        let v3 = stabilizer.update("I need to send a report");
        assert_eq!(v3.committed_prefix, "I need to send");
        assert_eq!(v3.unstable_suffix, "a report");
    }

    #[test]
    fn test_stabilizer_finalize() {
        let mut stabilizer = TranscriptStabilizer::new(2);
        stabilizer.update("hello world");
        let final_text = stabilizer.finalize("hello world finally");
        assert_eq!(final_text, "hello world finally");
    }
}
