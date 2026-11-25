# Data Model

- **Purpose:** Define PostgreSQL + pgvector tables, indexes, the local SQLCipher schema, sync rules and storage hygiene.
- **Audience:** Backend and client engineers, agents.
- **Status:** Draft
- **Last updated:** 2026-11-25
- **Related docs:** [API_CONTRACT](API_CONTRACT.md), [RUST_CORE](RUST_CORE.md), [SECURITY_PRIVACY](SECURITY_PRIVACY.md), [ADR-0007](decisions/ADR-0007-postgres-queue-and-quotas.md)

## PostgreSQL + pgvector [v1]
| Table | Columns (summary) |
|---|---|
| users | id, email, google_sub, ui_lang, preferred_ai_lang, deleted_at (no password hash: sign-in is Google or email code) |
| invites | code or allowlisted email, used_by, created_at, expires_at |
| refresh_tokens | user, token hash, expires_at, revoked_at |
| conversations | user, client-generated id, title, active_language, study_mode_default, updated_at, deleted_at |
| messages | conversation, **unique client_msg_id**, parent, role, content_md, status, citations_json, study_step, language, model_used, created_at, updated_at |
| attachments | user, sha256, filename, mime_type, size_bytes, page_count, status, expires_at |
| upload_chunks | attachment, chunk_index, data, created_at (temporary) |
| extracted_content | attachment, full Markdown, token count, OCR page info, parser version |
| document_chunks | attachment, chunk_index, page_number, heading, content, text-search vector, embedding |
| processing_jobs | attachment, status, stage, attempts, error_code, updated_at (+ heartbeat) |
| feedback | message, user, rating, reason_code, comment, created_at |
| usage_logs | user, UTC date, request count, tokens in/out, upload bytes |

## Indexes [v1]
- conversations: (user, updated_at DESC, id DESC); full-text index on title.
- messages: unique (client_msg_id); index on conversation and (created_at DESC, id DESC).
- document_chunks: HNSW on embedding; GIN on text-search vector.
- usage_logs: unique (user, UTC date). Quotas use an atomic insert-or-update.
- **Embedding dimension depends on the model chosen: confirm before creating the table** (UNVERIFIED; [VERIFY_BEFORE_BUILD](VERIFY_BEFORE_BUILD.md)).

## Job queue without Redis [v1]
An in-process worker claims rows from `processing_jobs` using row locking (`FOR UPDATE SKIP LOCKED` style) with a heartbeat; stale jobs are recovered after restarts ([ADR-0007](decisions/ADR-0007-postgres-queue-and-quotas.md)).

## Local SQLCipher schema [v1]
Mirrors conversations and messages, plus `attachment_cache`, `drafts` (per conversation) and `outbox`. Schema migrations are versioned and run on open; key loss on app update is mitigated by a Keystore-held key and tested migrations.

## Sync and conflict rules [v1]
- Server is the source of truth for sent messages.
- The client generates a time-ordered unique ID per message; outbox replays never duplicate (server unique constraint).
- Phase 1 conversation creation, rename, and deletion are server-first and online-only; local cache changes only after the server response. Server soft-delete tombstones propagate through paged sync and remove the local conversation and its messages.
- Offline conversation edits and last-write-wins conflict handling are deferred to Phase 5; Phase 1 does not queue edits or resolve concurrent edits.
- Conversation pages use the `(updated_at, id)` keyset; message pages use `(created_at, id)`. Bounds for recent offline conversation/message cache are supplied by app configuration, not constants in the Rust core.
- Multi-device sync is [v2] and out of scope.

## Storage hygiene [v1]
Raw files and chunks exist only while needed: purge after processing or within 24 hours. Only extracted text and embeddings persist. Deleting an attachment or conversation removes its chunks. Orphaned chunks are swept on a schedule. Retention rules: [SECURITY_PRIVACY](SECURITY_PRIVACY.md).
