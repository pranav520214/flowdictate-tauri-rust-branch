//! # flowdictate-runtime
//!
//! Owns hardware detection, accelerator selection, model lifecycle,
//! worker process management, and the memory governor.
//!
//! ## Trust Boundary
//!
//! This crate is the bridge between the trusted Rust core and the
//! untrusted native inference runtimes (NeMo-Speech.cpp, llama.cpp).
//! It must never trust output from native runtimes without validation.
//!
//! ## Responsibilities (§5)
//!
//! - CPU architecture detection (x86_64 / ARM64)
//! - Accelerator detection (CUDA / Metal / Vulkan / CPU)
//! - Backend selection with safe fallback
//! - Model lifecycle (load / unload / verify-before-load)
//! - Worker process lifecycle (spawn / health-check / restart / disable)
//! - Memory governor (sequential / concurrent model residency)
//! - Resource budgets and bounds

pub mod backend;
pub mod detect;
pub mod governor;
pub mod ipc;
pub mod native_ffi;
pub mod resource_paths;
pub mod supervisor;
pub mod worker;

/// Maximum number of worker restart attempts before disabling for the session.
pub const MAX_WORKER_RESTARTS: u32 = 3;

/// Maximum time to wait for a worker process to respond before declaring it dead.
pub const WORKER_HEALTH_TIMEOUT_MS: u64 = 5000;
