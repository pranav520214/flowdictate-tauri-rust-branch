# FlowDictate Audio Pipeline

## Pipeline Overview

```
┌──────────────┐    lock-free     ┌──────────────┐    crossbeam    ┌──────────────┐
│  Microphone  │──── SPSC ring ──│  Processor   │──── channel ───│  ASR Engine  │
│  Capture     │    buffer        │  Thread      │                │              │
│  (cpal)      │    (ringbuf)     │              │                │ (whisper.cpp)│
│              │                  │ 1. Channel   │                │              │
│ • native PCM │                  │    convert   │                │ • rolling    │
│ • 44.1/48kHz │                  │ 2. Resample  │                │   window     │
│ • callback:  │                  │    → 16kHz   │                │ • partial    │
│   zero-alloc │                  │ 3. Normalize │                │   hypotheses │
│   zero-IO    │                  │    → f32     │                │ • consensus  │
│   zero-locks │                  │ 4. VAD       │                │   commit     │
└──────────────┘                  └──────────────┘                └──────────────┘
```

---

## Microphone Capture

### Platform Backend
- **Windows**: WASAPI (via cpal)
- **macOS**: CoreAudio (via cpal)
- **Linux**: ALSA or PulseAudio (via cpal)

### Device Enumeration
- List available input devices on startup
- Allow user to select preferred device
- Handle device hot-plug/disconnect gracefully
- Default to system default input device

### Native Formats
Microphones commonly produce:
- 44,100 Hz (CD standard)
- 48,000 Hz (professional audio / USB mics)
- Various bit depths: 16-bit, 24-bit, 32-bit float

### Callback Contract

The audio capture callback **MUST**:
- Copy samples into the preallocated ring buffer
- Return immediately

The audio capture callback **MUST NOT**:
- Perform heap allocation (`malloc`, `Vec::push`, `Box::new`)
- Perform disk I/O (`File::write`, database operations)
- Perform network operations
- Perform model inference
- Acquire blocking locks (`Mutex::lock`)
- Call `tracing` macros (they may allocate)

The callback's job is approximately:
```
capture → normalize sample format → copy to ring buffer → return
```

---

## Ring Buffer

### Design
- **Type**: SPSC (Single-Producer Single-Consumer) via `ringbuf`
- **Allocation**: Preallocated at startup, never grows
- **Producer**: Audio callback thread (writes captured samples)
- **Consumer**: Processor thread (reads samples)
- **Overflow policy**: Drop oldest frames — **NEVER** block the callback

### Size Calculation
```
Sample rate:   48,000 Hz
Channels:      1 (mono after conversion) or 2 (if stereo mic)
Sample size:   4 bytes (f32)
Duration:      5 seconds

Buffer size = 48,000 × 1 × 4 × 5 = 960,000 bytes ≈ 938 KB
```

5 seconds provides sufficient margin for the processor thread to consume data even during brief scheduling delays.

---

## Channel Conversion

- **Stereo → Mono**: Average left and right channels `(L + R) / 2`
- **Multi-channel → Mono**: Average all channels
- **Mono → Mono**: Passthrough

Amplitude is preserved during conversion.

---

## Resampling

### Library
`rubato` — Pure Rust sinc interpolation resampler.

### Parameters
- **Input**: native sample rate (44,100 Hz or 48,000 Hz)
- **Output**: 16,000 Hz (whisper.cpp requirement)
- **Quality**: Sinc interpolation with configurable quality
- **Location**: Processor thread — NOT in the audio callback

### Benchmark Targets
| Metric | Target |
|--------|--------|
| Latency per 30ms frame | < 20 ms |
| CPU usage | < 5% of one core |
| Allocations per frame | 0 (preallocated buffers) |

---

## PCM Normalization

Convert all sample formats to the canonical internal representation:

| Input Format | Conversion |
|-------------|------------|
| i16 | `sample as f32 / 32768.0` |
| i32 | `sample as f32 / 2147483648.0` |
| f32 | Clamp to [-1.0, 1.0] |

**Output**: f32 samples in [-1.0, 1.0] range, mono, 16 kHz.

---

## Voice Activity Detection

### Engine
WebRTC VAD via `webrtc-vad` — lightweight C-based detection with microsecond decision time.

### State Machine
```
                 ┌───────────────────────────────────────┐
                 │                                       │
                 ▼                                       │
          ┌──────────┐    speech     ┌──────────────┐    │
          │ Silence  │──────────────│ SpeechStart  │    │
          └──────────┘              └──────┬───────┘    │
                ▲                          │             │
                │                          ▼             │
                │                   ┌──────────────┐    │
                │                   │ SpeechCont.  │◄───┘
                │                   └──────┬───────┘
                │                          │
                │              silence < threshold
                │                          │
                │                          ▼
                │                   ┌──────────────┐
                │         speech    │ ShortPause   │
                │         ◄─────────┤              │
                │                   └──────┬───────┘
                │                          │
                │              silence > threshold
                │                          │
                │                          ▼
                │                   ┌──────────────┐
                └───────────────────│ Utterance    │
                                   │ Complete     │
                                   └──────────────┘
```

### Safety Invariants
- Failed VAD state **CANNOT** create unbounded audio buffers
- Maximum utterance duration enforced (default: 300 seconds)
- If VAD stays in SpeechContinue for > max duration, force segment boundary
- Buffer size is fixed — overflow drops oldest frames

### Configuration
| Parameter | Default | Description |
|-----------|---------|-------------|
| `silence_threshold_ms` | 1500 | Silence duration to end utterance |
| `short_pause_ms` | 300 | Silence duration for short pause (not end) |
| `max_utterance_s` | 300 | Maximum single utterance duration |
| `vad_aggressiveness` | 2 | WebRTC VAD mode (0=least, 3=most aggressive) |

---

## Streaming ASR Integration

### Rolling Window
- Bounded active decoding window (e.g., 30 seconds)
- New speech frames enter the window
- Old frames are discarded after processing
- Timestamps track which segments have been decoded

### Partial Hypotheses
- Whisper produces text hypotheses that may change as more audio arrives
- Partial text is sent to the UI overlay for live display
- Only consensus-stable text is committed to the output

### Consensus Commit
- Track hypothesis text over multiple inference passes
- Text stable across N consecutive passes → committed
- Committed text is never rewritten
- Unstable text displayed as provisional (grayed/italic in overlay)

---

## Finalization Triggers

An utterance is finalized when:

1. **Global hotkey released** — primary trigger
2. **Silence threshold reached** — configurable, default 1.5s
3. **Explicit stop command** — future voice command support
4. **Max session duration** — safety limit, default 300s

For long sessions, soft boundaries are created at natural pauses to prevent unbounded accumulation.

---

## Memory Safety Summary

| Invariant | Enforcement |
|-----------|-------------|
| Ring buffer bounded | Preallocated, overflow drops oldest |
| Audio callback allocation-free | No heap ops, no locks, no I/O |
| VAD cannot cause unbounded buffer | Max duration limit + fixed ring buffer |
| Audio data in volatile memory only | Never written to disk during normal operation |
| Buffers zeroized after use | `SensitiveBuffer` with `ZeroizeOnDrop` |
