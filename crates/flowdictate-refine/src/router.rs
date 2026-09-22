//! # Refinement Pipeline Router
//!
//! Orchestrates the multi-stage refinement pipeline:
//!
//! ```text
//! ASR text → Command Classifier
//!           ├─ Voice Command → executed directly
//!           └─ Dictation Text
//!                ↓
//!           Stage A: Deterministic Cleanup
//!                ↓
//!           Router Decision (needs LLM?)
//!           ├─ No  → Output Sanitization → Final Text
//!           └─ Yes → Stage B: LLM Refinement
//!                      ↓
//!                    Semantic Verifier
//!                    ├─ Passed → Output Sanitization → Final Text
//!                    └─ Failed → Fallback to Stage A Text
//! ```

use crate::commands::{classify_utterance, ClassificationResult, VoiceCommand};
use crate::deterministic::{clean_text, DeterministicOptions};
use crate::dictionary::UserDictionary;
use crate::llm::Refiner;
use crate::sanitize::sanitize_output;
use crate::verifier::SemanticVerifier;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineRefineError {
    #[error("sanitization failed: {0}")]
    Sanitize(#[from] crate::sanitize::SanitizeError),
}

/// The final outcome of the refinement pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefinedOutcome {
    /// Utterance was classified as an explicit voice command.
    Command(VoiceCommand),
    /// Utterance was refined into sanitized text ready for injection.
    Text(String),
}

/// High-level text refinement pipeline coordinator.
#[derive(Default)]
pub struct RefinementPipeline {
    deterministic_opts: DeterministicOptions,
    dictionary: UserDictionary,
    verifier: SemanticVerifier,
    use_llm: bool,
}

impl RefinementPipeline {
    pub fn new(use_llm: bool) -> Self {
        Self {
            use_llm,
            ..Default::default()
        }
    }

    /// Process raw ASR text through the complete refinement pipeline.
    pub fn process<R: Refiner>(
        &self,
        raw_text: &str,
        refiner: &mut R,
    ) -> Result<RefinedOutcome, PipelineRefineError> {
        // 1. Voice Command Classification (§12)
        match classify_utterance(raw_text) {
            ClassificationResult::Command(cmd) => return Ok(RefinedOutcome::Command(cmd)),
            ClassificationResult::Dictation(_) => {}
        }

        // 2. Stage A: Deterministic Cleanup + Dictionary Replacements (§11, §12)
        let cleaned = clean_text(raw_text, &self.deterministic_opts);
        let deterministic_text = self.dictionary.apply_replacements(&cleaned);

        // 3. Stage B: LLM Refinement (if enabled & needed)
        let candidate_text =
            if self.use_llm && refiner.is_ready() && should_route_to_llm(&deterministic_text) {
                match refiner.refine(&deterministic_text) {
                    Ok(llm_output) => {
                        // 4. Semantic Verification (§11)
                        match self.verifier.verify(&deterministic_text, &llm_output) {
                            Ok(verified) => verified.to_string(),
                            Err(rejection) => {
                                tracing::warn!(
                                    event = "verifier_rejected_llm_output",
                                    reason = ?rejection
                                );
                                // Fail-safe fallback to deterministic text
                                deterministic_text
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!(
                            event = "llm_refinement_failed",
                            error = %err
                        );
                        // Fail-safe fallback to deterministic text
                        deterministic_text
                    }
                }
            } else {
                deterministic_text
            };

        // 5. Output Sanitization
        let final_text = sanitize_output(&candidate_text)?;

        Ok(RefinedOutcome::Text(final_text))
    }
}

/// Heuristic to decide if text benefits from LLM refinement.
fn should_route_to_llm(text: &str) -> bool {
    let lower = text.to_lowercase();
    // Check for self-correction patterns
    lower.contains("i mean") || lower.contains("actually no") || lower.contains("or rather")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::NoopRefiner;

    #[test]
    fn routes_voice_commands() {
        let pipeline = RefinementPipeline::new(false);
        let mut refiner = NoopRefiner;
        let res = pipeline.process("new paragraph", &mut refiner).unwrap();
        assert_eq!(res, RefinedOutcome::Command(VoiceCommand::NewParagraph));
    }

    #[test]
    fn routes_dictation_through_deterministic_pipeline() {
        let pipeline = RefinementPipeline::new(false);
        let mut refiner = NoopRefiner;
        let res = pipeline
            .process("hello um world period", &mut refiner)
            .unwrap();
        assert_eq!(res, RefinedOutcome::Text("Hello world.".to_string()));
    }

    #[test]
    fn routes_dictation_with_user_dictionary() {
        let pipeline = RefinementPipeline::new(false);
        let mut refiner = NoopRefiner;
        let res = pipeline
            .process("i love flow dictate on git hub period", &mut refiner)
            .unwrap();
        assert_eq!(
            res,
            RefinedOutcome::Text("I love FlowDictate on GitHub.".to_string())
        );
    }
}
