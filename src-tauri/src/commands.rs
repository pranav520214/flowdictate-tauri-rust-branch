//! Tauri IPC command handlers.
//!
//! These commands are the typed interface between the WebView UI and the
//! Rust backend. They enforce the trust boundary at the IPC level.

use crate::state::AppState;
use serde::Serialize;

/// Pipeline status information (safe to display in UI).
#[derive(Debug, Serialize)]
pub struct PipelineStatus {
    pub state: String,
    pub model_loaded: bool,
    pub model_id: Option<String>,
}

/// Privacy dashboard data.
#[derive(Debug, Serialize)]
pub struct PrivacyDashboard {
    pub network_access: String,
    pub cloud_processing: String,
    pub telemetry: String,
    pub audio_storage: String,
    pub history_mode: String,
    pub context_level: String,
    pub personalization: String,
    pub model: String,
    pub database_encrypted: bool,
}

#[tauri::command]
pub fn get_status(_state: tauri::State<AppState>) -> PipelineStatus {
    PipelineStatus {
        state: "idle".to_string(),
        model_loaded: false,
        model_id: None,
    }
}

#[tauri::command]
pub fn get_privacy_dashboard(_state: tauri::State<AppState>) -> PrivacyDashboard {
    PrivacyDashboard {
        network_access: "NONE".to_string(),
        cloud_processing: "NONE".to_string(),
        telemetry: "OFF / NOT PRESENT".to_string(),
        audio_storage: "OFF".to_string(),
        history_mode: "Session Only".to_string(),
        context_level: "None (Level 0)".to_string(),
        personalization: "Enabled".to_string(),
        model: "Not loaded".to_string(),
        database_encrypted: true,
    }
}

// TODO(milestone-4): Add start_dictation, stop_dictation commands
// TODO(milestone-5): Add real-time waveform data streaming
// TODO(milestone-6): Add settings update commands
