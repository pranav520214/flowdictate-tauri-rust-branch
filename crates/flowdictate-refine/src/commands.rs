//! # Voice Command Classifier (§12)
//!
//! Separates explicit voice commands from regular dictation BEFORE LLM refinement.
//!
//! ## Invariant
//!
//! The LLM is NEVER given authority to decide whether an utterance is a command.
//! Command matching uses a strict deterministic grammar. Destructive commands
//! (e.g. undo, delete) are isolated and require exact matching.

use serde::{Deserialize, Serialize};

/// Recognized voice commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceCommand {
    /// Insert a newline (`\n`)
    NewLine,
    /// Insert two newlines (`\n\n`)
    NewParagraph,
    /// Undo recent input
    Undo,
    /// Delete previous word
    DeletePreviousWord,
    /// Clear current line or buffer
    Clear,
    /// Select all text
    SelectAll,
    /// Cancel/abort current dictation without injecting
    Cancel,
}

/// The result of classifying an utterance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassificationResult {
    /// Pure voice command executed by the system.
    Command(VoiceCommand),
    /// Regular speech for dictation.
    Dictation(String),
}

/// Classifies an incoming text segment into either a VoiceCommand or Dictation.
pub fn classify_utterance(text: &str) -> ClassificationResult {
    let normalized = text.trim().to_lowercase();
    let cleaned = normalized.trim_end_matches(['.', '!', '?']).trim();

    match cleaned {
        "new line" | "newline" | "next line" => {
            ClassificationResult::Command(VoiceCommand::NewLine)
        }
        "new paragraph" | "new para" => ClassificationResult::Command(VoiceCommand::NewParagraph),
        "undo that" | "undo" => ClassificationResult::Command(VoiceCommand::Undo),
        "delete previous word" | "delete last word" | "delete word" => {
            ClassificationResult::Command(VoiceCommand::DeletePreviousWord)
        }
        "clear line" | "clear" | "delete all" => ClassificationResult::Command(VoiceCommand::Clear),
        "select all" => ClassificationResult::Command(VoiceCommand::SelectAll),
        "cancel" | "cancel dictation" | "stop dictation" => {
            ClassificationResult::Command(VoiceCommand::Cancel)
        }
        _ => ClassificationResult::Dictation(text.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_commands_correctly() {
        assert_eq!(
            classify_utterance("New line."),
            ClassificationResult::Command(VoiceCommand::NewLine)
        );
        assert_eq!(
            classify_utterance("undo that!"),
            ClassificationResult::Command(VoiceCommand::Undo)
        );
        assert_eq!(
            classify_utterance("new paragraph"),
            ClassificationResult::Command(VoiceCommand::NewParagraph)
        );
        assert_eq!(
            classify_utterance("cancel dictation"),
            ClassificationResult::Command(VoiceCommand::Cancel)
        );
    }

    #[test]
    fn classifies_regular_dictation() {
        assert_eq!(
            classify_utterance("I want to create a new line of code"),
            ClassificationResult::Dictation("I want to create a new line of code".to_string())
        );
        assert_eq!(
            classify_utterance("Undo is my favorite shortcut"),
            ClassificationResult::Dictation("Undo is my favorite shortcut".to_string())
        );
    }
}
