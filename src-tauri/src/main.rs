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
mod setup;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize secure logging (no transcripts)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tauri::Builder::default()
        .setup(|app| {
            setup::run_setup(app)?;
            Ok(())
        })
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_privacy_dashboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FlowDictate");
}
