# FlowDictate Codec Security

## Principle: Minimal Codec Surface

Every additional parser and codec increases attack surface. Use the smallest codec surface necessary.

---

## Live Pipeline Isolation

The live inference path uses **ONLY** normalized PCM:

```
Microphone → native PCM → mono → resample 16kHz → f32 PCM → VAD → ASR
```

**No compressed audio encoding/decoding occurs in the live pipeline.**
**No container parsing occurs in the live pipeline.**
**No disk I/O occurs in the live pipeline.**

External audio codecs are isolated from the trusted live path.

---

## Supported Codecs

| Codec | Library | License | Use Case | Live Pipeline? |
|-------|---------|---------|----------|----------------|
| **WAV/PCM** | `hound` (pure Rust) | Apache-2.0 | Test fixtures, debugging, diagnostic exports | **No** |
| **FLAC** | `claxon` (pure Rust) | Apache-2.0 | Optional lossless training data, test fixtures | **No** |
| **Opus** | `audiopus` (C FFI) | ISC | Optional compressed recording (feature-gated) | **No** |

### Explicitly Not Supported
- MP3 — unnecessary, patent-encumbered history
- AAC — proprietary codec, unnecessary
- FFmpeg — massive attack surface, not needed
- Video codecs — completely out of scope
- General multimedia frameworks — excessive surface

---

## Validation Requirements

Before decoding ANY external audio file, validate:

| Check | Limit | Rationale |
|-------|-------|-----------|
| File size | ≤ 2 GB | Prevent excessive memory allocation |
| Container magic bytes | Must match expected format | Reject misidentified files |
| Channel count | ≤ 16 | Reject absurd values |
| Sample rate | ≤ 384,000 Hz | Reject absurd values |
| Duration | ≤ 24 hours (configurable) | Prevent excessive processing |
| Decompression ratio | ≤ 100:1 | Detect decompression bombs |
| Integer arithmetic | Checked multiplication | Prevent overflow in size calculations |

---

## Rejection Policy

**Reject immediately** (do not attempt to process):
- Unsupported container formats
- Absurd sample rates (> 384 kHz)
- Oversized channel counts (> 16)
- Malformed container headers
- Files exceeding size limit
- Decompressed output exceeding ratio limit

All rejections produce a typed error — never a panic or undefined behavior.

---

## Decompression Bounds

A tiny compressed input must NOT trigger uncontrolled memory allocation.

**Implementation**: Track bytes written during decompression. If output exceeds `input_size × max_ratio`, abort decoding and return an error.

---

## Fuzzing Targets

| Target | Priority | Input |
|--------|----------|-------|
| WAV header parser | **High** | Random bytes, truncated files, extreme values |
| FLAC frame decoder | **Medium** | Malformed FLAC streams, corrupted frames |
| Opus packet decoder | **Medium** | Invalid Opus packets, extreme durations |
| Container type detection | **High** | Random bytes, format confusion attacks |

No fuzz target may send data to an external service.
