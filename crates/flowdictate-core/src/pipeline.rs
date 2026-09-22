//! # Pipeline Orchestrator
//!
//! Coordinates the full dictation pipeline across dedicated OS threads with
//! explicit session state validation, monotonic latency tracking, and
//! a volatile no-loss recovery buffer (§4, §7, §29, §36).

use crossbeam_channel::{bounded, Receiver, Sender};
use flowdictate_asr::consensus::TranscriptStabilizer;
use flowdictate_asr::engine::{NoopRecognizer, Recognizer};
use flowdictate_asr::transcript::TranscriptEvent;
use flowdictate_inject::inject::TextInjector;
use flowdictate_refine::commands::VoiceCommand;
use flowdictate_refine::llm::NoopRefiner;
use flowdictate_refine::router::{RefinedOutcome, RefinementPipeline};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use crate::metrics::{ReliabilityCounters, ReliabilitySnapshot, SessionLatencyTracker};
use crate::session::{DictationState, RecoveryBuffer, SessionId};

/// Events sent to update the UI overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiUpdateEvent {
    StateChanged {
        state_label: String,
        session_id: Option<u64>,
    },
    PartialTranscript {
        session_id: u64,
        committed_prefix: String,
        unstable_suffix: String,
        text: String,
    },
    Waveform(Vec<f32>),
    FinalText {
        session_id: u64,
        text: String,
    },
    RecoverableError {
        session_id: u64,
        preserved_text: Option<String>,
        error_message: String,
    },
}

/// Commands to control the pipeline.
pub enum PipelineCommand {
    StartListening,
    StopListening,
    ProcessAudio(Vec<f32>),
    Shutdown,
}

/// The main pipeline coordinator.
pub struct DictationPipeline {
    command_tx: Sender<PipelineCommand>,
    ui_rx: Receiver<UiUpdateEvent>,
    counters: Arc<ReliabilityCounters>,
    is_running: Arc<AtomicBool>,
}

impl DictationPipeline {
    /// Initialize and spawn the pipeline processing thread.
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = bounded::<PipelineCommand>(128);
        let (ui_tx, ui_rx) = bounded::<UiUpdateEvent>(128);
        let counters = Arc::new(ReliabilityCounters::new());
        let is_running = Arc::new(AtomicBool::new(true));

        let running_clone = Arc::clone(&is_running);
        let counters_clone = Arc::clone(&counters);
        thread::Builder::new()
            .name("flowdictate-pipeline".to_string())
            .spawn(move || {
                run_pipeline_loop(cmd_rx, ui_tx, counters_clone, running_clone);
            })
            .expect("failed to spawn pipeline thread");

        Self {
            command_tx: cmd_tx,
            ui_rx,
            counters,
            is_running,
        }
    }

    /// Trigger start listening (push-to-talk press).
    pub fn start_listening(&self) {
        let _ = self.command_tx.send(PipelineCommand::StartListening);
    }

    /// Trigger stop listening (push-to-talk release).
    pub fn stop_listening(&self) {
        let _ = self.command_tx.send(PipelineCommand::StopListening);
    }

    /// Feed audio samples to the active pipeline.
    pub fn feed_audio(&self, samples: Vec<f32>) {
        let _ = self.command_tx.send(PipelineCommand::ProcessAudio(samples));
    }

    /// Receive UI update event (non-blocking).
    pub fn try_recv_ui(&self) -> Option<UiUpdateEvent> {
        self.ui_rx.try_recv().ok()
    }

    /// Snapshot of local reliability metrics (§36).
    pub fn counters(&self) -> ReliabilitySnapshot {
        self.counters.snapshot()
    }
}

impl Default for DictationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DictationPipeline {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        let _ = self.command_tx.send(PipelineCommand::Shutdown);
    }
}

