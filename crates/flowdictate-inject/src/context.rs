//! # Application Context Detection
//!
//! Detects the currently focused application and optionally gathers
//! context to improve transcription accuracy.
//!
//! ## Permission Ladder (§22)
//!
//! - **Level 0** (default): No external application context
//! - **Level 1**: Current application identity only
//! - **Level 2**: Selected text or current textbox content
//! - **Level 3**: Additional explicitly approved context
//!
//! Context is NEVER gathered without the user's explicit permission level setting.
//! The current permission level is always visible in the privacy dashboard.

// TODO(milestone-4): Implement focused application detection
// TODO(milestone-4): Implement permission ladder enforcement
// TODO(milestone-7): Implement selected text retrieval (Level 2)
