# FlowDictate

**Privacy-First Local Voice Computing**

> Press hotkey → speak → see live transcription → release → text appears at cursor.

FlowDictate is a free, local-first, offline-first voice-input system that processes all speech entirely on your machine. No cloud APIs, no telemetry, no subscriptions.

## Privacy Guarantees

- **Zero cloud inference** — all processing happens locally
- **Zero telemetry** — no data collection of any kind
- **Zero network requirement** — works completely offline
- **Encrypted storage** — all persistent data uses SQLCipher with OS-protected keys
- **Transparent controls** — see exactly what is stored and accessed

## Current Status

**Milestone 0 — Architecture & Threat Model** (in progress)

See [`docs/`](docs/) for architecture documentation:

- [`ARCHITECTURE.md`](docs/ARCHITECTURE.md) — System architecture & module map
- [`THREAT_MODEL.md`](docs/THREAT_MODEL.md) — Security threat analysis
- [`PRIVACY_MODEL.md`](docs/PRIVACY_MODEL.md) — Privacy architecture & data flows
- [`AUDIO_PIPELINE.md`](docs/AUDIO_PIPELINE.md) — Audio capture pipeline
- [`CODEC_SECURITY.md`](docs/CODEC_SECURITY.md) — Codec & container security
- [`MODEL_SECURITY.md`](docs/MODEL_SECURITY.md) — Model supply-chain integrity
- [`MODEL_BENCHMARKS.md`](docs/MODEL_BENCHMARKS.md) — Benchmark methodology
- [`SECURITY_TESTING.md`](docs/SECURITY_TESTING.md) — Test strategy
- [`PERFORMANCE.md`](docs/PERFORMANCE.md) — Performance targets & budget

## Architecture

```
flowdictate-core          ← Pipeline orchestrator
├── flowdictate-audio     ← Microphone capture, ring buffer, VAD
├── flowdictate-asr       ← Whisper.cpp streaming ASR
├── flowdictate-refine    ← Deterministic cleanup + optional local LLM
├── flowdictate-inject    ← Hotkey, text injection, app context
├── flowdictate-storage   ← SQLCipher DB, OS keyring
└── flowdictate-security  ← Model integrity, memory safety, path validation
```

## License

MIT OR Apache-2.0
