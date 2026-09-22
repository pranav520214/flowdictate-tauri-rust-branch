//! # History Modes
//!
//! Controls transcript persistence behavior:
//!
//! - **Off**: No transcripts persisted. No crash recovery. No hidden caches.
//! - **SessionOnly**: Encrypted crash recovery during session. Deleted on clean exit.
//! - **EncryptedPersistent**: Full history in SQLCipher DB.

// TODO(milestone-6): Implement history mode enum and transitions
// TODO(milestone-6): Implement session-only cleanup on exit
// TODO(milestone-6): Implement "Delete all local data" control
