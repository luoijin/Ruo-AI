# AGENTS.md: Entry point for AI agents

- **Purpose:** Give any AI coding agent the rules, reading order and definition of done before it touches the repo.
- **Audience:** AI coding agents and the owner.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [skills.md](skills.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [ROADMAP.md](ROADMAP.md), [docs/decisions/README.md](docs/decisions/README.md)

## Project summary and status
Ruo AI is an Android-only AI assistant (students, self-learners; bilingual users on mid-range phones and flaky networks) for Google Play and a portfolio. Not an MVP: v1 is the full scope at release quality. **Status: documentation scaffold only; no application code yet. Next: Phase 1.**

## Reading order
1. This file → 2. [PRODUCT](docs/PRODUCT.md) → 3. [ARCHITECTURE](docs/ARCHITECTURE.md) → 4. [FOLDER_STRUCTURE](docs/FOLDER_STRUCTURE.md) → 5. the current [phase doc](docs/phases/PHASE-1-core-spine.md) → 6. the layer docs you touch ([ANDROID_CLIENT](docs/ANDROID_CLIENT.md), [RUST_CORE](docs/RUST_CORE.md), [API_CONTRACT](docs/API_CONTRACT.md), [DATA_MODEL](docs/DATA_MODEL.md), [AI_PIPELINE](docs/AI_PIPELINE.md), [SECURITY_PRIVACY](docs/SECURITY_PRIVACY.md)) → 7. the matching playbook in [skills.md](skills.md).

## Locked decisions [v1]
- Android only. No iOS. Min SDK 26. ([ADR-0001](docs/decisions/ADR-0001-android-only.md))
- Kotlin + Compose client; headless Rust core via cargo-ndk + UniFFI. ([ADR-0002](docs/decisions/ADR-0002-rust-uniffi-client-core.md))
- Python FastAPI modular monolith; no Axum gateway in v1. ([ADR-0003](docs/decisions/ADR-0003-fastapi-only-backend.md))
- One multimodal provider (Gemini) behind `ModelProvider`. ([ADR-0004](docs/decisions/ADR-0004-single-gemini-provider.md))
- PostgreSQL + pgvector; no Redis, no S3; $0/month infrastructure. ([ADR-0005](docs/decisions/ADR-0005-zero-cost-infrastructure.md))
- Voice transcript is reviewed before sending; never auto-send. ([ADR-0006](docs/decisions/ADR-0006-review-before-send-voice.md))
- Postgres-based job queue, rate limits and quotas. ([ADR-0007](docs/decisions/ADR-0007-postgres-queue-and-quotas.md))
- Invite-only beta enforced on the backend; Google sign-in plus email one-time code. ([ADR-0008](docs/decisions/ADR-0008-invite-only-beta.md))

**Rule:** do not change a locked decision without writing a new ADR and getting the owner's approval.

## Hard rules
1. The Rust core stays headless: no Android lifecycle, content URIs or Compose state.
2. No secrets or API keys in the app or repo. The AI provider key lives only on the backend.
3. No v2 features ([BACKLOG_V2](BACKLOG_V2.md) is read-only during v1).
4. No iOS code or portability arguments.
5. No Redis or S3 in v1.
6. All untrusted document and OCR text goes through the injection-isolation wrapper ([AI_PIPELINE](docs/AI_PIPELINE.md)).
7. User-facing strings live in resource files (English and Filipino).
8. No raw uploads kept beyond the retention limit ([SECURITY_PRIVACY](docs/SECURITY_PRIVACY.md)).

## Working agreements
- Work phase by phase, one small change at a time.
- Write tests with the code. Run linters and tests before declaring done.
- Update docs and [CHANGELOG](CHANGELOG.md) in the same change.
- Never fabricate versions, prices, quotas, model names or API details. Check official docs; record status in [VERIFY_BEFORE_BUILD](docs/VERIFY_BEFORE_BUILD.md). Mark anything unconfirmed **UNVERIFIED**.
- If a requirement conflicts or is missing, stop and ask; log it in [OPEN_QUESTIONS](docs/OPEN_QUESTIONS.md).
- Dev hardware is constrained: physical phone, no emulator images ([DEVELOPMENT_SETUP](docs/DEVELOPMENT_SETUP.md)).

## Definition of done (any task)
- [ ] Behavior matches the phase doc and [PRODUCT](docs/PRODUCT.md) acceptance criteria
- [ ] Tests added and passing; linters clean
- [ ] No secrets, no hard-coded strings, no unverified facts stated as fact
- [ ] Docs updated; ADR written if a significant decision was made
- [ ] CHANGELOG `[Unreleased]` updated
- [ ] Security checklist in [skills.md](skills.md) applied if the change touches auth, uploads, prompts or storage

## Command cheat sheet (fill in once code exists)
| Task | Command |
|---|---|
| Start backend stack | `docker compose -f infra/docker-compose.yml up` |
| Backend tests | `pytest (in ai-service/)` |
| Backend lint | `ruff check . && ruff format --check . (in ai-service/)` |
| Rust tests | `cargo test (in core-rust/)` |
| Rust lint | `cargo clippy --all-targets (in core-rust/)` |
| Build Rust for Android | `TODO: cargo ndk ... (Phase 1 Stage B)` |
| Generate UniFFI bindings | `TODO: uniffi-bindgen ... (Phase 1 Stage B)` |
| Android build | `TODO: ./gradlew assembleDebug (Phase 1 Stage B)` |
| Android tests | `TODO: ./gradlew test (Phase 1 Stage B)` |
| Migrations | `alembic upgrade head (in ai-service/)` |
| Run evals | `TODO: pytest evals/ (Phase 1 Stage D)` |

## Escalation: stop and ask the owner when
- A locked decision, the API contract or the data model must change.
- A requirement conflicts with another doc, or is missing.
- A fact is UNVERIFIED and the task depends on it.
- A change would add a permission, a paid service, or a new data category sent to the provider.
- A task seems to need a v2 feature.
