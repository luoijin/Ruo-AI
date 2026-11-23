# API Contract

- **Purpose:** Define all /v1 endpoints, SSE events, error format, idempotency and versioning. Canonical home for the API.
- **Audience:** Backend and client engineers, agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
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
GET `/conversations` (paginated, `?q=` search) · POST `/conversations` · PATCH `/conversations/{id}` (rename, study mode, active language) · DELETE `/conversations/{id}` (soft delete).

## Messages [v1]
POST `/conversations/{id}/messages` starts an SSE stream. Body fields:
| Field | Meaning |
|---|---|
| client_msg_id | Idempotency key (time-ordered unique ID from the client) |
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
data: {"message_id":"...","detected_language":"tl-PH","study_step":1,"model":"...","fallback":false}

event: delta
data: {"text":"Una, tingnan natin ang "}

event: citation
data: {"attachment_id":"...","page":3,"snippet":"..."}

event: done
data: {"usage":{"input_tokens":840,"output_tokens":210},"finish_reason":"stop"}

event: error
data: {"code":"RATE_LIMIT_EXCEEDED","message":"...","retry_after_sec":42}
```
Numbers above are illustrative payload examples, not limits. The `fallback` flag is set when a lighter model was used ([AI_PIPELINE](AI_PIPELINE.md)).

## Errors [v1]
Problem-details object (type, title, status, detail) plus a machine-readable `code` and a `request_id`. Content type `application/problem+json`.

## Idempotency [v1]
`messages.client_msg_id` is unique. A retried send with the same ID returns the stored result or resumes the stream; it never creates a duplicate. Outbox replays rely on this ([DATA_MODEL](DATA_MODEL.md)).

## Versioning rules [v1]
- Breaking changes require `/v2` paths; additive changes (new optional fields, new SSE events) stay in `/v1`.
- Clients ignore unknown SSE events and fields.
- Any contract change updates this doc, the Rust types and the changelog in the same change.
