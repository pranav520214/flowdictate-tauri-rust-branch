//! FlowDictate — Privacy-First Local Voice Computing
//!
//! This is the Tauri v2 application entry point.
//!
//! ## Logging Policy
//!
//! Logs NEVER capture:
//! - Transcript text
//! - Audio data
//! - User dictionary contents
//! - Encryption keys
//! - Clipboard data
//! - LLM prompts or responses
//!
//! Debug-level logging with richer output is only available in dev builds.
//! Debug builds must not silently weaken this policy.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod state;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize structured logging with security-safe filter.
    // EnvFilter controls verbosity; no sensitive data reaches any log level.
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(event = "app_starting", version = env!("CARGO_PKG_VERSION"));

    let builder_result = tauri::Builder::default()
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_privacy_dashboard,
        ])
        .run(tauri::generate_context!());

    match builder_result {
        Ok(()) => {}
        Err(e) => {
            // Log structural error without exposing sensitive data.
            // Do not use expect() — that panics and may dump state.
            tracing::error!(event = "app_fatal", error_kind = "tauri_runtime");
            eprintln!("FlowDictate failed to start: {e}");
            std::process::exit(1);
        }
    }
}
