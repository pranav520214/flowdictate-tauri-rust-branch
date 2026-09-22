//! # Semantic Verifier
//!
//! Deterministic safety net protecting the output of the LLM refiner (§11).
//!
//! Evaluates the LLM's proposed text against the deterministic original text.
//! If the LLM output violates any structural or safety constraints, it is rejected,
//! and the deterministic original is used instead.

use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum VerifierRejection {
    #[error("edit distance ratio exceeded: {actual:.2} > {max:.2}")]
    EditRatioExceeded { actual: f64, max: f64 },

    #[error("output is too short relative to original")]
    TooShort,

    #[error("output is too long relative to original")]
    TooLong,

    #[error("protected tokens were modified or deleted")]
    ProtectedTokensModified,

    #[error("hallucination detected: new semantic clauses introduced")]
    Hallucination,
}

/// A deterministic semantic verifier for LLM output.
pub struct SemanticVerifier {
    max_edit_distance_ratio: f64,
    min_length_ratio: f64,
    max_length_ratio: f64,
}

impl Default for SemanticVerifier {
    fn default() -> Self {
        Self {
            max_edit_distance_ratio: 0.5, // Max 50% of tokens changed
            min_length_ratio: 0.5,        // LLM cannot delete more than half
            max_length_ratio: 1.5,        // LLM cannot expand by more than 50%
        }
    }
}

impl SemanticVerifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Verify the proposed text against the original.
    ///
    /// Returns `Ok(proposed)` if it passes all checks, otherwise `Err(VerifierRejection)`.
    pub fn verify<'a>(
        &self,
        original: &'a str,
        proposed: &'a str,
    ) -> Result<&'a str, VerifierRejection> {
        let orig_len = original.len();
        let prop_len = proposed.len();

        // Trivial accept for empty strings
        if orig_len == 0 && prop_len == 0 {
            return Ok(proposed);
        }

        // Cannot refine empty string to non-empty (hallucination)
        if orig_len == 0 && prop_len > 0 {
            return Err(VerifierRejection::Hallucination);
        }

        // 1. Length bound checks
        let len_ratio = prop_len as f64 / orig_len as f64;
        if len_ratio < self.min_length_ratio {
            return Err(VerifierRejection::TooShort);
        }
        if len_ratio > self.max_length_ratio {
            return Err(VerifierRejection::TooLong);
        }

        // 2. Protected token checks (numbers, emails)
        if !self.verify_protected_tokens(original, proposed) {
            return Err(VerifierRejection::ProtectedTokensModified);
        }

        // 3. Token-level change ratio check
        let orig_words: Vec<&str> = original.split_whitespace().collect();
        let prop_words: Vec<&str> = proposed.split_whitespace().collect();
        if !orig_words.is_empty() && !prop_words.is_empty() {
            let common = orig_words.iter().filter(|w| prop_words.contains(w)).count();
            let changed_ratio =
                1.0 - (common as f64 / orig_words.len().max(prop_words.len()) as f64);
            if changed_ratio > self.max_edit_distance_ratio {
                return Err(VerifierRejection::EditRatioExceeded {
                    actual: changed_ratio,
                    max: self.max_edit_distance_ratio,
                });
            }
        }

        Ok(proposed)
    }

    /// Ensures that "protected" tokens from the original are preserved in the proposed text (§26).
    fn verify_protected_tokens(&self, original: &str, proposed: &str) -> bool {
        let orig_protected = extract_protected_tokens(original);
        let prop_protected = extract_protected_tokens(proposed);

        // All critical tokens that were in the original MUST be in the proposed.
        for token in orig_protected {
            if !prop_protected.contains(&token) {
                return false;
            }
        }
        true
    }
}

/// Deterministically extract critical protected tokens that must never be altered or dropped (§26).
fn extract_protected_tokens(text: &str) -> HashSet<String> {
    let mut tokens = HashSet::new();

    for raw_token in text.split_whitespace() {
        let token = raw_token.trim_matches(|c: char| {
            c == '.' || c == ',' || c == '!' || c == '?' || c == ';' || c == ':'
        });
        if token.is_empty() {
            continue;
        }

        // 1. Any token containing digits (integers, decimals, percentages, dates, times, versions, IPs)
        if token.chars().any(|c| c.is_ascii_digit()) {
            tokens.insert(token.to_string());
            continue;
        }

        // 2. Email addresses
        if token.contains('@') && token.contains('.') {
            tokens.insert(token.to_string());
            continue;
        }

        // 3. URLs
        if token.starts_with("http://")
            || token.starts_with("https://")
            || token.starts_with("www.")
        {
            tokens.insert(token.to_string());
            continue;
        }

        // 4. Filesystem paths
        if token.contains('/') || token.contains('\\') {
            tokens.insert(token.to_string());
            continue;
        }

        // 5. Command flags (e.g. --workspace, -v)
        if token.starts_with('-') && token.len() > 1 {
            tokens.insert(token.to_string());
            continue;
        }

        // 6. Currency symbols ($, €, £, ¥, ₹)
        if token.starts_with('$')
            || token.starts_with('€')
            || token.starts_with('£')
            || token.starts_with('¥')
            || token.starts_with('₹')
        {
            tokens.insert(token.to_string());
            continue;
        }

        // 7. Code identifiers with underscore
        if token.contains('_') && token.len() > 1 {
            tokens.insert(token.to_string());
            continue;
        }

        // 8. Quoted tokens ("value", 'value')
        if (token.starts_with('"') && token.ends_with('"'))
            || (token.starts_with('\'') && token.ends_with('\''))
        {
            tokens.insert(token.to_string());
            continue;
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_refinement() {
        let v = SemanticVerifier::new();
        let result = v.verify("um so I think we should go", "So, I think we should go.");
        assert_eq!(result, Ok("So, I think we should go."));
    }

    #[test]
    fn rejects_hallucination_from_empty() {
        let v = SemanticVerifier::new();
        let result = v.verify("", "Hello there!");
        assert_eq!(result, Err(VerifierRejection::Hallucination));
    }

    #[test]
    fn rejects_massive_expansion() {
        let v = SemanticVerifier::new();
        let orig = "short";
        let prop = "this is a very very long hallucinated string that just keeps going";
        let result = v.verify(orig, prop);
        assert_eq!(result, Err(VerifierRejection::TooLong));
    }

    #[test]
    fn rejects_missing_numbers() {
        let v = SemanticVerifier::new();
        let orig = "call me at 555-1234 please";
        let prop = "call me at tomorrow please";
        let result = v.verify(orig, prop);
        assert_eq!(result, Err(VerifierRejection::ProtectedTokensModified));
    }

    #[test]
    fn rejects_version_and_ip_tampering() {
        let v = SemanticVerifier::new();
        let orig = "Deploy version 3.12.4 to 10.0.0.8.";
        let prop = "Deploy version 3.12.5 to 10.0.0.8.";
        let result = v.verify(orig, prop);
        assert_eq!(result, Err(VerifierRejection::ProtectedTokensModified));
    }

    #[test]
    fn rejects_url_and_email_tampering() {
        let v = SemanticVerifier::new();
        let orig = "Send email to user@company.com with https://github.com/flowdictate";
        let prop = "Send email to user@attacker.com with https://github.com/flowdictate";
        let result = v.verify(orig, prop);
        assert_eq!(result, Err(VerifierRejection::ProtectedTokensModified));
    }
}
