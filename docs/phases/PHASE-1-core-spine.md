# Phase 1: Core spine

- **Purpose:** Plan and gate Phase 1: Core spine.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ROADMAP](../../ROADMAP.md), [PRODUCT](../PRODUCT.md), [AGENTS.md](../../AGENTS.md), [skills.md](../../skills.md)

## Goal
Sign in, stream replies and read cached chats offline. Effort: large.

## Features covered [v1]
Features 1 (chat and history), 13 (sign-in), 14 (usage limits), 15 (moderation basics), 16 (cached reading), 17 (theme, basic settings).

## Ordered tasks
1. Create monorepo, `.gitignore`, CI skeleton (Rust, backend, Android build). [v1]
2. **Spike (week one):** minimal Rust crate opens an SQLCipher DB and calls back into Kotlin via UniFFI on the phone; resolve the OpenSSL/SQLCipher build; record in an ADR. [v1]
3. Freeze the first UniFFI interface, including cancel ([RUST_CORE](../RUST_CORE.md)). [v1]
4. Docker Compose with FastAPI + Postgres/pgvector; Alembic migrations for users, invites, refresh_tokens, conversations, messages, usage_logs. [v1]
5. Verify Gemini key, model names and free-tier limits ([VERIFY_BEFORE_BUILD](../VERIFY_BEFORE_BUILD.md)); implement `ModelProvider` + fake provider + streaming call. [v1]
6. Auth: Google ID token verification, email one-time code, JWT + refresh rotation, invite gate. [v1]
7. Conversations CRUD and search; message SSE stream with `client_msg_id` idempotency. [v1]
8. Quotas and rate limits in Postgres; problem-details errors; basic moderation. [v1]
9. Rust: SSE client with reconnect and cancel; SQLCipher store and migrations; Keystore-wrapped key. [v1]
10. Compose: Auth, Chat, history, Markdown/code rendering, theme; strings in English and Filipino. [v1]
11. Offline reading of cached history. [v1]
12. Seed 20 multilingual eval prompts; write the first ADRs for decisions made. [v1]

## Dependencies
[VERIFY_BEFORE_BUILD](../VERIFY_BEFORE_BUILD.md) items for toolchain, Gemini limits and model names.

## Acceptance criteria
- Sign in with Google and with an email code; uninvited users are rejected by the backend.
- A reply streams token by token; stop cancels it.
- Retrying a send never duplicates a message.
- Cached chats open offline.
- Quota and rate-limit errors show a clear retry time.

## Test requirements
Rust unit tests (SSE framing, backoff, migrations); backend tests (auth rotation, quotas, idempotency, streaming with fake provider); FFI test with Taglish/emoji strings; Compose test of the chat screen.

## Agent notes (pitfalls)
- SQLCipher + OpenSSL on Android is the top build risk: spike first, do not build features on top of an unproven toolchain.
- Do not store the DB key in Rust or in plain preferences.
- Hop to the main dispatcher in callbacks.
- Do not add Redis, a gateway, or any v2 feature.

## Definition of done
- [ ] Acceptance criteria met and demonstrated on a physical phone
- [ ] Tests listed above passing in CI
- [ ] Docs, API_CONTRACT/DATA_MODEL (if changed) and CHANGELOG updated
- [ ] No unverified fact stated as fact; VERIFY_BEFORE_BUILD statuses updated
- [ ] Security checklist in [skills.md](../../skills.md) applied
