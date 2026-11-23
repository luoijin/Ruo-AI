# Folder Structure

- **Purpose:** Define the target monorepo tree, naming conventions and where new code belongs. Target only: most entries do not exist yet.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ARCHITECTURE](ARCHITECTURE.md), [ANDROID_CLIENT](ANDROID_CLIENT.md), [RUST_CORE](RUST_CORE.md)

## Top level [v1]
```
ruo-ai/
  android/      Kotlin/Compose project
  core-rust/    Cargo workspace (ruo_core cdylib)
  ai-service/   FastAPI backend
  infra/        Compose, CI, deployment notes
  docs/         All documentation
  AGENTS.md  skills.md  README.md  CHANGELOG.md  ROADMAP.md  BACKLOG_V2.md  CONTRIBUTING.md
```

## android/
```
android/
  settings.gradle.kts        module list
  build.gradle.kts           root build config
  gradle/                    version catalog (versions UNVERIFIED until checked)
  app/                       :app application, activities, navigation graph, share-target manifest
  core/
    ui/                      :core:ui design tokens, theme, Markdown/code renderer, attachment preview, citation chips
    rustbridge/              :core:rustbridge Kotlin wrapper over UniFFI bindings, Keystore key handling, Flow adapters
      src/main/jniLibs/      generated .so files (gitignored)
  feature/
    chat/                    :feature:chat chat screen, history, composer, tray, study controls, TTS
    camera/                  :feature:camera CameraX capture, ML Kit scanner, image prep
    auth/                    :feature:auth Google and email-code sign-in
    settings/                :feature:settings theme, languages, privacy, delete/export
```
Each feature module: `ui/` (Composables), `presentation/` (UiState, UiIntent, UiEffect, ViewModel), `domain/` (small use cases), `res/` (strings in `values/` and `values-fil/`).

## core-rust/
```
core-rust/
  Cargo.toml                 workspace
  ruo_core/
    Cargo.toml               crate-type cdylib (+ lib for tests)
    src/lib.rs               UniFFI exports (RuoCore, listeners, records, CoreError)
    src/db/                  SQLCipher open, migrations, repositories
    src/net/                 HTTP client, SSE parser, reconnect/backoff
    src/upload/              resumable upload state machine, hashing
    src/outbox/              outbox queue and state transitions
    src/validate/            client-side validation
    tests/                   integration tests
  scripts/                   cargo-ndk build and bindgen scripts
```

## ai-service/
```
ai-service/
  pyproject.toml             dependencies (versions UNVERIFIED)
  alembic.ini  migrations/   schema migrations
  app/
    main.py                  app factory, router mount under /v1
    core/                    config, logging, errors (problem details), db session
    auth/                    Google ID token verify, email OTP, JWT, refresh rotation, invites
    conversations/           CRUD, search, edit/regenerate, soft delete
    chat/                    prompt assembly, SSE streaming, language, study state, retrieval
    attachments/             upload receiver, validation, extraction worker, chunking
    audio/                   transcription endpoint
    providers/               ModelProvider interface, Gemini implementation, fake provider for tests
    safety/                  rate limits, quotas, moderation, injection sanitizer
    feedback/                feedback endpoint
  tests/                     pytest, fixtures (real PDF/DOCX)
  evals/                     prompts, runner, results
```
Module layout per backend module: `router.py`, `service.py`, `models.py`, `schemas.py`.

## infra/
```
infra/
  docker-compose.yml         FastAPI + Postgres/pgvector for local dev
  .env.example               variable names only, no values
  ci/ or .github/workflows/  CI (GitHub Actions requires workflows under repo-root .github/workflows)
  deploy/                    hosting notes
```

## docs/
Flat layer docs, plus `decisions/` (ADRs), `phases/`, `source/` (exported plan).

## Naming conventions
- Kotlin: `PascalCase` classes, `camelCase` functions, packages `app.ruo.<module>` (placeholder, see [OPEN_QUESTIONS](OPEN_QUESTIONS.md) Q9).
- Rust: `snake_case` modules and functions, `PascalCase` types.
- Python: `snake_case`; one router per module.
- Docs: `UPPER_SNAKE.md` for top-level docs; ADRs `ADR-NNNN-slug.md`.
- Strings: keys `feature_screen_purpose`; Filipino in `values-fil/`.

## Where new code goes
| Kind | Location |
|---|---|
| New screen | `android/feature/<name>/` |
| Shared UI component | `android/core/ui/` |
| New exported Rust function | `core-rust/ruo_core/src/lib.rs` plus the module that implements it |
| New endpoint | `ai-service/app/<module>/router.py`, then [API_CONTRACT](API_CONTRACT.md) |
| New table | Alembic migration, then [DATA_MODEL](DATA_MODEL.md) |
| New provider | `ai-service/app/providers/` |
| New eval case | `ai-service/evals/` |
