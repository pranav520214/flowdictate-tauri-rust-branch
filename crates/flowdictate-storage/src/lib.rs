//! # flowdictate-storage
//!
//! Encrypted local storage and credential management for FlowDictate.
//!
//! ## Architecture
//!
//! - **Database**: SQLCipher-encrypted SQLite for all persistent data
//! - **Credentials**: OS-native keyring for encryption key storage
//! - **History**: Three modes (Off, SessionOnly, EncryptedPersistent)
//! - **Profile**: User personalization data with import/export
//!
//! ## Key Management
//!
//! The database encryption key is:
//! - Generated randomly on first launch
//! - Stored in the OS-native credential store (Windows Credential Manager,
//!   macOS Keychain, Linux Secret Service)
//! - NEVER hard-coded, logged, or stored alongside the database
//! - NEVER derived from transcription text or spoken words

pub mod credentials;
pub mod database;
pub mod history;
pub mod profile;
