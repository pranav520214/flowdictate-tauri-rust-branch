//! # flowdictate-inject
//!
//! Text injection, global hotkey management, and application context detection.
//!
//! ## Modules
//!
//! - **hotkey** — Global hotkey registration (push-to-talk trigger)
//! - **inject** — Text insertion into the focused application
//! - **context** — Focused application detection and context permission ladder
//!
//! ## Security
//!
//! - The injection subsystem MUST NOT interpret transcript text as executable commands
//! - Plain dictation and future action execution are completely separate capabilities
//! - Context gathering uses the smallest available scope (permission ladder)

pub mod context;
pub mod hotkey;
pub mod inject;
