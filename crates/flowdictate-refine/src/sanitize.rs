//! # Output Sanitization
//!
//! Validates and sanitizes refined text before injection into the target application.
//!
//! ## Invariant
//!
//! Output text is NEVER treated as executable instructions or shell commands.
//! Any control characters, NUL bytes, or oversized payloads are stripped or rejected.

use thiserror::Error;

/// Maximum allowed injected text length in bytes (64 KiB).
pub const MAX_INJECTION_BYTES: usize = 65_536;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SanitizeError {
    #[error("text exceeds maximum length: {size} > {max}")]
    TooLong { size: usize, max: usize },
    #[error("text contains forbidden NUL byte")]
    NullByteFound,
}

/// Sanitize text for safe OS-level injection.
///
/// Removes any invalid control characters (except `\n` and `\t`),
/// rejects NUL bytes, and verifies length limits.
pub fn sanitize_output(input: &str) -> Result<String, SanitizeError> {
    if input.len() > MAX_INJECTION_BYTES {
        return Err(SanitizeError::TooLong {
            size: input.len(),
            max: MAX_INJECTION_BYTES,
        });
    }

    if input.contains('\0') {
        return Err(SanitizeError::NullByteFound);
    }

    // Filter out ASCII control characters except \n (0x0A) and \t (0x09)
    let sanitized: String = input
        .chars()
        .filter(|&c| {
            if c == '\n' || c == '\t' {
                true
            } else {
                !c.is_control()
            }
        })
        .collect();

    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_valid_text() {
        let text = "Hello world!\nThis is a tab:\there.";
        let res = sanitize_output(text);
        assert_eq!(res, Ok(text.to_string()));
    }

    #[test]
    fn rejects_null_byte() {
        let text = "Hello\0World";
        assert_eq!(sanitize_output(text), Err(SanitizeError::NullByteFound));
    }

    #[test]
    fn strips_control_characters() {
        // \x07 is bell, \x08 is backspace
        let text = "Hello\x07 \x08World";
        let res = sanitize_output(text).unwrap();
        assert_eq!(res, "Hello World");
    }

    #[test]
    fn rejects_oversized_text() {
        let big = "a".repeat(MAX_INJECTION_BYTES + 1);
        assert!(matches!(
            sanitize_output(&big),
            Err(SanitizeError::TooLong { .. })
        ));
    }
}
