//! # ASR Worker Process (§4, §10)
//!
//! Isolated process running NeMo-Speech.cpp streaming ASR.
//! Communicates with the main process via length-prefixed IPC over standard I/O.

use flowdictate_runtime::ipc::{read_message, write_message, WorkerRequest, WorkerResponse};
use flowdictate_runtime::native_ffi::NemoEngine;
use flowdictate_runtime::resource_paths::AppResourcePaths;
use std::io::{stdin, stdout};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("asr-worker started and awaiting IPC initialization");

    let mut stdin_lock = stdin().lock();
    let mut stdout_lock = stdout().lock();
    let mut engine: Option<NemoEngine> = None;
    let paths = AppResourcePaths::from_current_exe();

    loop {
        let req: Result<WorkerRequest, _> = read_message(&mut stdin_lock);
        let req = match req {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("IPC read error: {}", e);
                break;
            }
        };

        match req {
            WorkerRequest::Initialize {
                model_path,
                backend,
            } => {
                let dll_path = paths.dll_dir().join("nemo-speech.dll");
                tracing::info!(event = "initializing_nemotron", model = %model_path, backend = ?backend);

                let backend_code = match backend {
                    flowdictate_runtime::backend::ComputeBackend::Cuda => 1,
                    flowdictate_runtime::backend::ComputeBackend::Metal => 2,
                    flowdictate_runtime::backend::ComputeBackend::Vulkan => 3,
                    flowdictate_runtime::backend::ComputeBackend::Cpu => 0,
                };

                match NemoEngine::load(Some(dll_path), &model_path, backend_code) {
                    Ok(eng) => {
                        engine = Some(eng);
                        let _ = write_message(&mut stdout_lock, &WorkerResponse::Ready);
                    }
                    Err(err) => {
                        tracing::error!(event = "nemotron_init_failed", error = %err);
                        let _ = write_message(
                            &mut stdout_lock,
                            &WorkerResponse::Error { message: err },
                        );
                    }
                }
            }
            WorkerRequest::AudioChunk { samples } => {
                if let Some(ref mut eng) = engine {
                    let partial_opt = eng.feed_audio(&samples);
                    let text = partial_opt.unwrap_or_default();
                    let resp = WorkerResponse::Transcript {
                        is_final: false,
                        is_authoritative: false,
                        text,
                    };
                    let _ = write_message(&mut stdout_lock, &resp);
                } else {
                    let _ = write_message(
                        &mut stdout_lock,
                        &WorkerResponse::Error {
                            message: "ASR engine not initialized".to_string(),
                        },
                    );
                }
            }
            WorkerRequest::Reset => {
                if let Some(ref mut eng) = engine {
                    eng.reset();
                }
            }
            WorkerRequest::Shutdown => {
                tracing::info!("asr-worker shutting down cleanly");
                break;
            }
            _ => {
                let _ = write_message(
                    &mut stdout_lock,
                    &WorkerResponse::Error {
                        message: "unsupported request in asr-worker".to_string(),
                    },
                );
            }
        }
    }
}
