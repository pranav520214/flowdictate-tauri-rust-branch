//! # Worker Process Management
//!
//! Manages native inference workers in separate isolated processes (§4).
//! Uses the IPC protocol over standard I/O (stdin/stdout).

use crate::ipc::{read_message, write_message, IpcError, WorkerRequest, WorkerResponse};
use std::process::{Child, Command, Stdio};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("failed to spawn worker process: {0}")]
    SpawnFailed(std::io::Error),
    #[error("worker process exited unexpectedly")]
    Exited,
    #[error("IPC communication error: {0}")]
    Ipc(#[from] IpcError),
    #[error("worker initialization failed: {0}")]
    InitializationFailed(String),
    #[error("worker timeout")]
    Timeout,
}

pub struct WorkerProcess {
    child: Child,
    // We use Arc<Mutex> here to allow concurrent reads/writes from the parent.
    // In a full implementation, you'd want dedicated reader/writer threads.
}

impl WorkerProcess {
    /// Spawn a new worker process.
    pub fn spawn(executable_path: &std::path::Path) -> Result<Self, WorkerError> {
        let child = Command::new(executable_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(WorkerError::SpawnFailed)?;

        Ok(Self { child })
    }

    /// Send a request to the worker and wait for a single response.
    /// Does not implement timeouts currently; in production, you'd wrap this
    /// in a timeout future or select! macro.
    pub fn send_request(&mut self, req: &WorkerRequest) -> Result<WorkerResponse, WorkerError> {
        let stdin = self.child.stdin.as_mut().ok_or(WorkerError::Exited)?;
        write_message(stdin, req)?;

        let stdout = self.child.stdout.as_mut().ok_or(WorkerError::Exited)?;
        let resp = read_message(stdout)?;
        Ok(resp)
    }

    /// Initialize the worker and wait for the Ready response.
    pub fn initialize(
        &mut self,
        model_path: String,
        backend: crate::backend::ComputeBackend,
    ) -> Result<(), WorkerError> {
        let req = WorkerRequest::Initialize {
            model_path,
            backend,
        };
        let resp = self.send_request(&req)?;

        match resp {
            WorkerResponse::Ready => Ok(()),
            WorkerResponse::Error { message } => Err(WorkerError::InitializationFailed(message)),
            _ => Err(WorkerError::InitializationFailed(
                "unexpected response".to_string(),
            )),
        }
    }

    /// Stop the worker cleanly.
    pub fn shutdown(&mut self) {
        if let Some(stdin) = self.child.stdin.as_mut() {
            let _ = write_message(stdin, &WorkerRequest::Shutdown);
        }
        let _ = self.child.wait();
    }
}

impl Drop for WorkerProcess {
    fn drop(&mut self) {
        self.shutdown();
    }
}
