//! # Pipeline Latency & Reliability Instrumentation (§7, §36)
//!
//! Provides monotonic timestamp tracking for individual dictation sessions
//! and local-only structural reliability counters.
//!
//! ## Privacy & Security Invariant
//!
//! Under NO circumstances does this module store, log, or transmit transcript text,
//! audio data, or user credentials. All counters and timestamps are structural and
//! remain strictly on the local machine.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Per-session latency tracker measuring critical path checkpoints (§7).
#[derive(Debug, Clone)]
pub struct SessionLatencyTracker {
    pub t_hotkey_detect: Option<Instant>,
    pub t_audio_ready: Option<Instant>,
    pub t_first_speech: Option<Instant>,
    pub t_first_partial: Option<Instant>,
    pub t_hotkey_release: Option<Instant>,
    pub t_asr_final: Option<Instant>,
    pub t_stage_a: Option<Instant>,
    pub t_llm_start: Option<Instant>,
    pub t_llm_complete: Option<Instant>,
    pub t_verify: Option<Instant>,
    pub t_inject_start: Option<Instant>,
    pub t_inject_complete: Option<Instant>,
}

impl Default for SessionLatencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionLatencyTracker {
    /// Create a new session latency tracker initialized at the current instant.
    pub fn new() -> Self {
        Self {
            t_hotkey_detect: Some(Instant::now()),
            t_audio_ready: None,
            t_first_speech: None,
            t_first_partial: None,
            t_hotkey_release: None,
            t_asr_final: None,
            t_stage_a: None,
            t_llm_start: None,
            t_llm_complete: None,
            t_verify: None,
            t_inject_start: None,
            t_inject_complete: None,
        }
    }

    pub fn record_hotkey_detect(&mut self) {
        self.t_hotkey_detect = Some(Instant::now());
    }

    pub fn record_audio_ready(&mut self) {
        self.t_audio_ready = Some(Instant::now());
    }

    pub fn record_first_speech(&mut self) {
        if self.t_first_speech.is_none() {
            self.t_first_speech = Some(Instant::now());
        }
    }

    pub fn record_first_partial(&mut self) {
        if self.t_first_partial.is_none() {
            self.t_first_partial = Some(Instant::now());
        }
    }

    pub fn record_hotkey_release(&mut self) {
        self.t_hotkey_release = Some(Instant::now());
    }

    pub fn record_asr_final(&mut self) {
        self.t_asr_final = Some(Instant::now());
    }

    pub fn record_stage_a(&mut self) {
        self.t_stage_a = Some(Instant::now());
    }

    pub fn record_llm_start(&mut self) {
        self.t_llm_start = Some(Instant::now());
    }

    pub fn record_llm_complete(&mut self) {
        self.t_llm_complete = Some(Instant::now());
    }

    pub fn record_verify(&mut self) {
        self.t_verify = Some(Instant::now());
    }

    pub fn record_inject_start(&mut self) {
        self.t_inject_start = Some(Instant::now());
    }

    pub fn record_inject_complete(&mut self) {
        self.t_inject_complete = Some(Instant::now());
    }

    /// Latency between hotkey detection and audio system readiness.
    pub fn hotkey_to_listening(&self) -> Option<Duration> {
        match (self.t_hotkey_detect, self.t_audio_ready) {
            (Some(start), Some(end)) if end >= start => Some(end - start),
            _ => None,
        }
    }

    /// Latency between speech detection and first useful partial transcript.
    pub fn speech_to_first_partial(&self) -> Option<Duration> {
        match (self.t_first_speech, self.t_first_partial) {
            (Some(start), Some(end)) if end >= start => Some(end - start),
            _ => None,
        }
    }

    /// Latency between hotkey release and ASR finalization.
    pub fn release_to_asr_final(&self) -> Option<Duration> {
        match (self.t_hotkey_release, self.t_asr_final) {
            (Some(start), Some(end)) if end >= start => Some(end - start),
            _ => None,
        }
    }

    /// Latency between ASR finalization and text injection complete.
    pub fn asr_final_to_injection(&self) -> Option<Duration> {
        match (self.t_asr_final, self.t_inject_complete) {
            (Some(start), Some(end)) if end >= start => Some(end - start),
            _ => None,
        }
    }

    /// Total session duration from hotkey release to final visible text injection.
    pub fn release_to_injection(&self) -> Option<Duration> {
        match (self.t_hotkey_release, self.t_inject_complete) {
            (Some(start), Some(end)) if end >= start => Some(end - start),
            _ => None,
        }
    }

    /// Overall turnaround time from hotkey detect to injection complete.
    pub fn total_turnaround(&self) -> Option<Duration> {
        match (self.t_hotkey_detect, self.t_inject_complete) {
            (Some(start), Some(end)) if end >= start => Some(end - start),
            _ => None,
        }
    }
}

