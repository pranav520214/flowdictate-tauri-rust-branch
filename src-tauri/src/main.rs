//! FlowDictate — Privacy-First Local Voice Computing
//!
//! This is the Tauri v2 application entry point.
//!
//! ## Logging Policy
//!
//! Release builds use a restricted log filter that NEVER captures:
//! - Transcript text
//! - Audio data
//! - User dictionary contents
//! - Encryption keys
//! - Clipboard data
//!
//! Debug-level logging with richer output is only available in dev builds.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod state;

fn main() {
    // Initialize logging with security-safe filter
    // Release builds: info level, no sensitive fields
    // Debug builds: debug level with additional diagnostics
    let log_filter = if cfg!(debug_assertions) {
        "flowdictate=debug"
    } else {
        "flowdictate=info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .init();

    tracing::info!("FlowDictate starting");

    tauri::Builder::default()
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_privacy_dashboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FlowDictate");
}
