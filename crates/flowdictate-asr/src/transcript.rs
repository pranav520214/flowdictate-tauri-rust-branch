//! # ASR Transcript Events
//!
//! Defines the streaming transcript event protocol (§9).
//!
//! Only `Committed` and `Final` events may enter the refinement pipeline.
//! `Partial` events update the UI but are never authoritative.

use serde::{Deserialize, Serialize};

/// A streaming transcript event from the ASR engine.
///
/// ## Lifecycle
///
/// ```text
/// Partial → Partial → ... → Committed → ... → Final
/// ```
///
/// - `Partial`: Unstable hypothesis. May change. UI display only.
/// - `Committed`: Stable segment. Enters the refinement pipeline.
/// - `Final`: End of utterance. Enters the refinement pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscriptEvent {
    /// Unstable partial hypothesis. May be revised by subsequent events.
    /// Safe to display in the UI but MUST NOT be sent to the LLM
    /// or used as authoritative pipeline state.
    Partial(String),

    /// A segment that the ASR engine has committed as stable.
    /// This text will not be revised by subsequent partial updates.
    /// May enter the refinement pipeline.
    Committed(String),

    /// The final transcript for the completed utterance.
    /// The ASR session is complete after this event.
    /// Enters the refinement pipeline.
    Final(String),
}

impl TranscriptEvent {
    /// Returns the text content regardless of event type.
    pub fn text(&self) -> &str {
        match self {
            Self::Partial(t) | Self::Committed(t) | Self::Final(t) => t,
        }
    }

    /// Returns true if this event is authoritative (Committed or Final).
    /// Only authoritative events may enter the refinement pipeline.
    pub fn is_authoritative(&self) -> bool {
        matches!(self, Self::Committed(_) | Self::Final(_))
    }

    /// Returns true if this is the terminal event for a session.
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Final(_))
    }
}

/// Maximum length of a single transcript event text in bytes.
/// Rejects unbounded output from native runtimes (§17).
pub const MAX_TRANSCRIPT_TEXT_BYTES: usize = 32_768;

/// Validate a transcript event's text content.
///
/// Rejects:
/// - Text exceeding MAX_TRANSCRIPT_TEXT_BYTES
/// - Invalid UTF-8 (should not happen with String, but defense-in-depth)
/// - Embedded NUL characters
pub fn validate_transcript_text(text: &str) -> Result<(), TranscriptValidationError> {
    if text.len() > MAX_TRANSCRIPT_TEXT_BYTES {
        return Err(TranscriptValidationError::TooLong {
            len: text.len(),
            max: MAX_TRANSCRIPT_TEXT_BYTES,
        });
    }
    if text.contains('\0') {
        return Err(TranscriptValidationError::EmbeddedNul);
    }
    Ok(())
}

/// Errors from transcript text validation.
#[derive(Debug, Clone)]
pub enum TranscriptValidationError {
    TooLong { len: usize, max: usize },
    EmbeddedNul,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_is_not_authoritative() {
        let evt = TranscriptEvent::Partial("hello".to_string());
        assert!(!evt.is_authoritative());
        assert!(!evt.is_final());
    }

    #[test]
    fn committed_is_authoritative() {
        let evt = TranscriptEvent::Committed("hello world".to_string());
        assert!(evt.is_authoritative());
        assert!(!evt.is_final());
    }

    #[test]
    fn final_is_authoritative_and_terminal() {
        let evt = TranscriptEvent::Final("hello world.".to_string());
        assert!(evt.is_authoritative());
        assert!(evt.is_final());
    }

    #[test]
    fn rejects_too_long_text() {
        let long_text = "a".repeat(MAX_TRANSCRIPT_TEXT_BYTES + 1);
        assert!(matches!(
            validate_transcript_text(&long_text),
            Err(TranscriptValidationError::TooLong { .. })
        ));
    }

    #[test]
    fn rejects_embedded_nul() {
        assert!(matches!(
            validate_transcript_text("hello\0world"),
            Err(TranscriptValidationError::EmbeddedNul)
        ));
    }

    #[test]
    fn accepts_valid_text() {
        assert!(validate_transcript_text("Hello, world! 日本語 Ñ").is_ok());
    }
}
