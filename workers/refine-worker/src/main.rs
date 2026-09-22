//! # Refinement Worker Process (§4, §11)
//!
//! Isolated process running llama.cpp for Qwen3 transcript refinement.
//! Communicates with the main process via length-prefixed IPC over standard I/O.

use flowdictate_runtime::ipc::{read_message, write_message, WorkerRequest, WorkerResponse};
use flowdictate_runtime::native_ffi::LlamaEngine;
use flowdictate_runtime::resource_paths::AppResourcePaths;
use std::io::{stdin, stdout};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("refine-worker started and awaiting IPC initialization");

    let mut stdin_lock = stdin().lock();
    let mut stdout_lock = stdout().lock();
    let mut engine: Option<LlamaEngine> = None;
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
                backend: _,
            } => {
                let dll_path = paths.dll_dir().join("llama.dll");
                tracing::info!(event = "initializing_qwen", model = %model_path);

                match LlamaEngine::load(Some(dll_path), &model_path) {
                    Ok(eng) => {
                        engine = Some(eng);
                        let _ = write_message(&mut stdout_lock, &WorkerResponse::Ready);
                    }
                    Err(err) => {
                        tracing::error!(event = "qwen_init_failed", error = %err);
                        let _ = write_message(
                            &mut stdout_lock,
                            &WorkerResponse::Error { message: err },
                        );
                    }
                }
            }
            WorkerRequest::Refine { text } => {
                if let Some(ref mut eng) = engine {
                    match eng.refine(&text) {
                        Ok(refined) => {
                            let _ = write_message(
                                &mut stdout_lock,
                                &WorkerResponse::RefinedText { text: refined },
                            );
                        }
                        Err(err) => {
                            tracing::warn!(event = "refine_failed", error = %err);
                            let _ = write_message(
                                &mut stdout_lock,
                                &WorkerResponse::Error { message: err },
                            );
                        }
                    }
                } else {
                    let _ = write_message(
                        &mut stdout_lock,
                        &WorkerResponse::Error {
                            message: "Refinement engine not initialized".to_string(),
                        },
                    );
                }
            }
            WorkerRequest::Reset => {
                // LLM is stateless across requests
            }
            WorkerRequest::Shutdown => {
                tracing::info!("refine-worker shutting down cleanly");
                break;
            }
            _ => {
                let _ = write_message(
                    &mut stdout_lock,
                    &WorkerResponse::Error {
                        message: "unsupported request in refine-worker".to_string(),
                    },
                );
            }
        }
    }
}
