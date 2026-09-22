//! # Session State Machine & No-Loss Recovery Buffer (§4, §3, §29)
//!
//! Enforces an explicit, testable state machine for each dictation session.
//! Tracks unique session IDs to reject stale worker events, and maintains
//! a volatile recovery buffer so captured speech is never discarded upon failure.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use thiserror::Error;

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique, monotonic session identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SessionId(pub u64);

impl SessionId {
    /// Generate a new globally unique SessionId.
    pub fn next() -> Self {
        SessionId(SESSION_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Status of text injection attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InjectionStatus {
    Pending,
    Succeeded,
    Failed(String),
}

/// Errors from invalid state machine transitions.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StateTransitionError {
    #[error("illegal state transition from {from:?} to {to:?}")]
    IllegalTransition { from: String, to: String },
    #[error("session ID mismatch: current {current:?}, transition target {target:?}")]
    SessionMismatch {
        current: SessionId,
        target: SessionId,
    },
}

/// Explicit session state machine states (§4).
#[derive(Debug, Clone, PartialEq)]
pub enum DictationState {
    Idle,
    Preparing {
        session_id: SessionId,
    },
    Listening {
        session_id: SessionId,
        started_at: Instant,
    },
    FinalizingAsr {
        session_id: SessionId,
    },
    Refining {
        session_id: SessionId,
        raw_transcript: String,
    },
    Verifying {
        session_id: SessionId,
        candidate_transcript: String,
    },
    Injecting {
        session_id: SessionId,
        final_transcript: String,
    },
    Completed {
        session_id: SessionId,
        final_transcript: String,
    },
    RecoverableFailure {
        session_id: SessionId,
        preserved_transcript: Option<String>,
        error: String,
    },
}

impl DictationState {
    /// Get the associated session ID if not Idle.
    pub fn session_id(&self) -> Option<SessionId> {
        match self {
            Self::Idle => None,
            Self::Preparing { session_id }
            | Self::Listening { session_id, .. }
            | Self::FinalizingAsr { session_id }
            | Self::Refining { session_id, .. }
            | Self::Verifying { session_id, .. }
            | Self::Injecting { session_id, .. }
            | Self::Completed { session_id, .. }
            | Self::RecoverableFailure { session_id, .. } => Some(*session_id),
        }
    }

    /// Check if a transition from `self` to `next` is valid per state rules.
    pub fn can_transition_to(&self, next: &DictationState) -> bool {
        // Any state with an active session can fail-safe transition to RecoverableFailure
        if let DictationState::RecoverableFailure { session_id, .. } = next {
            return match self.session_id() {
                Some(current_id) => current_id == *session_id,
                None => false,
            };
        }

        // Return to Idle is permitted from Completed or RecoverableFailure
        if matches!(next, DictationState::Idle) {
            return matches!(
                self,
                DictationState::Completed { .. }
                    | DictationState::RecoverableFailure { .. }
                    | DictationState::Idle
            );
        }

        match (self, next) {
            (DictationState::Idle, DictationState::Preparing { .. }) => true,
            (
                DictationState::Preparing { session_id: s1 },
                DictationState::Listening { session_id: s2, .. },
            ) => s1 == s2,
            (
                DictationState::Listening { session_id: s1, .. },
                DictationState::FinalizingAsr { session_id: s2 },
            ) => s1 == s2,
            (
                DictationState::FinalizingAsr { session_id: s1 },
                DictationState::Refining { session_id: s2, .. },
            ) => s1 == s2,
            // Fast path: bypass LLM refinement directly to Injecting
            (
                DictationState::FinalizingAsr { session_id: s1 },
                DictationState::Injecting { session_id: s2, .. },
            ) => s1 == s2,
            (
                DictationState::Refining { session_id: s1, .. },
                DictationState::Verifying { session_id: s2, .. },
            ) => s1 == s2,
            (
                DictationState::Refining { session_id: s1, .. },
                DictationState::Injecting { session_id: s2, .. },
            ) => s1 == s2,
            (
                DictationState::Verifying { session_id: s1, .. },
                DictationState::Injecting { session_id: s2, .. },
            ) => s1 == s2,
            (
                DictationState::Injecting { session_id: s1, .. },
                DictationState::Completed { session_id: s2, .. },
            ) => s1 == s2,
            _ => false,
        }
    }

    /// Execute a validated transition, returning the new state.
    pub fn transition_to(&mut self, next: DictationState) -> Result<(), StateTransitionError> {
        if !self.can_transition_to(&next) {
            return Err(StateTransitionError::IllegalTransition {
                from: format!("{self:?}"),
                to: format!("{next:?}"),
            });
        }
        *self = next;
        Ok(())
    }
}

/// Volatile recovery buffer preserving user speech across every stage (§29).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryBuffer {
    pub session_id: SessionId,
    pub raw_asr_final: Option<String>,
    pub stage_a_final: Option<String>,
    pub llm_candidate: Option<String>,
    pub accepted_final: Option<String>,
    pub injection_status: InjectionStatus,
}

impl RecoveryBuffer {
    pub fn new(session_id: SessionId) -> Self {
        Self {
            session_id,
            raw_asr_final: None,
            stage_a_final: None,
            llm_candidate: None,
            accepted_final: None,
            injection_status: InjectionStatus::Pending,
        }
    }

