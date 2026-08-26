# FlowDictate Performance

## Target Hardware

| Spec | Minimum |
|------|---------|
| CPU | Dual-core (any recent x86_64 or ARM64) |
| GPU | Integrated graphics (not required for inference) |
| RAM | 4 GB total system |
| Storage | ~500 MB for application + models |
| GPU/NPU | Not required — entirely CPU-capable |

Accelerators (CUDA, Metal, Vulkan) may be detected and used when available but are **never** required.

---

## RAM Budget

### Eco Mode (whisper-tiny, deterministic-only refinement)

| Component | RAM |
|-----------|-----|
| OS + Desktop Environment | ~1,000 MB |
| Tauri shell + WebView | ~150 MB |
| whisper-tiny Q5_1 model | ~250 MB |
| Audio buffers + VAD | ~100 MB |
| SQLCipher + app overhead | ~50 MB |
| **Total** | **~1,550 MB** |
| **Headroom on 4 GB** | **~2,450 MB** |

### Balanced Mode (whisper-base, sequential pipeline with optional LLM)

| Component | RAM |
|-----------|-----|
| OS + Desktop Environment | ~1,000 MB |
| Tauri shell + WebView | ~150 MB |
| whisper-base Q5_1 (active during ASR) | ~375 MB |
| Audio buffers + VAD | ~100 MB |
| *-- ASR completes, audio buffers released --* | |
| Qwen2.5-0.5B Q4_K_M (if needed) | ~630 MB |
| SQLCipher + app overhead | ~50 MB |
| **Peak (with LLM)** | **~2,305 MB** |
| **Peak (deterministic only)** | **~1,675 MB** |
| **Headroom on 4 GB** | **~1,700 – 2,325 MB** |

### Quality Mode (whisper-small, concurrent models)

| Component | RAM |
|-----------|-----|
| OS + Desktop Environment | ~1,000 MB |
| Tauri shell + WebView | ~150 MB |
| whisper-small Q5_1 | ~850 MB |
| Audio buffers + VAD | ~100 MB |
| SmolLM2-1.7B Q4_K_M | ~1,050 MB |
| SQLCipher + app overhead | ~50 MB |
| **Total** | **~3,200 MB** |
| **Requires** | **≥ 6 GB system RAM** |

---

## Latency Budget

| Stage | Target (ms) | Notes |
|-------|-------------|-------|
| Microphone → ring buffer | < 5 | Lock-free SPSC write |
| Ring buffer → processor | < 10 | SPSC read |
| Channel conversion + resample | < 20 | Per 30ms frame, rubato sinc |
| VAD decision | < 1 | WebRTC VAD = ~µs |
| ASR inference (whisper-base, 5s chunk) | < 1,000 | RTF ~0.15 on modern CPU |
| Deterministic cleanup | < 5 | String operations |
| LLM refinement (Qwen 0.5B, ~100 tokens) | < 2,000 | CPU inference |
| Output sanitization | < 1 | Bounds checking |
| Text injection (enigo) | < 50 | OS input API latency |
| **Total (no LLM)** | **< 1,100** | |
| **Total (with LLM)** | **< 3,100** | |

**Key UX metric**: End of speech → final text visible. Optimize this path aggressively.

---

## Performance Modes

| Mode | ASR Model | Refinement | Model Lifecycle |
|------|-----------|------------|-----------------|
| **Eco** | whisper-tiny | Deterministic only | Unload after 60s idle |
| **Balanced** | whisper-base | Deterministic + LLM warm | Keep warm when memory permits |
| **Quality** | whisper-small+ | Deterministic + larger LLM | Always loaded |

---

## Measurement Methodology

For each benchmark, report:

| Metric | What |
|--------|------|
| **Median** | 50th percentile latency |
| **p95** | 95th percentile latency |
| **Peak memory** | Maximum RSS during test |
| **Steady-state memory** | RSS after initialization, at idle |
| **CPU usage** | Average CPU % during active inference |
| **Real-time factor** | Processing time / audio duration |

> [!IMPORTANT]
> Do not report targets as measurements. Actual benchmarks will be conducted during Milestone 10 on representative hardware and recorded in this document.

---

## Model Warm/Cold Behavior

### Cold Start
| Model | Expected Load Time (CPU) |
|-------|--------------------------|
| whisper-tiny Q5_1 (~31 MB) | < 500 ms |
| whisper-base Q5_1 (~57 MB) | < 1,000 ms |
| Qwen2.5-0.5B Q4_K_M (~398 MB) | < 3,000 ms |

### Warm
Model kept in memory — inference begins immediately upon receiving audio.

### Eco Mode Unload
After `unload_timeout` (default 60s) of inactivity:
1. Release model memory
2. Log non-sensitive unload event
3. On next hotkey press, reload (cold start latency applies)

---

## Actual Measurements

> **Status**: To be filled during Milestone 10.
>
> This section will contain real benchmark results measured on:
> - Target minimum hardware (dual-core, 4 GB RAM)
> - Development hardware (for comparison)
> - Multiple operating systems
