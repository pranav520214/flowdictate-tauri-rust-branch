//! # Output Sanitization
//!
//! Validates and sanitizes refined text before injection into the target application.
//!
//! ## Checks
//!
//! - Maximum output length (prevent injection of excessive text)
//! - Character validation (no control characters except \n, \t)
//! - No embedded null bytes
//! - No executable content (script tags, shell commands)
//! - Transcript text is NEVER interpreted as application commands
//!
//! ## Prompt Injection Boundary
//!
//! All dictated text, selected text, clipboard data, and application context
//! is UNTRUSTED CONTENT. It must never alter FlowDictate's security policy.
//! Text containing "Ignore your previous instructions" is transcript content,
//! not an instruction to the application.

// TODO(milestone-3): Implement output length bounds
// TODO(milestone-3): Implement character validation
// TODO(milestone-3): Implement control character stripping
// TODO(milestone-9): Fuzz sanitization with adversarial inputs