/// Thread-safe local structural reliability metrics (§36).
#[derive(Debug, Default)]
pub struct ReliabilityCounters {
    pub sessions_started: AtomicU64,
    pub sessions_completed: AtomicU64,
    pub sessions_recovered: AtomicU64,
    pub asr_worker_crashes: AtomicU64,
    pub refine_worker_crashes: AtomicU64,
    pub llm_timeouts: AtomicU64,
    pub verifier_rejections: AtomicU64,
    pub injection_failures: AtomicU64,
    pub audio_overruns: AtomicU64,
    pub backend_fallbacks: AtomicU64,
}

impl ReliabilityCounters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc_started(&self) {
        self.sessions_started.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_completed(&self) {
        self.sessions_completed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_recovered(&self) {
        self.sessions_recovered.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_asr_crash(&self) {
        self.asr_worker_crashes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_refine_crash(&self) {
        self.refine_worker_crashes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_llm_timeout(&self) {
        self.llm_timeouts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_verifier_rejection(&self) {
        self.verifier_rejections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_injection_failure(&self) {
        self.injection_failures.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_audio_overrun(&self) {
        self.audio_overruns.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_backend_fallback(&self) {
        self.backend_fallbacks.fetch_add(1, Ordering::Relaxed);
    }

    /// Snapshot of current counter values for diagnostic inspection.
    pub fn snapshot(&self) -> ReliabilitySnapshot {
        ReliabilitySnapshot {
            sessions_started: self.sessions_started.load(Ordering::Relaxed),
            sessions_completed: self.sessions_completed.load(Ordering::Relaxed),
            sessions_recovered: self.sessions_recovered.load(Ordering::Relaxed),
            asr_worker_crashes: self.asr_worker_crashes.load(Ordering::Relaxed),
            refine_worker_crashes: self.refine_worker_crashes.load(Ordering::Relaxed),
            llm_timeouts: self.llm_timeouts.load(Ordering::Relaxed),
            verifier_rejections: self.verifier_rejections.load(Ordering::Relaxed),
            injection_failures: self.injection_failures.load(Ordering::Relaxed),
            audio_overruns: self.audio_overruns.load(Ordering::Relaxed),
            backend_fallbacks: self.backend_fallbacks.load(Ordering::Relaxed),
        }
    }
}

/// Point-in-time snapshot of reliability counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReliabilitySnapshot {
    pub sessions_started: u64,
    pub sessions_completed: u64,
    pub sessions_recovered: u64,
    pub asr_worker_crashes: u64,
    pub refine_worker_crashes: u64,
    pub llm_timeouts: u64,
    pub verifier_rejections: u64,
    pub injection_failures: u64,
    pub audio_overruns: u64,
    pub backend_fallbacks: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_tracker_calculations() {
        let mut tracker = SessionLatencyTracker::new();
        let t0 = Instant::now();
        tracker.t_hotkey_detect = Some(t0);
        tracker.t_audio_ready = Some(t0 + Duration::from_millis(50));
        tracker.t_first_speech = Some(t0 + Duration::from_millis(100));
        tracker.t_first_partial = Some(t0 + Duration::from_millis(350));
        tracker.t_hotkey_release = Some(t0 + Duration::from_millis(1500));
        tracker.t_asr_final = Some(t0 + Duration::from_millis(1650));
        tracker.t_inject_complete = Some(t0 + Duration::from_millis(1750));

        assert_eq!(
            tracker.hotkey_to_listening(),
            Some(Duration::from_millis(50))
        );
        assert_eq!(
            tracker.speech_to_first_partial(),
            Some(Duration::from_millis(250))
        );
        assert_eq!(
            tracker.release_to_asr_final(),
            Some(Duration::from_millis(150))
        );
        assert_eq!(
            tracker.asr_final_to_injection(),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            tracker.release_to_injection(),
            Some(Duration::from_millis(250))
        );
        assert_eq!(
            tracker.total_turnaround(),
            Some(Duration::from_millis(1750))
        );
    }

    #[test]
    fn test_reliability_counters_increment() {
        let counters = ReliabilityCounters::new();
        counters.inc_started();
        counters.inc_started();
        counters.inc_completed();
        counters.inc_recovered();
        counters.inc_verifier_rejection();

        let snap = counters.snapshot();
        assert_eq!(snap.sessions_started, 2);
        assert_eq!(snap.sessions_completed, 1);
        assert_eq!(snap.sessions_recovered, 1);
        assert_eq!(snap.verifier_rejections, 1);
        assert_eq!(snap.asr_worker_crashes, 0);
    }
}
