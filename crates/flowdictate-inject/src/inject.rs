//! # Text Injection
//!
//! Inserts refined text into the currently focused application.
//!
//! ## Strategy
//!
//! 1. Prefer native OS accessibility/text-input mechanisms where available
//! 2. Fall back to simulated keyboard input via `enigo` where necessary
//!
//! ## Security
//!
//! - Transcript text is NEVER interpreted as executable commands
//! - The injection subsystem only types characters, it does not execute them
//! - Plain dictation and future action execution are completely separate

// TODO(milestone-4): Implement text injection via enigo
// TODO(milestone-4): Investigate native alternatives per platform
// TODO(milestone-4): Implement error reporting for injection failures