/// Core event loop processing audio, ASR, refinement, and text injection.
fn run_pipeline_loop(
    cmd_rx: Receiver<PipelineCommand>,
    ui_tx: Sender<UiUpdateEvent>,
    counters: Arc<ReliabilityCounters>,
    is_running: Arc<AtomicBool>,
) {
    let mut state = DictationState::Idle;
    let mut current_session: Option<SessionId> = None;
    let mut current_tracker: Option<SessionLatencyTracker> = None;
    let mut current_recovery: Option<RecoveryBuffer> = None;

    let mut recognizer = NoopRecognizer;
    let mut refiner = NoopRefiner;
    let refinement = RefinementPipeline::new(false);
    let mut injector = TextInjector::new();
    let mut stabilizer = TranscriptStabilizer::default();

    let mut audio_buffer = Vec::<f32>::new();

    while is_running.load(Ordering::Relaxed) {
        let cmd = match cmd_rx.recv() {
            Ok(c) => c,
            Err(_) => break,
        };

        match cmd {
            PipelineCommand::StartListening => {
                // If a previous session is still active, fail-safe finalize it
                if let Some(sid) = current_session {
                    tracing::warn!(event = "session_preempted", previous_session = sid.0);
                }

                let session_id = SessionId::next();
                counters.inc_started();

                let mut tracker = SessionLatencyTracker::new();
                tracker.record_hotkey_detect();
                tracker.record_audio_ready();

                let recovery = RecoveryBuffer::new(session_id);

                // Transition through state machine: Idle -> Preparing -> Listening
                state = DictationState::Idle;
                let _ = state.transition_to(DictationState::Preparing { session_id });
                let _ = state.transition_to(DictationState::Listening {
                    session_id,
                    started_at: Instant::now(),
                });

                current_session = Some(session_id);
                current_tracker = Some(tracker);
                current_recovery = Some(recovery);

                audio_buffer.clear();
                recognizer.reset();
                stabilizer.reset();

                let _ = ui_tx.send(UiUpdateEvent::StateChanged {
                    state_label: "Listening".to_string(),
                    session_id: Some(session_id.0),
                });
            }
            PipelineCommand::ProcessAudio(chunk) => {
                if let (
                    Some(session_id),
                    DictationState::Listening {
                        session_id: current_sid,
                        ..
                    },
                ) = (current_session, &state)
                {
                    if session_id == *current_sid {
                        if let Some(ref mut tracker) = current_tracker {
                            tracker.record_first_speech();
                        }

                        audio_buffer.extend_from_slice(&chunk);

                        // Compute basic RMS level for waveform display
                        if !chunk.is_empty() {
                            let rms = (chunk.iter().map(|&s| s * s).sum::<f32>()
                                / chunk.len() as f32)
                                .sqrt();
                            let _ = ui_tx.send(UiUpdateEvent::Waveform(vec![rms]));
                        }

                        // Feed audio to recognizer
                        if let Ok(events) = recognizer.feed_audio(&chunk) {
                            for evt in events {
                                if let TranscriptEvent::Partial(raw) = evt {
                                    if let Some(ref mut tracker) = current_tracker {
                                        tracker.record_first_partial();
                                    }
                                    let view = stabilizer.update(&raw);
                                    let _ = ui_tx.send(UiUpdateEvent::PartialTranscript {
                                        session_id: session_id.0,
                                        committed_prefix: view.committed_prefix,
                                        unstable_suffix: view.unstable_suffix,
                                        text: view.display_text,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            PipelineCommand::StopListening => {
                if let (
                    Some(session_id),
                    DictationState::Listening {
                        session_id: current_sid,
                        ..
                    },
                ) = (current_session, &state)
                {
                    if session_id == *current_sid {
                        if let Some(ref mut tracker) = current_tracker {
                            tracker.record_hotkey_release();
                        }

                        // Transition: Listening -> FinalizingAsr
                        let _ = state.transition_to(DictationState::FinalizingAsr { session_id });
                        let _ = ui_tx.send(UiUpdateEvent::StateChanged {
                            state_label: "Finalizing".to_string(),
                            session_id: Some(session_id.0),
                        });

                        // Simulated final ASR transcript (will use real Nemotron events in Phase C)
                        let raw_transcript = "hello world period";

                        if let Some(ref mut tracker) = current_tracker {
                            tracker.record_asr_final();
                        }
                        if let Some(ref mut recovery) = current_recovery {
                            recovery.preserve_asr_final(raw_transcript.to_string());
                        }

                        // Transition: FinalizingAsr -> Refining
                        let _ = state.transition_to(DictationState::Refining {
                            session_id,
                            raw_transcript: raw_transcript.to_string(),
                        });

                        if let Some(ref mut tracker) = current_tracker {
                            tracker.record_stage_a();
                        }

                        // Process through refinement pipeline
                        let outcome = refinement.process(raw_transcript, &mut refiner);

                        match outcome {
                            Ok(RefinedOutcome::Command(cmd)) => {
                                match cmd {
                                    VoiceCommand::NewLine => {
                                        let _ = injector.inject_text("\n");
                                    }
                                    VoiceCommand::NewParagraph => {
                                        let _ = injector.inject_text("\n\n");
                                    }
                                    VoiceCommand::Undo => {
                                        // Backspace simulation placeholder
                                    }
                                    VoiceCommand::DeletePreviousWord => {
                                        let _ = injector.inject_backspace(5);
                                    }
                                    _ => {}
                                }
                                counters.inc_completed();
                                let _ = state.transition_to(DictationState::Completed {
                                    session_id,
                                    final_transcript: String::new(),
                                });
                            }
                            Ok(RefinedOutcome::Text(text)) => {
                                if let Some(ref mut recovery) = current_recovery {
                                    recovery.accept_final(text.clone());
                                }

                                // Transition: Refining -> Injecting
                                let _ = state.transition_to(DictationState::Injecting {
                                    session_id,
                                    final_transcript: text.clone(),
                                });

                                if let Some(ref mut tracker) = current_tracker {
                                    tracker.record_inject_start();
                                }

                                let _ = ui_tx.send(UiUpdateEvent::FinalText {
                                    session_id: session_id.0,
                                    text: text.clone(),
                                });

                                // Perform injection
                                match injector.inject_text(&text) {
                                    Ok(()) => {
                                        if let Some(ref mut tracker) = current_tracker {
                                            tracker.record_inject_complete();
                                            if let Some(dur) = tracker.total_turnaround() {
                                                tracing::info!(
                                                    event = "session_turnaround_ms",
                                                    session_id = session_id.0,
                                                    ms = dur.as_millis()
                                                );
                                            }
                                        }
                                        if let Some(ref mut recovery) = current_recovery {
                                            recovery.mark_injection_success();
                                        }
                                        counters.inc_completed();
                                        let _ = state.transition_to(DictationState::Completed {
                                            session_id,
                                            final_transcript: text,
                                        });
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            event = "injection_failed",
                                            session_id = session_id.0,
                                            error = %e
                                        );
                                        counters.inc_injection_failure();
                                        if let Some(ref mut recovery) = current_recovery {
                                            recovery.mark_injection_failed(e.to_string());
                                        }

                                        let preserved = current_recovery
                                            .as_ref()
                                            .and_then(|r| r.best_preserved_text())
                                            .map(String::from);

                                        let _ = state.transition_to(
                                            DictationState::RecoverableFailure {
                                                session_id,
                                                preserved_transcript: preserved.clone(),
                                                error: e.to_string(),
                                            },
                                        );

                                        let _ = ui_tx.send(UiUpdateEvent::RecoverableError {
                                            session_id: session_id.0,
                                            preserved_text: preserved,
                                            error_message:
                                                "Text injection failed; captured speech preserved."
                                                    .to_string(),
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!(event = "refinement_failed", session_id = session_id.0, error = %e);
                                counters.inc_verifier_rejection();
                                let preserved = current_recovery
                                    .as_ref()
                                    .and_then(|r| r.best_preserved_text())
                                    .map(String::from);

                                let _ = state.transition_to(DictationState::RecoverableFailure {
                                    session_id,
                                    preserved_transcript: preserved.clone(),
                                    error: e.to_string(),
                                });

                                let _ = ui_tx.send(UiUpdateEvent::RecoverableError {
                                    session_id: session_id.0,
                                    preserved_text: preserved,
                                    error_message:
                                        "Text processing error; captured speech preserved."
                                            .to_string(),
                                });
                            }
                        }

                        // Clean reset back to Idle
                        let _ = state.transition_to(DictationState::Idle);
                        let _ = ui_tx.send(UiUpdateEvent::StateChanged {
                            state_label: "Idle".to_string(),
                            session_id: None,
                        });
                        current_session = None;
                    }
                }
            }
            PipelineCommand::Shutdown => {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_pipeline_session_lifecycle() {
        let pipeline = DictationPipeline::new();
        pipeline.start_listening();

        // Feed some mock audio
        pipeline.stop_listening();

        let start = std::time::Instant::now();
        while (pipeline.counters().sessions_completed == 0
            && pipeline.counters().injection_failures == 0)
            && start.elapsed() < Duration::from_secs(2)
        {
            thread::sleep(Duration::from_millis(20));
        }

        let counters = pipeline.counters();
        assert_eq!(counters.sessions_started, 1);
        assert!(counters.sessions_completed == 1 || counters.injection_failures == 1);
    }
}
