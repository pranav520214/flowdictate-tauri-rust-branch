# FlowDictate Model Security

## Threat

Models are **executable-adjacent untrusted input**. Model file formats are complex binary structures parsed by C/C++ code (whisper.cpp, llama.cpp). Vulnerabilities in these parsers could lead to arbitrary code execution.

---

## Integrity Gate

Every model file must pass integrity verification **before** loading.

```
Model discovered
       │
       ▼
┌──────────────┐     FAIL
│ Validate     │──────────────→ REJECT (path traversal)
│ Path         │
└──────┬───────┘
       │ OK
       ▼
┌──────────────┐     FAIL
│ Validate     │──────────────→ REJECT (size mismatch)
│ File Size    │
└──────┬───────┘
       │ OK
       ▼
┌──────────────┐     FAIL
│ Calculate    │──────────────→ REJECT (hash mismatch)
│ SHA-256 Hash │
└──────┬───────┘
       │ OK
       ▼
┌──────────────┐     FAIL
│ Compare      │──────────────→ REJECT (unknown model)
│ Manifest     │
└──────┬───────┘
       │ OK
       ▼
   LOAD MODEL
```

**If ANY check fails → DO NOT LOAD. Fail closed.**

There is no "show warning and continue" path. Unknown hashes fail closed.

---

## Manifest Format

```json
{
  "version": 1,
  "models": [
    {
      "id": "whisper-base-q5_1",
      "architecture": "whisper",
      "variant": "base",
      "multilingual": true,
      "quantization": "q5_1",
      "parameters": "74M",
      "sha256": "<64-character hex digest>",
      "file_size_bytes": 59736064,
      "filename": "ggml-base.bin",
      "runtime": "whisper.cpp",
      "runtime_version_min": "1.5.0"
    }
  ]
}
```

The manifest may be:
1. **Compiled into the binary** (highest integrity — cannot be modified without rebuilding)
2. **Loaded from a signed location** (allows user-added models with explicit hash registration)

---

## Plugin Prohibition

Model loading **MUST NEVER** accept:
- Arbitrary executable plugins
- Dynamic libraries loaded alongside models
- Script files interpreted during model loading
- Custom operator definitions from model files

Only plain model weight formats supported directly by whisper.cpp (GGML) and llama.cpp (GGUF) are permitted.

---

## User-Added Models

Users may add models by:
1. Placing model files in the `models/` directory
2. Computing the SHA-256 hash of the file
3. Adding the hash to the user section of the manifest

Without step 3, the model will be **rejected** by the integrity gate.

---

## Model Failure Handling

All model failures must be handled gracefully — never crash dictation.

| Failure | Handling |
|---------|----------|
| Out of memory | Release model, fall back to smaller model or deterministic-only |
| Context exhaustion | Truncate input, warn user |
| Corrupted model | Reject at integrity gate |
| Initialization failure | Log error (no sensitive data), notify user |
| Decoder failure | Fall back to deterministic refinement |
| Timeout | Abort inference, return partial result |
| Invalid token output | Discard output, use raw transcript |

### Fallback Hierarchy
```
LLM refinement fails
       │
       ▼
Deterministic refinement
       │ fails
       ▼
Sanitized raw transcript (always available)
```

**Never fall back to a cloud service.**
