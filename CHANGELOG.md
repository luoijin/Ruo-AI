# Changelog

- **Purpose:** Record every notable change. Format: Keep a Changelog; versioning: Semantic Versioning.
- **Audience:** Everyone.
- **Status:** Draft
- **Last updated:** 2026-11-25
- **Related docs:** [CONTRIBUTING.md](CONTRIBUTING.md), [AGENTS.md](AGENTS.md)

Every merged change updates this file under `[Unreleased]`.

## Categories
- **Added**: new features or files
- **Changed**: changes to existing behavior
- **Deprecated**: soon to be removed
- **Removed**: removed features
- **Fixed**: bug fixes
- **Security**: vulnerability fixes or hardening

## [Unreleased]
### Added
- Backend foundation in `ai-service`: FastAPI modular monolith with health check, problem-details error handler, structured logging, and configuration via environment variables.
- Alembic database schema migrations for `users`, `invites`, `email_otps`, `refresh_tokens`, `conversations`, `messages` (unique `client_msg_id`), and `usage_logs`.
- Beta invite gate and CLI management script (`scripts/manage_invites.py`).
- Auth service: Google ID token verification, email one-time code (`EmailSender` abstraction with console/dev sender only), JWT access tokens, and rotating hashed refresh tokens.
- ModelProvider architecture with `FakeProvider` (Taglish sample) and `GeminiProvider` (reading models from configuration, with jittered retry on 429).
- Conversation & Chat SSE streaming with `client_msg_id` idempotency, daily message quotas in Postgres, and calm refusal moderation for self-harm crisis support.
- Headless Rust core `ruo_core`: SSE parser, reconnect backoff and jitter policy, SQLCipher/SQLite store migrations, outbox state types, stream cancellation, and UniFFI interface.
- GitHub Actions CI workflow for Rust and Backend test suites.
- Concurrency test suite on PostgreSQL verifying boundary race conditions and first-request-of-day concurrency.
- Architecture decision record [ADR-0009](docs/decisions/ADR-0009-uniffi-version-and-toolchain.md) establishing UniFFI 0.32.2 as the Android bridge baseline.
- Minimal Android `:app` / `:core:rustbridge` SQLCipher spike, arm64 NDK/UniFFI/APK CI build, and on-device instrumentation checks for encrypted storage and callbacks.
- Accepted [ADR-0010](docs/decisions/ADR-0010-sqlcipher-android-build.md), recording the verified Android SQLCipher/OpenSSL build, artifact size, and third-party notice requirements.
- Froze Phase 1 UniFFI interface v1 and implemented `:core:rustbridge` with Keystore-wrapped SQLCipher keys, Kotlin Flow adapters, and dispatcher hops; no Compose screens were added.
- Added the approved Gate 3 interface conditions: client-generated conversation IDs, keyset sync cursors and tombstones, stream ID semantics, sign-out/key-loss policy, safe errors, and Phase 1 online-only conversation edits.
- Added multi-agent Phase 1 status docs: `docs/PHASE-1-STATUS.md`, per-agent status files under `docs/status/`, and the updated Phase 1 Definition of Done and command cheat sheet in `AGENTS.md`.

### Changed
- Removed unverified default model name from `GEMINI_FALLBACK_MODEL` (remains unconfigured until verified).
- Added backend container healthcheck, database connection parameters, and automatic Alembic migration execution to `infra/docker-compose.yml`.
- Corrected the email sender description and tightened UniFFI, rusqlite, and Android toolchain verification notes.
- Recorded Gemini streaming latency trials and verified the documented thinking-level controls.
- Added the AndroidX annotation dependency required by generated UniFFI bindings and renamed exported Rust error payload fields to `details` to avoid Kotlin `Throwable.message` conflicts.
- Verified the Gate 2 SQLCipher/UniFFI build in CI and passed the key, encrypted-header, multilingual round-trip, and Kotlin callback test on a physical API 34 arm64 phone.
- Added an explicit host-debug plain-SQLite logic-test mode, compile guards against Android/release use, a Linux SQLCipher test job, and a `gh`-based latest-library download script.
- Preserved full server timestamp precision in local keyset pagination and added FFI instrumentation coverage for SQLCipher key errors, encrypted storage, callbacks, and stream cancellation.
- Made the local-change callback instrumentation subscribe before the draft write, use unique mock server IDs across streams, and report failures by safe event type.
- Published the two Android instrumentation APKs as a separate CI artifact so device verification can fetch them without downloading the Rust library.
- Added device coverage for concurrent-stream conflicts and assistant persistence after SSE `done`; added a release-mode test that rejects loopback HTTP even when `allow_cleartext_debug` is true.
- Kept the sync-only deleted-conversation flag out of Kotlin's `listConversations` API; the UI listing now returns active conversations only.
- Added the Gate 4 stable CI debug-signing-key plan, including repository-secret names, OAuth SHA-1 registration, and trusted-workflow handling. No signing key or workflow secret is committed.
- Added HTTP 401 error-mapping coverage, listener teardown policy, backup-rule parsing and configured offline-cache boundary tests; tracked UI, compatibility, offline-edit, API 26, and low-RAM acceptance gaps in their phase docs.

### Fixed
- Fixed quota check-then-act TOCTOU race condition using a single atomic conditional upsert (`ON CONFLICT ... DO UPDATE ... WHERE request_count < limit RETURNING; zero rows = exhausted`) so quota limits are strictly enforced under concurrency without over-counting or over-allowing.

### Security
- Enforced strict startup validation outside `ENVIRONMENT=development` to reject empty or dev placeholder values for `JWT_SECRET_KEY` and `GOOGLE_CLIENT_ID`.

### Removed
- Removed unused, unenforced `HOURLY_RATE_LIMIT` configuration variable.


## [0.0.0] - 2026-11-23
- Repository initialized.
