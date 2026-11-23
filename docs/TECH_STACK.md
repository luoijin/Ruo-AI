# Tech Stack

- **Purpose:** List each technology, why chosen, what it replaced or deferred, and verification status of versions.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [VERIFY_BEFORE_BUILD](VERIFY_BEFORE_BUILD.md), [ARCHITECTURE](ARCHITECTURE.md)

All versions are **UNVERIFIED** until checked. Never copy a version from memory or from the planning document.

## Choices [v1]
| Area | Choice | Why | Replaced or deferred |
|---|---|---|---|
| UI | Kotlin + Jetpack Compose | Native Android, modern UI toolkit | iOS/cross-platform: ruled out ([ADR-0001](decisions/ADR-0001-android-only.md)) |
| Client core | Rust + cargo-ndk + UniFFI | Reliability for streaming, uploads, encrypted cache; skills showcase | Pure Kotlin networking ([ADR-0002](decisions/ADR-0002-rust-uniffi-client-core.md)) |
| Local DB | SQLCipher | Encrypted history, drafts, outbox | Plain SQLite |
| Backend | Python FastAPI, modular monolith | Fast to build; strong AI/doc-parsing ecosystem | Rust Axum gateway: v2 ADR only ([ADR-0003](decisions/ADR-0003-fastapi-only-backend.md)) |
| Database | PostgreSQL + pgvector | One store for data, queue, quotas, vectors | Redis, vector DB ([ADR-0005](decisions/ADR-0005-zero-cost-infrastructure.md)) |
| AI | Gemini behind `ModelProvider` | One multimodal provider; swappable | Multi-provider ([ADR-0004](decisions/ADR-0004-single-gemini-provider.md)) |
| Extraction | PyMuPDF, pdfplumber, python-docx | Text and tables from PDF and DOCX | OCR-first (loses handwriting/math layout) |
| Camera/scan | CameraX, ML Kit Document Scanner | Standard Android capture and cleanup | none |
| DI | Hilt (verify it still suits) | Standard for Android | none |
| CI | GitHub Actions free tier | $0 | paid CI |
| Hosting | Oracle Cloud Always Free VM preferred | No cold starts; RAM for parsing | Koyeb, Render, Neon, Supabase: UNVERIFIED fallbacks |
| TTS | Android built-in engine | Free; saves AI quota | Gemini TTS: v2 |

## Version table (all UNVERIFIED)
| Component | Version | Verify with |
|---|---|---|
| Android SDK platform / target | UNVERIFIED | Compose BOM release notes; developer.android.com |
| Compose BOM | UNVERIFIED | maven.google.com (androidx.compose:compose-bom) |
| Kotlin / AGP / Gradle | UNVERIFIED | kotlinlang.org; developer.android.com/build/releases |
| CameraX | UNVERIFIED | developer.android.com/jetpack/androidx/releases/camera |
| ML Kit Document Scanner | UNVERIFIED | developers.google.com/ml-kit |
| Hilt | UNVERIFIED | developer.android.com/training/dependency-injection/hilt-android |
| Rust toolchain | UNVERIFIED | `rustup check` |
| cargo-ndk | UNVERIFIED | `cargo search cargo-ndk` or crates.io |
| UniFFI | UNVERIFIED | crates.io/crates/uniffi; verify syntax against its docs |
| SQLCipher binding (rusqlite feature or equivalent) | UNVERIFIED | crates.io; test OpenSSL-on-Android build early |
| Python | UNVERIFIED | python.org |
| FastAPI | UNVERIFIED | pypi.org/project/fastapi |
| PyMuPDF / pdfplumber / python-docx | UNVERIFIED | pypi.org |
| PostgreSQL / pgvector | UNVERIFIED | postgresql.org; github.com/pgvector/pgvector |
| Alembic / SQLAlchemy | UNVERIFIED | pypi.org |
| Gemini model names (chat, transcription, embeddings) | `gemini-4-hydrogen` exists (verified, launched 2026-11-13); all others UNVERIFIED | Google AI Studio model list |

Checklist and status tracking: [VERIFY_BEFORE_BUILD](VERIFY_BEFORE_BUILD.md).
