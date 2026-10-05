//! # Worker Supervisor & Lifecycle Management (§21)
//!
//! Supervises native inference worker processes with state tracking and
//! bounded restart policies to prevent infinite respawn loops.

use crate::backend::ComputeBackend;
use crate::worker::{WorkerError, WorkerProcess};
use std::path::{Path, PathBuf};

/// Discrete lifecycle states of an isolated native worker (§21).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStatus {
    NotStarted,
    Starting,
    Ready,
    Busy,
    Crashed,
    Restarting,
    Disabled,
}

/// Bounded worker supervisor managing child process restarts and state.
pub struct WorkerSupervisor {
    name: String,
    executable_path: PathBuf,
    process: Option<WorkerProcess>,
    status: WorkerStatus,
    restart_count: u32,
    max_restarts: u32,
}

impl WorkerSupervisor {
    /// Create a new supervisor for a named worker with bounded restarts.
    pub fn new<P: AsRef<Path>>(name: &str, executable_path: P, max_restarts: u32) -> Self {
        Self {
            name: name.to_string(),
            executable_path: executable_path.as_ref().to_path_buf(),
            process: None,
            status: WorkerStatus::NotStarted,
            restart_count: 0,
            max_restarts,
        }
    }

    /// Current lifecycle status of the worker.
    pub fn status(&self) -> WorkerStatus {
        self.status
    }

    /// Number of restarts executed so far.
    pub fn restart_count(&self) -> u32 {
        self.restart_count
    }

    /// Spawn and initialize the worker process if not already running.
    pub fn ensure_ready(
        &mut self,
        model_path: &str,
        backend: ComputeBackend,
    ) -> Result<(), WorkerError> {
        if self.status == WorkerStatus::Disabled {
            return Err(WorkerError::InitializationFailed(format!(
                "worker {} is disabled due to repeated crashes",
                self.name
            )));
        }

        if self.status == WorkerStatus::Ready && self.process.is_some() {
            return Ok(());
        }

        self.status = WorkerStatus::Starting;
        tracing::info!(event = "worker_starting", worker = %self.name);

        let mut proc = WorkerProcess::spawn(&self.executable_path)?;
        proc.initialize(model_path.to_string(), backend)?;

        self.process = Some(proc);
        self.status = WorkerStatus::Ready;
        tracing::info!(event = "worker_ready", worker = %self.name);
        Ok(())
    }

    /// Record a worker failure and attempt bounded restart.
    pub fn handle_crash(&mut self) -> Result<(), WorkerError> {
        self.process = None;
        self.status = WorkerStatus::Crashed;
        tracing::warn!(
            event = "worker_crashed",
            worker = %self.name,
            restarts = self.restart_count
        );

        if self.restart_count < self.max_restarts {
            self.restart_count += 1;
            self.status = WorkerStatus::Restarting;
            tracing::info!(
                event = "worker_restarting",
                worker = %self.name,
                attempt = self.restart_count
            );
            Ok(())
        } else {
            self.status = WorkerStatus::Disabled;
            tracing::error!(
                event = "worker_disabled_max_restarts_exceeded",
                worker = %self.name,
                max = self.max_restarts
            );
            Err(WorkerError::InitializationFailed(format!(
                "worker {} permanently disabled for this session",
                self.name
            )))
        }
    }

    /// Gracefully shutdown the worker process.
    pub fn shutdown(&mut self) {
        if let Some(mut proc) = self.process.take() {
            proc.shutdown();
        }
        self.status = WorkerStatus::NotStarted;
    }
}

impl Drop for WorkerSupervisor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervisor_initial_state() {
        let sup = WorkerSupervisor::new("test-worker", "dummy_path", 3);
        assert_eq!(sup.status(), WorkerStatus::NotStarted);
        assert_eq!(sup.restart_count(), 0);
    }

    #[test]
    fn test_bounded_restart_policy() {
        let mut sup = WorkerSupervisor::new("test-worker", "dummy_path", 2);

        // Crash 1: restarts
        assert!(sup.handle_crash().is_ok());
        assert_eq!(sup.status(), WorkerStatus::Restarting);
        assert_eq!(sup.restart_count(), 1);

        // Crash 2: restarts
        assert!(sup.handle_crash().is_ok());
        assert_eq!(sup.status(), WorkerStatus::Restarting);
        assert_eq!(sup.restart_count(), 2);

        // Crash 3: exceeds max_restarts -> Disabled!
        assert!(sup.handle_crash().is_err());
        assert_eq!(sup.status(), WorkerStatus::Disabled);

        // Cannot start when disabled
        let res = sup.ensure_ready("model.gguf", ComputeBackend::Cpu);
        assert!(res.is_err());
    }
}
