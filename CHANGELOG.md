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
- Architecture decision record [ADR-0009](docs/decisions/ADR-0009-uniffi-version-and-toolchain.md) establishing UniFFI 0.28.3 baseline and toolchain for Android bridge.

### Changed
- Removed unverified default model name from `GEMINI_FALLBACK_MODEL` (remains unconfigured until verified).
- Added backend container healthcheck, database connection parameters, and automatic Alembic migration execution to `infra/docker-compose.yml`.
- Corrected the email sender description and tightened UniFFI, rusqlite, and Android toolchain verification notes.

### Fixed
- Fixed quota check-then-act TOCTOU race condition using a single atomic conditional upsert (`ON CONFLICT ... DO UPDATE ... WHERE request_count < limit RETURNING; zero rows = exhausted`) so quota limits are strictly enforced under concurrency without over-counting or over-allowing.

### Security
- Enforced strict startup validation outside `ENVIRONMENT=development` to reject empty or dev placeholder values for `JWT_SECRET_KEY` and `GOOGLE_CLIENT_ID`.

### Removed
- Removed unused, unenforced `HOURLY_RATE_LIMIT` configuration variable.


## [0.0.0] - 2026-11-23
- Repository initialized.
