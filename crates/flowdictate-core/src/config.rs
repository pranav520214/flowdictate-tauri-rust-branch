//! # Application Configuration
//!
//! Defines all user-configurable settings for FlowDictate.

use serde::{Deserialize, Serialize};

/// Performance mode controlling resource usage and model selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum PerformanceMode {
    /// Smallest model, lowest memory, deterministic refinement only.
    Eco,
    /// Smallest ASR meeting quality targets, tiny LLM warm when memory permits.
    #[default]
    Balanced,
    /// Larger local models if installed. Still entirely offline.
    Quality,
}

/// Context permission level for application awareness.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum ContextLevel {
    /// No external application context (default).
    #[default]
    None,
    /// Current application identity only.
    AppIdentity,
    /// Selected text or current textbox content.
    SelectedText,
    /// Additional explicitly approved context.
    Extended,
}

/// History persistence mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum HistoryMode {
    /// No transcript persistence. No crash recovery.
    Off,
    /// Encrypted crash recovery during session. Deleted on clean exit.
    #[default]
    SessionOnly,
    /// Full history in encrypted database.
    EncryptedPersistent,
}

/// Complete application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub performance_mode: PerformanceMode,
    pub context_level: ContextLevel,
    pub history_mode: HistoryMode,
    pub personalization_enabled: bool,
    pub hotkey: String,
    pub language: String,
    pub silence_threshold_ms: u32,
    pub max_session_duration_s: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            performance_mode: PerformanceMode::default(),
            context_level: ContextLevel::default(),
            history_mode: HistoryMode::default(),
            personalization_enabled: true,
            hotkey: "CmdOrCtrl+Shift+Space".to_string(),
            language: "en".to_string(),
            silence_threshold_ms: 1500,
            max_session_duration_s: 300,
        }
    }
}
