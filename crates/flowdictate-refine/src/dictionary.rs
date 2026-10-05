//! # User Dictionary & Deterministic Replacement Rules (§11, §12)
//!
//! Manages personal vocabulary and deterministic text transformation rules.
//!
//! ## Invariants
//!
//! - Replacement rules are explicit, inspectable, and deterministic.
//! - Enforces an execution loop cap (max 100 transformations) to prevent recursive expansion loops.
//! - Personal vocabulary remains strictly local and is never logged.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Maximum number of replacement passes allowed to prevent recursive substitution loops.
const MAX_TRANSFORMATION_PASSES: usize = 5;

/// Local personal vocabulary and deterministic replacement rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDictionary {
    /// Specialized terms, names, and acronyms.
    pub terms: HashSet<String>,
    /// Explicit deterministic replacements: `(pattern_lowercase, replacement)`.
    pub replacements: Vec<(String, String)>,
}

impl Default for UserDictionary {
    fn default() -> Self {
        let mut dict = Self::new();
        // Built-in common tech/voice dictation defaults
        dict.add_replacement("flow dictate", "FlowDictate");
        dict.add_replacement("git hub", "GitHub");
        dict.add_replacement("k eight s", "K8s");
        dict.add_replacement("nemo speech", "NeMo-Speech");
        dict.add_term("Nemotron");
        dict.add_term("Qwen");
        dict
    }
}

impl UserDictionary {
    pub fn new() -> Self {
        Self {
            terms: HashSet::new(),
            replacements: Vec::new(),
        }
    }

    /// Add a specialized vocabulary term.
    pub fn add_term(&mut self, term: &str) {
        self.terms.insert(term.to_string());
    }

    /// Add a deterministic replacement rule.
    pub fn add_replacement(&mut self, pattern: &str, replacement: &str) {
        self.replacements
            .push((pattern.to_lowercase(), replacement.to_string()));
    }

    /// Check if a word is in the personal dictionary.
    pub fn is_known_term(&self, word: &str) -> bool {
        self.terms.contains(word)
    }

    /// Apply replacement rules deterministically with recursion protection.
    pub fn apply_replacements(&self, input: &str) -> String {
        if self.replacements.is_empty() || input.is_empty() {
            return input.to_string();
        }

        let mut current = input.to_string();

        for _pass in 0..MAX_TRANSFORMATION_PASSES {
            let mut changed = false;

            for (pattern, replacement) in &self.replacements {
                let lower = current.to_lowercase();
                if let Some(pos) = lower.find(pattern) {
                    let mut next = String::with_capacity(current.len() + replacement.len());
                    next.push_str(&current[..pos]);
                    next.push_str(replacement);
                    next.push_str(&current[pos + pattern.len()..]);
                    current = next;
                    changed = true;
                    break; // Restart search after substitution
                }
            }

            if !changed {
                break;
            }
        }

        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replacements_applied_case_insensitively() {
        let dict = UserDictionary::default();
        let input = "I am using flow dictate with git hub and k eight s.";
        let output = dict.apply_replacements(input);
        assert_eq!(output, "I am using FlowDictate with GitHub and K8s.");
    }

    #[test]
    fn test_recursive_replacement_loop_protection() {
        let mut dict = UserDictionary::new();
        // Malicious or accidental circular rule: "a" -> "a b"
        dict.add_replacement("foo", "foo bar");
        let res = dict.apply_replacements("hello foo");
        // Must terminate safely within MAX_TRANSFORMATION_PASSES and not infinite loop
        assert!(res.contains("foo bar"));
    }
}
