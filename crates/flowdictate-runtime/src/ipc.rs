//! # Inter-Process Communication
//!
//! Implements the authenticated local IPC protocol (§16).
//!
//! Provides length-prefixed binary framing over any `Read` / `Write` stream.
//! Enforces strict maximum message sizes to prevent memory exhaustion attacks
//! from compromised native worker processes.

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use thiserror::Error;

/// Current version of the IPC protocol.
pub const PROTOCOL_VERSION: u32 = 1;

/// Maximum payload size in bytes (16 MiB).
/// Prevents allocation bombs from malformed IPC messages.
pub const MAX_PAYLOAD_SIZE: usize = 16 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("payload exceeds maximum size: {size} > {max}")]
    PayloadTooLarge { size: usize, max: usize },
    #[error("protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },
}

/// A request sent from the parent process to a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerRequest {
    /// Initialize the worker with a verified model path.
    Initialize {
        model_path: String,
        backend: crate::backend::ComputeBackend,
    },
    /// Feed a chunk of audio to the ASR worker.
    AudioChunk { samples: Vec<f32> },
    /// Request refinement of a text segment.
    Refine { text: String },
    /// Reset the worker state.
    Reset,
    /// Gracefully shutdown the worker.
    Shutdown,
}

/// A response sent from a worker to the parent process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerResponse {
    /// Worker successfully initialized.
    Ready,
    /// ASR transcript event.
    Transcript {
        is_final: bool,
        is_authoritative: bool,
        text: String,
    },
    /// Refined text from the LLM.
    RefinedText { text: String },
    /// Worker encountered an error.
    Error { message: String },
}

/// Write a framed message to a stream.
///
/// Framing format:
/// `[version: u32][payload_len: u32][bincode_payload]`
pub fn write_message<W: Write, M: Serialize>(writer: &mut W, message: &M) -> Result<(), IpcError> {
    let payload = bincode::serialize(message)?;

    if payload.len() > MAX_PAYLOAD_SIZE {
        return Err(IpcError::PayloadTooLarge {
            size: payload.len(),
            max: MAX_PAYLOAD_SIZE,
        });
    }

    writer.write_all(&PROTOCOL_VERSION.to_le_bytes())?;
    writer.write_all(&(payload.len() as u32).to_le_bytes())?;
    writer.write_all(&payload)?;
    writer.flush()?;

    Ok(())
}

/// Read a framed message from a stream.
pub fn read_message<R: Read, M: for<'de> Deserialize<'de>>(reader: &mut R) -> Result<M, IpcError> {
    let mut version_bytes = [0u8; 4];
    reader.read_exact(&mut version_bytes)?;
    let version = u32::from_le_bytes(version_bytes);

    if version != PROTOCOL_VERSION {
        return Err(IpcError::VersionMismatch {
            expected: PROTOCOL_VERSION,
            actual: version,
        });
    }

    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes)?;
    let payload_len = u32::from_le_bytes(len_bytes) as usize;

    if payload_len > MAX_PAYLOAD_SIZE {
        return Err(IpcError::PayloadTooLarge {
            size: payload_len,
            max: MAX_PAYLOAD_SIZE,
        });
    }

    let mut payload = vec![0u8; payload_len];
    reader.read_exact(&mut payload)?;

    let message = bincode::deserialize(&payload)?;
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_request() {
        let mut buf = Vec::new();
        let req = WorkerRequest::Refine {
            text: "Hello world".to_string(),
        };

        write_message(&mut buf, &req).unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let decoded: WorkerRequest = read_message(&mut cursor).unwrap();

        match decoded {
            WorkerRequest::Refine { text } => assert_eq!(text, "Hello world"),
            _ => panic!("wrong variant"),
        }
    }
}
