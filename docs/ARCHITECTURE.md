# FlowDictate Architecture

## System Overview

FlowDictate is a **local voice-computing input layer** that processes all speech entirely on the user's machine.

**Primary interaction flow:**
```
Hotkey Press → Listening → Live Partial Transcription → Release/Pause
    → Deterministic Cleanup → [Optional LLM Refinement] → Text at Cursor
```

**Non-negotiable constraints:**
- Zero cloud inference APIs
- Zero telemetry or analytics
- Zero network requirement for normal operation
- All data stays on the user's machine

---

## Workspace Structure

```
flowdictate/
├── crates/
│   ├── flowdictate-core/       ← Pipeline orchestrator
│   ├── flowdictate-audio/      ← Microphone, ring buffer, VAD
│   ├── flowdictate-asr/        ← Whisper.cpp streaming ASR
│   ├── flowdictate-refine/     ← Deterministic + optional LLM cleanup
│   ├── flowdictate-inject/     ← Hotkey, text injection, app context
│   ├── flowdictate-storage/    ← SQLCipher DB, OS keyring
│   └── flowdictate-security/   ← Integrity, zeroize, path validation
├── src-tauri/                  ← Tauri v2 application shell
├── ui/                         ← Vanilla HTML/CSS/JS overlay + settings
├── models/                     ← Model files + integrity manifest
├── tests/                      ← Integration, security, fuzz tests
└── docs/                       ← Architecture documentation
```

---

## Dependency Graph

```
                    ┌──────────────────┐
                    │ flowdictate-core │
                    │  (orchestrator)  │
                    └───────┬──────────┘
          ┌─────────────────┼─────────────────────┐
          │                 │                     │
          ▼                 ▼                     ▼
┌─────────────────┐ ┌──────────────┐ ┌────────────────────┐
│flowdictate-audio│ │flowdictate-  │ │flowdictate-inject  │
│ cpal, ringbuf,  │ │    asr       │ │ enigo, global-     │
│ rubato, vad     │ │ whisper-rs   │ │ hotkey             │
└────────┬────────┘ └──────┬───────┘ └────────────────────┘
         │                 │
         │    ┌────────────┤
         │    │            │
         ▼    ▼            ▼
┌──────────────────┐ ┌──────────────────┐
│flowdictate-refine│ │flowdictate-      │
│ [llama-cpp-2]    │ │    storage       │
│ (optional)       │ │ rusqlite+cipher  │
└────────┬─────────┘ │ keyring          │
         │           └────────┬─────────┘
         │                    │
         ▼                    ▼
    ┌──────────────────────────────┐
    │    flowdictate-security      │
    │  sha2, zeroize, hex          │
    │  (no external dependencies   │
    │   that process user data)    │
    └──────────────────────────────┘
```

Each leaf crate is independently compilable and testable.

---

## Data Flow

```
┌───────────┐    SPSC ring     ┌────────────┐   crossbeam    ┌───────────┐
│ Microphone│──── buffer ──────│ Processor  │──── chan ──────│  ASR      │
│  (cpal)   │   (lock-free)   │  Thread    │               │  Thread   │
│           │                  │            │               │(whisper)  │
│ • native  │                  │ • mono     │               │           │
│   PCM     │                  │ • resample │               │ • rolling │
│ • 48kHz   │                  │   → 16kHz  │               │   window  │
│ • zero-   │                  │ • norm f32 │               │ • partial │
│   alloc   │                  │ • VAD      │               │   hypo.   │
└───────────┘                  └────────────┘               │ • commit  │
                                                            └─────┬─────┘
                                                                  │
                                          crossbeam channel       │
                                    ┌─────────────────────────────┘
                                    ▼
                              ┌───────────┐   crossbeam    ┌───────────┐
                              │ Refinement│──── chan ──────│  Inject   │
                              │  Thread   │               │           │
                              │           │               │ • enigo / │
                              │ • Stage A │               │   native  │
                              │   (rules) │               │ • sanitize│
                              │ • Stage B │               │   first   │
                              │   (LLM,   │               └───────────┘
                              │   opt.)   │
                              └───────────┘
```

---

## Thread Architecture

| Thread | Responsibility | Communication | Blocking? |
|--------|---------------|---------------|-----------|
| **Audio Callback** (OS-managed) | Capture PCM samples | Write to ring buffer (lock-free) | **Never** — zero-alloc, zero-IO |
| **Audio Processor** | Resample, normalize, VAD | Read ring buffer → write crossbeam channel | May block on channel send (bounded) |
| **ASR Inference** | Whisper.cpp streaming | Read crossbeam → write crossbeam | CPU-bound (blocking C++ FFI) |
| **Refinement** | Deterministic cleanup + optional LLM | Read crossbeam → write crossbeam | CPU-bound (optional LLM FFI) |
| **Main/UI** (Tauri) | Event loop, IPC, window management | Tauri events, state queries | Non-blocking (Tauri runtime) |

---

## Performance Modes

| Mode | ASR Model | Refinement | Memory | Unload Policy |
|------|-----------|------------|--------|---------------|
| **Eco** | whisper-tiny Q5_1 | Deterministic only | ~1.4 GB peak | Unload after 60s idle |
| **Balanced** | whisper-base Q5_1 | Deterministic + LLM warm | ~2.3 GB peak | Keep warm when memory permits |
| **Quality** | whisper-small+ (user-installed) | Deterministic + larger LLM | ~3+ GB peak | Always loaded |

---

## Security Architecture

See:
- [THREAT_MODEL.md](THREAT_MODEL.md) — Complete threat classification
- [PRIVACY_MODEL.md](PRIVACY_MODEL.md) — Data flow and privacy controls
- [MODEL_SECURITY.md](MODEL_SECURITY.md) — Model supply-chain integrity
- [CODEC_SECURITY.md](CODEC_SECURITY.md) — Audio codec security
- [SECURITY_TESTING.md](SECURITY_TESTING.md) — Test strategy

---

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| No async runtime | Dedicated threads + crossbeam | Real-time audio + blocking C++ FFI = poor async fit |
| No frontend framework | Vanilla HTML/CSS/JS | Overlay is ~50 lines of UI |
| Workspace crates | 7 library crates | Clear trust boundaries, independent testing |
| Feature-gated LLM | `llm` feature flag on refine crate | LLM dependency is optional, not default |
| No HTTP client | Excluded entirely from core | Zero-cloud requirement is non-negotiable |
| Sequential model use | ASR → release → LLM | Reduces peak memory from ~1 GB to ~750 MB |
