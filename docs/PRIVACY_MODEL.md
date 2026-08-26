# FlowDictate Privacy Model

## Principles

1. **Data minimization** — Collect and retain the minimum necessary
2. **Local-only** — Nothing leaves the machine
3. **Transparent** — User can see exactly what is stored and accessed
4. **User-controlled** — Explicit permission for everything
5. **Deletable** — Clear path to delete all data

---

## Data Classification

| Data Type | Sensitivity | Storage Location | Lifetime | User Control |
|-----------|------------|------------------|----------|--------------|
| Live microphone audio | **Critical** | Volatile memory (ring buffer) | Duration of utterance | Automatic cleanup |
| Raw ASR transcript | **Critical** | Volatile memory | Until refinement complete | Automatic cleanup |
| Refined transcript | **High** | Volatile memory → encrypted DB (if history on) | Until injected (or per history mode) | History mode setting |
| User dictionary | **High** | SQLCipher encrypted DB | Until user deletes | Full CRUD + export/import |
| Correction history | **High** | SQLCipher encrypted DB | Until user deletes | Inspect/delete/disable/clear |
| DB encryption key | **Critical** | OS keyring only | Indefinite | OS authentication required |
| Model files | **Medium** | Filesystem (integrity-verified) | Until user removes | User manages model directory |
| Application config | **Low** | Config file or encrypted DB | Indefinite | Settings UI |
| Session crash recovery | **High** | SQLCipher encrypted DB (if enabled) | Until clean exit | History mode setting |
| Audit log | **Low** | SQLCipher encrypted DB | Until user deletes | Viewable in settings |
| Application context | **High** | Volatile memory (permission-gated) | Duration of refinement | Context level setting |

---

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                    DATA NEVER LEAVES THIS BOX                       │
│                                                                     │
│  Microphone ──→ [volatile memory only] ──→ Ring Buffer              │
│                                               │                     │
│                                               ▼                     │
│                                           Processor                 │
│                                           (resample, VAD)           │
│                                               │                     │
│                                               ▼                     │
│                                        ASR Engine                   │
│                                        [volatile memory]            │
│                                               │                     │
│                                               ▼                     │
│                                         Raw Transcript              │
│                                         [volatile memory]           │
│                                               │                     │
│                    ┌──────────────────────────┤                     │
│                    ▼                          ▼                     │
│              Deterministic              Optional LLM                │
│              Cleanup                    Refinement                   │
│                    │                          │                     │
│                    └──────────┬───────────────┘                     │
│                               ▼                                     │
│                         Refined Text                                │
│                         [volatile memory]                           │
│                               │                                     │
│                    ┌──────────┴──────────┐                          │
│                    ▼                     ▼                          │
│              Text Injection      Encrypted DB (if history on)       │
│              → Target App        [SQLCipher + OS keyring key]       │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────┐       │
│  │ NEVER WRITTEN TO:                                        │       │
│  │ • Logs (plaintext)      • Temp files                     │       │
│  │ • Clipboard (unauth.)   • Crash dumps (by design)        │       │
│  │ • Network (no client)   • Unencrypted DB                 │       │
│  └─────────────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Context Permission Ladder

| Level | Access | Improves | Privacy Cost | Default? |
|-------|--------|----------|--------------|----------|
| **0** | No external context | — | None | **Yes** |
| **1** | Current application identity (name, window title) | Domain-specific vocabulary | Minimal — app name only |  |
| **2** | Selected text or current textbox content | Contextual accuracy, continuity | Medium — exposes document content |  |
| **3** | Additional explicitly approved context | Deep contextual understanding | High — broader content exposure |  |

**Transparency requirement**: The current context level is always visible in the privacy dashboard. Changing the level requires explicit user action.

---

## History Modes

### OFF
- No transcripts persisted to any storage
- No crash recovery transcripts
- No hidden caches containing transcript text
- Audio never retained
- Maximum privacy, minimum convenience

### Session Only (Default)
- Encrypted crash recovery data during active session
- Recovery data deleted on clean application exit
- Provides resilience against crashes without long-term persistence
- If application crashes, recovery data persists until next clean launch

### Encrypted Persistent
- Full transcript history in SQLCipher database
- Encryption key in OS keyring
- User can browse, search, and delete history entries
- "Delete all local data" control available

---

## Privacy Dashboard

The settings UI displays a clear, verifiable privacy status:

```
┌─────────────────────────────────────────┐
│          PRIVACY STATUS                  │
├─────────────────────────────────────────┤
│ NETWORK ACCESS      NONE                │
│ CLOUD PROCESSING    NONE                │
│ TELEMETRY           OFF / NOT PRESENT   │
│ AUDIO STORAGE       OFF                 │
│ HISTORY             [User Setting]      │
│ CONTEXT ACCESS      [Current Level]     │
│ PERSONALIZATION     [Current State]     │
│ MODEL               [Local Model Name]  │
│ DATABASE            ENCRYPTED           │
└─────────────────────────────────────────┘
```

---

## Logging Policy

### MUST NEVER appear in logs
- Microphone audio samples or representations
- Transcript text (raw or refined)
- Refinement prompts or LLM input/output
- User dictionary contents
- Application textbox contents or selected text
- Encryption keys or access tokens
- Clipboard data

### MAY appear in logs
- Pipeline state transitions (idle → listening → processing)
- Stage durations (ms)
- Buffer occupancy (%)
- Model identifier (name, not content)
- Anonymous local error codes
- Audio device names (non-sensitive metadata)
- Configuration changes (setting name, not value where sensitive)

---

## Secure Deletion Limitations

> [!IMPORTANT]
> FlowDictate documents these limitations honestly rather than making claims that cannot be verified.

**SSDs and NVMe**: Wear-leveling algorithms may retain copies of "deleted" data in unmapped flash pages. The `TRIM` command marks blocks as available but does not guarantee physical erasure. Full-disk encryption before sensitive data is written is the most effective mitigation.

**Copy-on-Write filesystems** (Btrfs, ZFS, APFS): Snapshots and CoW semantics may retain old versions of modified files indefinitely.

**Journaling filesystems** (ext4, NTFS): Journal entries may briefly contain fragments of written data.

**SQLCipher**: Deleting a row removes it from the logical database but does not guarantee the underlying storage pages are zeroed. `VACUUM` rewrites the database but cannot control SSD behavior.

**Our approach**: Prefer preventing unnecessary sensitive data from being written in the first place. Audio is never persisted. Transcripts are only persisted when explicitly enabled by the user. Encryption provides defense-in-depth.

---

## Data Deletion

"Delete all local data" performs:

1. Delete the SQLCipher database file
2. Remove the encryption key from OS keyring
3. Delete configuration files
4. Delete model cache data
5. Clear any session recovery data
6. Remove log files

**Limitation**: This cannot guarantee physical erasure from storage media (see above). It does guarantee that FlowDictate will not have access to the data upon next launch.
