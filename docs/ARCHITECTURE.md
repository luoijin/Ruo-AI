# Architecture

- **Purpose:** Describe the system, the Kotlin/Rust/Python boundaries and the key end-to-end flows.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [RUST_CORE](RUST_CORE.md), [API_CONTRACT](API_CONTRACT.md), [DATA_MODEL](DATA_MODEL.md), [AI_PIPELINE](AI_PIPELINE.md), [FOLDER_STRUCTURE](FOLDER_STRUCTURE.md)

## System overview [v1]
Three tiers: Android client (Kotlin + Compose over a headless Rust core), FastAPI modular monolith, and PostgreSQL + pgvector with the Gemini API behind `ModelProvider`.

```mermaid
flowchart TD
  subgraph Android
    K[Kotlin + Compose<br/>UI, CameraX, ML Kit, audio, share, permissions]
    R[Rust core via UniFFI<br/>SSE, uploads, SQLCipher, outbox, validation]
    K -->|UniFFI .so| R
  end
  R -->|HTTPS REST + SSE| B
  subgraph Backend[FastAPI modular monolith]
    B[auth / conversations / chat / attachments+worker / audio / providers / safety]
  end
  B --> P[(PostgreSQL + pgvector)]
  B --> G[Gemini API via ModelProvider]
```

## Boundaries
| Layer | Owns | Never does |
|---|---|---|
| Kotlin | Compose UI, navigation, CameraX, ML Kit scanner, pickers, audio record/playback, permissions, share intents, image resize and EXIF strip | Network streaming, DB access |
| Rust core | SSE client and reconnect, resumable uploader and hashing, SQLCipher store, outbox, client validation | Android lifecycle, content URIs, Compose state |
| FastAPI | Auth, conversations, chat orchestration, extraction worker, transcription, provider calls, safety, quotas | Trust client input; expose the provider key |

Details: [ANDROID_CLIENT](ANDROID_CLIENT.md), [RUST_CORE](RUST_CORE.md), [AI_PIPELINE](AI_PIPELINE.md).

## Flow 1: streaming chat [v1]
```mermaid
sequenceDiagram
  participant UI as Kotlin UI
  participant RC as Rust core
  participant API as FastAPI
  participant M as Gemini
  UI->>RC: queue_message(client_msg_id)
  RC->>RC: save locally (queued)
  RC->>API: POST /v1/conversations/{id}/messages (SSE)
  API->>API: auth, quota, safety, language detect, prompt assembly
  API->>M: stream request
  M-->>API: tokens
  API-->>RC: meta, delta..., citation, done
  RC-->>UI: on_delta / on_complete
  Note over RC,API: Retry with same client_msg_id is idempotent
```

## Flow 2: image question [v1]
```mermaid
sequenceDiagram
  participant UI as Kotlin
  participant RC as Rust
  participant API as FastAPI
  participant M as Gemini (vision)
  UI->>UI: scan/pick, resize ~2048px, strip location
  UI->>RC: upload_attachment(path)
  RC->>API: init, chunks
  API-->>RC: ready
  UI->>RC: send with intent/question
  API->>M: image sent directly (no OCR first)
  M-->>API: stream
  API-->>RC: SSE events
```

## Flow 3: file extraction [v1]
```mermaid
sequenceDiagram
  participant RC as Rust
  participant API as FastAPI
  participant W as Job worker
  participant DB as Postgres
  RC->>API: init + chunks
  API->>API: verify type by header, size caps
  API->>DB: job row (queued)
  W->>DB: claim job (row lock, heartbeat)
  W->>W: extract text/tables; vision OCR only for scanned pages
  W->>DB: extracted_content; chunks + embeddings if long
  W->>DB: purge raw chunks, status ready
  API-->>RC: attachment events SSE: ready
```

## Flow 4: voice with review [v1]
```mermaid
sequenceDiagram
  participant UI as Kotlin
  participant RC as Rust
  participant API as FastAPI
  UI->>UI: record 16 kHz mono to file
  UI->>RC: upload audio
  RC->>API: POST /v1/audio/transcribe
  API-->>RC: transcript, language, confidence
  RC-->>UI: result
  UI->>UI: fill composer; warn if low confidence
  Note over UI: User edits, then taps Send. Never auto-send.
```

## Flow 5: share intake [v1]
```mermaid
sequenceDiagram
  participant App as Other app
  participant UI as Kotlin
  participant RC as Rust
  App->>UI: share intent (image/PDF/text)
  UI->>UI: copy into app sandbox immediately
  UI->>UI: offer New chat / Add to recent chat
  UI->>RC: upload_attachment(sandbox path)
```

## Flow 6: offline outbox sync [v1]
```mermaid
sequenceDiagram
  participant UI as Kotlin
  participant RC as Rust
  participant WM as WorkManager
  participant API as FastAPI
  UI->>RC: queue_message (offline)
  RC->>RC: store Queued, time-ordered ID
  WM->>RC: flush_outbox (network constraint)
  RC->>API: replay with client_msg_id
  API-->>RC: stored or duplicate-ignored
  RC->>RC: mark sent
```

## Cross-cutting
Safety and injection isolation: [AI_PIPELINE](AI_PIPELINE.md), [SECURITY_PRIVACY](SECURITY_PRIVACY.md). Contracts: [API_CONTRACT](API_CONTRACT.md). Gateway-ready contracts mean a v2 gateway could be added without client changes ([ADR-0003](decisions/ADR-0003-fastapi-only-backend.md)).
