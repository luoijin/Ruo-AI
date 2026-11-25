# Ruo AI

- **Purpose:** Say what Ruo AI is, where it stands, and how to find everything else.
- **Audience:** Anyone opening the repo (owner, reviewers, AI agents).
- **Status:** Draft
- **Last updated:** 2026-11-25
- **Related docs:** [AGENTS.md](AGENTS.md), [docs/PRODUCT.md](docs/PRODUCT.md), [ROADMAP.md](ROADMAP.md)

## What it is
Ruo AI is an Android-only AI assistant for students and self-learners, built as a long-term product for Google Play and shown in a portfolio. It is not an MVP. v1 is the full release scope, built to release quality, one phase at a time. v2 is parked ([BACKLOG_V2.md](BACKLOG_V2.md)).

## Status
Phase 1 is underway: Gate 1 is complete, the Gate 2 SQLCipher Android spike was verified on API 34, and the Gate 3B Rust/UniFFI bridge is in progress. No Compose screens have been added yet; see [CHANGELOG.md](CHANGELOG.md).

## v1 features [v1]
Streaming chat with history · camera and multi-image input · file attachments with extraction (PDF, DOCX, TXT, CSV) · voice transcription with review before send · multilingual replies incl. Taglish and Bisaya-English · study mode (hint ladder, check my answer) · Android TTS · share-to-Ruo · document scanner cleanup · citations · quick action chips · feedback buttons · accounts (Google + email code), deletion, export · usage limits · safety · offline · settings. Acceptance criteria: [docs/PRODUCT.md](docs/PRODUCT.md).

## Tech stack
Kotlin + Jetpack Compose · headless Rust core (cargo-ndk + UniFFI) · Python FastAPI modular monolith · PostgreSQL + pgvector · Gemini behind a `ModelProvider` interface · $0/month infrastructure. Details and version status: [docs/TECH_STACK.md](docs/TECH_STACK.md).

## Repo layout
| Folder | Purpose |
|---|---|
| [android/](android/README.md) | Kotlin/Compose app |
| [core-rust/](core-rust/README.md) | Rust core exposed via UniFFI |
| [ai-service/](ai-service/README.md) | FastAPI backend |
| [infra/](infra/README.md) | Docker Compose, CI, deployment notes |
| [docs/](docs/FOLDER_STRUCTURE.md) | All documentation |

## Quick start
Setup lives in one place: [docs/DEVELOPMENT_SETUP.md](docs/DEVELOPMENT_SETUP.md). Agents start at [AGENTS.md](AGENTS.md).

## Docs index
- Product and design: [PRODUCT](docs/PRODUCT.md), [ARCHITECTURE](docs/ARCHITECTURE.md), [FOLDER_STRUCTURE](docs/FOLDER_STRUCTURE.md), [TECH_STACK](docs/TECH_STACK.md), [GLOSSARY](docs/GLOSSARY.md)
- Layers: [ANDROID_CLIENT](docs/ANDROID_CLIENT.md), [RUST_CORE](docs/RUST_CORE.md), [API_CONTRACT](docs/API_CONTRACT.md), [DATA_MODEL](docs/DATA_MODEL.md), [AI_PIPELINE](docs/AI_PIPELINE.md)
- Cross-cutting: [SECURITY_PRIVACY](docs/SECURITY_PRIVACY.md), [INFRASTRUCTURE](docs/INFRASTRUCTURE.md), [TESTING_EVALS](docs/TESTING_EVALS.md), [RELEASE_PLAYBOOK](docs/RELEASE_PLAYBOOK.md)
- Process: [ROADMAP](ROADMAP.md), [phases](docs/phases/PHASE-1-core-spine.md), [decisions (ADRs)](docs/decisions/README.md), [OPEN_QUESTIONS](docs/OPEN_QUESTIONS.md), [VERIFY_BEFORE_BUILD](docs/VERIFY_BEFORE_BUILD.md)
- Agents: [AGENTS.md](AGENTS.md), [skills.md](skills.md), [CONTRIBUTING.md](CONTRIBUTING.md)

## License
Placeholder: license not yet chosen (see [OPEN_QUESTIONS](docs/OPEN_QUESTIONS.md), Q8).
