# API Contract

- **Purpose:** Define all /v1 endpoints, SSE events, error format, idempotency and versioning. Canonical home for the API.
- **Audience:** Backend and client engineers, agents.
- **Status:** Draft
- **Last updated:** 2026-11-25
- **Related docs:** [DATA_MODEL](DATA_MODEL.md), [RUST_CORE](RUST_CORE.md), [AI_PIPELINE](AI_PIPELINE.md), [SECURITY_PRIVACY](SECURITY_PRIVACY.md)

All endpoints live under `/v1` [v1]. Numeric defaults are starting points to tune. Contracts stay gateway-ready.

## Auth [v1]
| Method | Path | Notes |
|---|---|---|
| POST | /auth/google | Verify Google ID token; requires invite code or allowlisted email to sign up |
| POST | /auth/email/request | Send one-time code |
| POST | /auth/email/verify | Exchange code for tokens |
| POST | /auth/refresh | Rotating refresh token |

Access token about 15 min; refresh token about 30 days, rotating, stored hashed.

## Profile [v1]
GET `/me` (profile, quota status) · POST `/me/export` (async data export) · DELETE `/me` (full cascade deletion).

## Conversations [v1]
POST `/conversations` requires a client-generated UUID `conversation_id`; title, active language, and study-mode default use their documented request defaults when omitted. Repeating the same ID for the same owner returns the existing row without changing it; an ID owned by another user returns `409 CONVERSATION_ID_CONFLICT`.

GET `/conversations` accepts `q`, `limit`, `before_updated_at`, `before_id`, and `include_deleted`. Results are newest-first by `(updated_at DESC, id DESC)`. The cursor fields must be supplied together; the next page uses the last row's `updated_at` and `id`. `include_deleted=true` is for sync and includes tombstones with `deleted_at`; normal UI reads exclude soft-deleted conversations. PATCH `/conversations/{id}` (rename, study mode, active language) and DELETE `/conversations/{id}` (soft delete) are server-first and online-only in Phase 1.

GET `/conversations/{id}` returns conversation metadata. GET `/conversations/{id}/messages` accepts `limit`, `before_created_at`, and `before_id`, ordered newest-first by `(created_at DESC, id DESC)`; clients reverse a page for chronological display. Each message includes its stable `status` (`complete`, `streaming`, or `failed`). The client configures its page size and bounded recent-message sync window. Delete tombstones are propagated through the conversation list, not by returning deleted messages.

## Messages [v1]
POST `/conversations/{id}/messages` starts an SSE stream. Body fields:
| Field | Meaning |
|---|---|
| client_msg_id | Required idempotency key (time-ordered unique ID from the client) |
| content | Required user message text |
| parent_message_id | For edit or regenerate |
| attachment_ids | Attachments to include |
| intent | summarize, explain, translate, solve, check_answer (optional) |
| study_mode | boolean |

## Attachments [v1]
POST `/attachments/init` · PUT `/attachments/{id}/chunk` (about 2 MB chunks) · GET `/attachments/{id}/events` (SSE: queued, parsing, ocr, indexing, ready, failed).

## Audio [v1]
POST `/audio/transcribe` returns `transcript`, `detected_language`, `confidence`.

## Feedback [v1]
POST `/feedback` with rating, reason code, optional comment, linked to a message.

## SSE events (message stream) [v1]
```
event: meta
data: {"client_msg_id":"...","user_message_id":"...","assistant_message_id":"...","detected_language":"tl-PH","study_step":1,"model":"...","fallback":false,"replayed":false}

event: delta
data: {"text":"Una, tingnan natin ang "}

event: citation
data: {"attachment_id":"...","page":3,"snippet":"..."}

event: done
data: {"client_msg_id":"...","assistant_message_id":"...","replayed":false,"usage":{"input_tokens":840,"output_tokens":210},"finish_reason":"stop"}

event: error
data: {"code":"RATE_LIMIT_EXCEEDED","message":"...","retryable":true,"retry_after_sec":42}
```
Numbers above are illustrative payload examples, not limits. The `fallback` flag is set when a lighter model was used ([AI_PIPELINE](AI_PIPELINE.md)).

## Errors [v1]
Problem-details object (type, title, status, detail) plus a machine-readable `code` and a `request_id`. Content type `application/problem+json`.

## Idempotency [v1]
`messages.client_msg_id` is unique. The client-generated `client_msg_id` identifies the send and is the client-side stream handle; database `messages.id` values are server IDs and are returned separately as `user_message_id` and `assistant_message_id`. An assistant row uses `asst_{client_msg_id}` as its unique client idempotency key.

A send persists its user row and streaming assistant placeholder before emitting deltas. A retry with the same `client_msg_id` returns a full replay when complete; while the original is active it returns `STREAM_IN_PROGRESS` with `retryable=true` and `retry_after_sec`; after a failed stream it restarts generation using the same rows and does not charge quota again. The assistant row's final content and `complete` state are committed before the `done` event. Retries never create another user or assistant row. Mid-stream continuation from a byte/event cursor is not promised. Outbox replays rely on this behavior ([DATA_MODEL](DATA_MODEL.md)).

## Versioning rules [v1]
- Breaking changes require `/v2` paths; additive changes (new optional fields, new SSE events) stay in `/v1`.
- Clients ignore unknown SSE events and fields.
- Any contract change updates this doc, the Rust types and the changelog in the same change.
