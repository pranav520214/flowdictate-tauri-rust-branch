//! # Memory Security Utilities
//!
//! Provides wrappers for sensitive data that ensure zeroization on drop.
//!
//! ## Guarantees and Limitations
//!
//! - Rust-owned memory: `zeroize` crate provides best-effort zeroization using
//!   volatile writes and compiler fences to prevent optimization.
//! - C++ runtime memory (whisper.cpp, llama.cpp): Internal buffers within the
//!   C++ inference runtimes are NOT zeroizable from Rust. This is documented
//!   as a residual risk in `docs/THREAT_MODEL.md`.
//! - OS/compiler behavior: The OS may page sensitive memory to swap. The compiler
//!   may create temporary copies. These limitations are documented honestly.
//!
//! ## Usage
//!
//! Wrap sensitive byte buffers in [`SensitiveBuffer`] to ensure they are cleared
//! when dropped. Use [`SensitiveString`] for text data like transcripts.

use zeroize::{Zeroize, ZeroizeOnDrop};

/// A byte buffer that is zeroized when dropped.
///
/// Use for: temporary audio buffers, encryption keys, intermediate transcripts.
#[derive(Clone, ZeroizeOnDrop)]
pub struct SensitiveBuffer {
    #[zeroize]
    data: Vec<u8>,
}

impl SensitiveBuffer {
    /// Create a new sensitive buffer with the given data.
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Access the buffer contents.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Access the buffer contents mutably.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get the length of the buffer.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// A string that is zeroized when dropped.
///
/// Use for: temporary transcript text, refinement prompts.
/// NEVER use for: error messages, log output, debug display.
#[derive(Clone, ZeroizeOnDrop)]
pub struct SensitiveString {
    #[zeroize]
    data: String,
}

impl SensitiveString {
    /// Create a new sensitive string.
    pub fn new(data: String) -> Self {
        Self { data }
    }

    /// Access the string contents.
    pub fn as_str(&self) -> &str {
        &self.data
    }
}

// Deliberately NOT implementing Display, Debug, or Into<String> for sensitive types
// to prevent accidental logging or inclusion in error messages.

// TODO(milestone-6): Add mlock/VirtualLock support to prevent paging to swap
// TODO(milestone-9): Document exact zeroization coverage and gaps
