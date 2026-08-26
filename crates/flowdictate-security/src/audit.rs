//! # Local Audit Trail
//!
//! Records security-relevant events to the local encrypted database.
//!
//! Audit events include:
//! - Model loaded / rejected
//! - Profile imported / rejected
//! - Security setting changed
//! - Data deletion performed
//! - Path validation failures
//!
//! ## Privacy
//!
//! Audit entries MUST NOT contain:
//! - Transcript text
//! - Audio data
//! - User dictionary contents
//! - Clipboard data
//! - Application context content
//!
//! They MAY contain:
//! - Event type
//! - Timestamp
//! - Model identifier (not content)
//! - Success/failure status
//! - Anonymous error code

use serde::{Deserialize, Serialize};

/// Categories of auditable security events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventKind {
    /// Model integrity verification result
    ModelVerification { model_id: String, success: bool },
    /// Profile import attempt
    ProfileImport { success: bool },
    /// Security setting changed
    SettingChanged { setting: String },
    /// Data deletion performed
    DataDeletion { scope: String },
    /// Path validation failure
    PathValidationFailure,
    /// Database key rotation
    KeyRotation { success: bool },
}

/// A single audit trail entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// When the event occurred (UTC timestamp)
    pub timestamp: String,
    /// Event category and details
    pub kind: AuditEventKind,
}

// TODO(milestone-6): Implement write_audit_event() to encrypted DB
// TODO(milestone-6): Implement query_audit_trail()
