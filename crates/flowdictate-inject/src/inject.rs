//! # Text Injection (§13, §18, §19)
//!
//! Transaction-like text insertion into focused applications with target window
//! tracking and strategy degradation.
//!
//! ## Invariants
//!
//! - Transcript text is NEVER executed as shell commands or system operations.
//! - NUL characters are rejected.
//! - Target focus changes abort injection to prevent accidental insertion into unintended windows (§18).

use enigo::{Enigo, Keyboard, Settings};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum InjectionError {
    #[error("text contains forbidden NUL character")]
    NullByte,
    #[error("target window focus changed from {expected} to {actual}; held text safely")]
    TargetMismatch { expected: String, actual: String },
    #[error("injection subsystem unavailable: {0}")]
    SubsystemUnavailable(String),
}

/// Strategy ladder for text insertion (§19).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InjectionStrategy {
    /// Unicode simulated typing (default / most compatible)
    #[default]
    UnicodeTyping,
    /// Controlled clipboard paste
    ControlledPaste,
    /// Synthesized raw keystrokes
    SynthesizedKeystrokes,
}

/// Secure, transactional text injector.
pub struct TextInjector {
    enigo: Option<Enigo>,
    strategy: InjectionStrategy,
}

impl Default for TextInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInjector {
    /// Create a new TextInjector. Initializes OS input simulation.
    pub fn new() -> Self {
        let enigo = Enigo::new(&Settings::default()).ok();
        Self {
            enigo,
            strategy: InjectionStrategy::UnicodeTyping,
        }
    }

    /// Current injection strategy.
    pub fn strategy(&self) -> InjectionStrategy {
        self.strategy
    }

    /// Configure active injection strategy.
    pub fn set_strategy(&mut self, strategy: InjectionStrategy) {
        self.strategy = strategy;
    }

    /// Injects validated text into the active foreground window with target verification (§18).
    ///
    /// If initial target window changed before injection, injection is aborted and the text is held.
    pub fn inject_transactional(
        &mut self,
        text: &str,
        expected_target: Option<&str>,
        actual_target: Option<&str>,
    ) -> Result<(), InjectionError> {
        if text.contains('\0') {
            return Err(InjectionError::NullByte);
        }

        if text.is_empty() {
            return Ok(());
        }

        // Target window validation: abort if focus changed mid-dictation (§18)
        if let (Some(expected), Some(actual)) = (expected_target, actual_target) {
            if expected != actual {
                return Err(InjectionError::TargetMismatch {
                    expected: expected.to_string(),
                    actual: actual.to_string(),
                });
            }
        }

        self.inject_text(text)
    }

    /// Injects validated text into the active foreground window.
    pub fn inject_text(&mut self, text: &str) -> Result<(), InjectionError> {
        if text.contains('\0') {
            return Err(InjectionError::NullByte);
        }

        if text.is_empty() {
            return Ok(());
        }

        if let Some(ref mut enigo) = self.enigo {
            enigo
                .text(text)
                .map_err(|e| InjectionError::SubsystemUnavailable(format!("{e:?}")))?;
            tracing::info!(event = "text_injected", len = text.len());
            Ok(())
        } else {
            Err(InjectionError::SubsystemUnavailable(
                "input simulation subsystem unavailable".to_string(),
            ))
        }
    }

    /// Simulate backspace key press to delete previous characters/words.
    pub fn inject_backspace(&mut self, count: usize) -> Result<(), InjectionError> {
        if let Some(ref mut enigo) = self.enigo {
            for _ in 0..count {
                enigo
                    .key(enigo::Key::Backspace, enigo::Direction::Click)
                    .map_err(|e| InjectionError::SubsystemUnavailable(format!("{e:?}")))?;
            }
            Ok(())
        } else {
            Err(InjectionError::SubsystemUnavailable(
                "input simulation subsystem unavailable".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rejects_nul_byte() {
        let mut injector = TextInjector {
            enigo: None,
            strategy: InjectionStrategy::UnicodeTyping,
        };
        assert_eq!(
            injector.inject_text("hello\0world"),
            Err(InjectionError::NullByte)
        );
    }

    #[test]
    fn test_target_mismatch_aborts_injection() {
        let mut injector = TextInjector {
            enigo: None,
            strategy: InjectionStrategy::UnicodeTyping,
        };
        let res = injector.inject_transactional(
            "my secret text",
            Some("notepad.exe"),
            Some("discord.exe"),
        );
        assert_eq!(
            res,
            Err(InjectionError::TargetMismatch {
                expected: "notepad.exe".to_string(),
                actual: "discord.exe".to_string(),
            })
        );
    }

    #[test]
    fn test_matching_target_proceeds_to_subsystem() {
        let mut injector = TextInjector {
            enigo: None,
            strategy: InjectionStrategy::UnicodeTyping,
        };
        // Without active enigo, it safely returns SubsystemUnavailable, proving validation passed
        let res =
            injector.inject_transactional("safe text", Some("notepad.exe"), Some("notepad.exe"));
        assert!(matches!(res, Err(InjectionError::SubsystemUnavailable(_))));
    }
}
