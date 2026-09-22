# FlowDictate Security Testing

## Test Strategy Overview

Security testing is continuous. Every CI run executes security regression tests. No test may require live microphone hardware or network access.

---

## Unit Tests (per crate)

### flowdictate-security
- Model manifest parsing: valid JSON, invalid JSON, missing fields
- SHA-256 hash computation: known input → known output
- Path validation: traversal detection, symlink detection, null bytes
- `SensitiveBuffer` zeroization on drop
- `SensitiveString` has no `Display`/`Debug` implementation

### flowdictate-audio
- Ring buffer: bounded write, overflow drops oldest, read after write
- Ring buffer: sustained input → memory stays constant
- Resampling: 48kHz → 16kHz with known input/output pairs
- PCM normalization: i16/i32/f32 conversion correctness
- Channel downmix: stereo → mono preserves amplitude
- VAD state machine: all transitions, timeout enforcement
- Codec validation: reject oversized files, absurd parameters

### flowdictate-asr
- Consensus commit: stable text after N passes
- Consensus: unstable text not committed prematurely
- Rolling window: bounded size enforcement

### flowdictate-refine
- Deterministic cleanup: each rule has test cases
- Filler word removal: "um", "uh", "like", "you know"
- Capitalization after sentence boundaries
- Spoken command replacement: "period" → "."
- Output sanitization: length bounds, control characters, null bytes
- Dictionary matching and replacement

### flowdictate-inject
- Context permission enforcement: Level 0 accesses nothing

### flowdictate-storage
- Database encryption round-trip: write → close → reopen with key → read
- Credential store: write key → read key → delete key
- History modes: OFF creates no persistent transcripts

---

## Integration Tests

### Full Pipeline (synthetic audio)
- Generate synthetic audio (tone, silence, known speech WAV)
- Feed through capture → VAD → ASR → refine → sanitize
- Verify output text is reasonable
- Verify no crashes or hangs

### Model Integrity
- Valid model hash → loads successfully
- Invalid model hash → rejected with error
- Missing model file → graceful error
- Corrupted model (byte flip) → rejected

### Database Lifecycle
- Create encrypted DB → write data → close → reopen → read data
- History OFF → verify no transcript rows in DB
- Session-only → verify cleanup on graceful exit
- Key rotation: re-encrypt with new key

---

## Security Regression Tests

These tests are automated and run in CI:

| # | Test | Description | Pass Criteria |
|---|------|-------------|---------------|
| 1 | Model hash verification | Load model with valid manifest hash | Model loads |
| 2 | Unknown model rejection | Attempt to load model with unknown hash | Load refused, error returned |
| 3 | Corrupted model rejection | Flip bytes in a model file, attempt to load | Load refused, error returned |
| 4 | Ring buffer overflow | Sustain maximum-rate input for 60s | Memory usage stays within 2× ring buffer size |
| 5 | Excessive duration | Simulate 600s recording (exceeds 300s max) | Session auto-segments or terminates |
| 6 | Malformed WAV | Truncated header, zero-length data, absurd channel count | Graceful error, no crash |
| 7 | Malformed FLAC | Corrupted frame headers | Graceful error, no crash |
| 8 | Malformed Opus | Invalid packet data | Graceful error, no crash |
| 9 | Decompression bounds | Tiny file claiming massive decompressed size | Rejected before excessive allocation |
| 10 | DB encryption | Attempt to read DB without key | Read fails |
| 11 | Credential store failure | Simulate keyring unavailable | Graceful degradation, user notified |
| 12 | Transcript logging | Inject marker string, scan logs | Marker not found in any log output |
| 13 | Prompt injection | "Ignore previous instructions" in transcript | Treated as content, system behavior unchanged |
| 14 | Context permissions | Level 0: attempt to read app context | No context accessed |
| 15 | Path traversal | `../../../etc/passwd` in model path | Path rejected |
| 16 | Profile import | Profile with traversal paths, oversized, invalid schema | All rejected |
| 17 | Crash recovery cleanup | Simulate crash → restart → verify cleanup | Recovery data processed and deleted |
| 18 | Temp file detection | Run pipeline, scan temp directory | No sensitive temp files created |
| 19 | Text sanitization | Input with control characters, excessive length | Sanitized or rejected |
| 20 | Deterministic fallback | Simulate LLM failure | Pipeline falls back to deterministic cleanup |
| 21 | Model failure fallback | Simulate ASR failure | Pipeline returns graceful error |

---

## Privacy Regression Test

### Marker String Test

1. Create unique marker strings:
   - `MARKER_TRANSCRIPT_7f3a2b` (simulating transcript text)
   - `MARKER_USERNAME_9c4d1e` (simulating username)
   - `MARKER_CONTEXT_5b8e3f` (simulating app context)
   - `MARKER_DICTIONARY_2a7c9d` (simulating dictionary entry)

2. Inject markers through the complete pipeline

3. After pipeline execution, scan:
   - All log files
   - Temporary directories
   - Crash recovery files
   - Non-encrypted application storage
   - Standard output / standard error

4. **Pass criteria**: Markers must NOT appear outside explicitly permitted encrypted storage.

---

## Network Isolation Test

### Procedure
1. Block all outbound network access (firewall rule or network namespace)
2. Start FlowDictate
3. Load model
4. Capture audio (synthetic)
5. Run VAD
6. Run ASR
7. Run refinement
8. Run text injection
9. Access personalization
10. Access history

### Pass Criteria
All operations complete successfully. No DNS resolution attempts. No outbound socket connections.

### Automated Variant
Where practical, use `strace`/`dtrace` or equivalent to verify the process does not call `connect()`, `sendto()`, or DNS resolution functions.

---

## Fuzz Targets

| Target | Priority | Input Space | Tool |
|--------|----------|-------------|------|
| WAV parser (`hound`) | **1** | Random bytes, truncated files | `cargo-fuzz` |
| Profile import parser | **2** | Malformed JSON, oversized, traversal paths | `cargo-fuzz` |
| Codec container detection | **3** | Random bytes, format confusion | `cargo-fuzz` |
| Model manifest parser | **4** | Malformed JSON, missing fields, extreme values | `cargo-fuzz` |
| Transcript sanitization | **5** | Unicode edge cases, control characters, long strings | `cargo-fuzz` |
| Consensus commit engine | **6** | Rapidly changing hypothesis sequences | `cargo-fuzz` |
| Text injection boundary | **7** | Special characters, escape sequences, long strings | `cargo-fuzz` |

No fuzz target may send data to an external service.

---

## CI Pipeline

```yaml
Triggers: push/PR to main
Matrix: ubuntu-latest, windows-latest, macos-latest

Steps:
  1. cargo fmt --check
  2. cargo clippy -- -D warnings
  3. cargo test --workspace
  4. cargo audit
  5. Security regression tests (subset suitable for CI)
```

All tests use synthetic audio fixtures — no live microphone required.