    pub fn preserve_asr_final(&mut self, text: String) {
        self.raw_asr_final = Some(text);
    }

    pub fn preserve_stage_a(&mut self, text: String) {
        self.stage_a_final = Some(text);
    }

    pub fn preserve_llm_candidate(&mut self, text: String) {
        self.llm_candidate = Some(text);
    }

    pub fn accept_final(&mut self, text: String) {
        self.accepted_final = Some(text);
    }

    pub fn mark_injection_success(&mut self) {
        self.injection_status = InjectionStatus::Succeeded;
    }

    pub fn mark_injection_failed(&mut self, reason: String) {
        self.injection_status = InjectionStatus::Failed(reason);
    }

    /// Highest quality available text that was preserved.
    /// Order of preference: accepted_final > stage_a_final > raw_asr_final.
    pub fn best_preserved_text(&self) -> Option<&str> {
        self.accepted_final
            .as_deref()
            .or(self.stage_a_final.as_deref())
            .or(self.raw_asr_final.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_state_transitions_full_cycle() {
        let sid = SessionId::next();
        let mut state = DictationState::Idle;

        assert!(state
            .transition_to(DictationState::Preparing { session_id: sid })
            .is_ok());
        assert!(state
            .transition_to(DictationState::Listening {
                session_id: sid,
                started_at: Instant::now()
            })
            .is_ok());
        assert!(state
            .transition_to(DictationState::FinalizingAsr { session_id: sid })
            .is_ok());
        assert!(state
            .transition_to(DictationState::Refining {
                session_id: sid,
                raw_transcript: "test".into()
            })
            .is_ok());
        assert!(state
            .transition_to(DictationState::Verifying {
                session_id: sid,
                candidate_transcript: "test.".into()
            })
            .is_ok());
        assert!(state
            .transition_to(DictationState::Injecting {
                session_id: sid,
                final_transcript: "test.".into()
            })
            .is_ok());
        assert!(state
            .transition_to(DictationState::Completed {
                session_id: sid,
                final_transcript: "test.".into()
            })
            .is_ok());
        assert!(state.transition_to(DictationState::Idle).is_ok());
    }

    #[test]
    fn test_fast_path_transition() {
        let sid = SessionId::next();
        let mut state = DictationState::Preparing { session_id: sid };
        state
            .transition_to(DictationState::Listening {
                session_id: sid,
                started_at: Instant::now(),
            })
            .unwrap();
        state
            .transition_to(DictationState::FinalizingAsr { session_id: sid })
            .unwrap();
        // Fast path: bypass Refining/Verifying directly to Injecting
        assert!(state
            .transition_to(DictationState::Injecting {
                session_id: sid,
                final_transcript: "test".into()
            })
            .is_ok());
    }

    #[test]
    fn test_rejects_invalid_transitions() {
        let sid = SessionId::next();
        let mut state = DictationState::Idle;
        // Cannot jump directly from Idle to Refining
        assert!(state
            .transition_to(DictationState::Refining {
                session_id: sid,
                raw_transcript: "test".into()
            })
            .is_err());

        // Cannot jump directly from Listening to Injecting
        let mut listening = DictationState::Listening {
            session_id: sid,
            started_at: Instant::now(),
        };
        assert!(listening
            .transition_to(DictationState::Injecting {
                session_id: sid,
                final_transcript: "test".into()
            })
            .is_err());
    }

    #[test]
    fn test_recoverable_failure_and_text_preservation() {
        let sid = SessionId::next();
        let mut buffer = RecoveryBuffer::new(sid);
        buffer.preserve_asr_final("hello world raw".to_string());
        buffer.preserve_stage_a("Hello world raw.".to_string());
        buffer.accept_final("Hello world, raw.".to_string());
        buffer.mark_injection_failed("target window lost".to_string());

        assert_eq!(buffer.best_preserved_text(), Some("Hello world, raw."));

        let mut state = DictationState::Injecting {
            session_id: sid,
            final_transcript: "Hello world, raw.".to_string(),
        };
        let fail = DictationState::RecoverableFailure {
            session_id: sid,
            preserved_transcript: buffer.best_preserved_text().map(String::from),
            error: "injection failed".to_string(),
        };
        assert!(state.transition_to(fail).is_ok());
    }
}
