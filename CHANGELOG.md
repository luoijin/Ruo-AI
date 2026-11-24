# Changelog

- **Purpose:** Record every notable change. Format: Keep a Changelog; versioning: Semantic Versioning.
- **Audience:** Everyone.
- **Status:** Draft
- **Last updated:** 2026-11-23
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
- Auth service: Google ID token verification, email one-time code (`EmailSender` abstraction with console/dev and SMTP support), JWT access tokens, and rotating hashed refresh tokens.
- ModelProvider architecture with `FakeProvider` (Taglish sample) and `GeminiProvider` (reading models from configuration, with jittered retry on 429).
- Conversation & Chat SSE streaming with `client_msg_id` idempotency, daily message quotas in Postgres, and calm refusal moderation for self-harm crisis support.
- Headless Rust core `ruo_core`: SSE parser, reconnect backoff and jitter policy, SQLCipher/SQLite store migrations, outbox state types, stream cancellation, and UniFFI interface.
- GitHub Actions CI workflow for Rust and Backend test suites.


## [0.0.0] - 2026-11-23
- Repository initialized.
