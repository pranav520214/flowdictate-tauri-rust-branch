# FlowDictate Model Benchmarks

## Purpose

Every model size decision must be justified by **measured benchmarks**, not assumptions. Do not report performance targets as benchmark results.

---

## ASR Benchmark Protocol

### Test Corpus
Curated audio samples per language:
- **English**: 10+ samples covering conversational, technical, and dictation styles
- **Hindi**: 5+ samples covering formal and informal speech
- **Hinglish**: 5+ samples with Hindi-English code-switching
- **Other Indic languages**: 3+ samples each (Punjabi, Bengali, Tamil, Telugu, Urdu)

All samples must have ground-truth transcriptions for WER calculation.

### Metrics
| Metric | Description |
|--------|-------------|
| **WER** | Word Error Rate (substitutions + deletions + insertions) / reference words |
| **RTF** | Real-Time Factor = processing time / audio duration. RTF < 1.0 = faster than real-time |
| **Peak RAM** | Maximum RSS during inference |
| **Steady-state RAM** | RSS after model is loaded and idle |
| **CPU usage** | Average CPU % during inference |

### Hardware Documentation
Every benchmark must record:
- CPU model and generation
- Core/thread count
- Total system RAM
- OS version
- whisper.cpp / llama.cpp version

### ASR Models to Test

| Model | Params | Quantization Variants |
|-------|--------|----------------------|
| whisper-tiny | 39M | Q4_0, Q4_1, Q5_0, Q5_1, Q8_0 |
| whisper-base | 74M | Q4_0, Q4_1, Q5_0, Q5_1, Q8_0 |
| whisper-small | 244M | Q5_0, Q5_1, Q8_0 |
| large-v3-turbo | 809M | Q5_1, Q8_0 |

### Results Template (to be filled during Milestone 10)

| Model | Quant | Language | WER | RTF | Peak RAM | CPU% | Hardware |
|-------|-------|----------|-----|-----|----------|------|----------|
| | | | | | | | |

---

## Refinement Model Benchmark Protocol

### Test Corpus
Raw transcripts with known correct refinements:
- 50+ examples covering: filler deletion, repetitions, false starts, self-corrections
- 20+ examples with programming terminology
- 10+ examples with Indian English patterns
- 10+ examples with code-switching

### Metrics
| Metric | Description |
|--------|-------------|
| **Edit accuracy** | % of corrections applied correctly |
| **False insertions** | Information added that wasn't in the original |
| **Meaning preservation** | Does refined text preserve the speaker's intent? |
| **Latency** | Time to refine one transcript segment |
| **RAM** | Memory usage during refinement |

### Models to Test

| Model | Params | Quant | Size |
|-------|--------|-------|------|
| SmolLM2-360M-Instruct | 360M | Q4_K_M | ~240 MB |
| Qwen2.5-0.5B-Instruct | 490M | Q4_K_M | ~398 MB |
| SmolLM2-1.7B-Instruct | 1.7B | Q4_K_M | ~1.05 GB |

### Comparison: Deterministic-Only vs Deterministic+LLM
Measure the incremental quality improvement of adding LLM refinement to justify its memory/latency cost.

---

## Model Escalation Policy

```
deterministic rules only
        │
        │ insufficient quality?
        ▼
tiny ASR + tiny refinement model
        │
        │ insufficient quality?
        ▼
quantization tuning (try Q8_0 instead of Q5_1)
        │
        │ insufficient quality?
        ▼
prompt optimization
        │
        │ insufficient quality?
        ▼
domain fine-tuning (LoRA/QLoRA)
        │
        │ insufficient quality?
        ▼
slightly larger local model
```

Every step up must be justified by benchmark delta.

---

## Fine-Tuning Strategy

If the smallest general-purpose model is insufficient, prefer **targeted fine-tuning** over increasing model size.

### Task Definition
The refinement model has a narrow task:
```
INPUT:  raw spoken transcript
OUTPUT: clean written transcript
```

It does NOT need to be a general chatbot.

### Training Format
```
INPUT:
uh so we should probably deploy this tomorrow actually no friday morning

OUTPUT:
We should probably deploy this Friday morning.
```

### Training Data Categories
- Filler word deletion ("um", "uh", "like", "you know")
- Repetition removal ("the the", "I I think")
- False start correction ("go to the— open the file")
- Self-correction ("Tuesday, no, Wednesday")
- Punctuation and capitalization
- Bullet/list formatting ("first... second... third...")
- Programming terminology ("camelCase", "API", "JSON")
- Indian English patterns
- Hindi-English code-switching
- Names and organization names
- Technical vocabulary (medical, legal, scientific)
- Informal messaging style
- Professional/formal messaging style

### Requirements
- Fine-tuning must be possible **locally** (no cloud training services)
- Prefer parameter-efficient methods: LoRA, QLoRA
- **Never upload user correction history** to any training service
- Training data stays encrypted locally
