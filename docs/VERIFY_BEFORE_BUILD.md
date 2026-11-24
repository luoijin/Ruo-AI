# Verify Before Build

- **Purpose:** Checklist of every UNVERIFIED item, how to verify it, and its status.
- **Audience:** Owner and agents.
- **Status:** Draft
- **Last updated:** 2026-11-24
- **Related docs:** [TECH_STACK](TECH_STACK.md), [OPEN_QUESTIONS](OPEN_QUESTIONS.md)

Update the Status column (Open / Verified + date + source). Never record a fact without a source.

| # | Item | How to verify | Needed by | Status |
|---|---|---|---|---|
| V1 | Is the main Gemini model on the free tier; actual per-minute and per-day limits for each model used | Google AI Studio rate-limit page | Phase 1 | Open |
| V2 | Exact model names: chat, lighter fallback, transcription, embeddings | AI Studio model list | Phase 1 / 3 / 4 | Verified 2026-11-23: `gemini-4-hydrogen` exists. Fallback model configurable (default: `gemini-1.5-flash` UNVERIFIED). |
| V3 | Embedding dimension | Model docs; confirm before creating `document_chunks` | Phase 3 | Open |
| V4 | Free hosting and Postgres terms: card required, idle sleep, RAM/CPU, storage | Each provider's current pricing and terms pages | Phase 3 | Open |
| V5 | Versions: Rust, UniFFI, cargo-ndk, SQLCipher binding | crates.io, project READMEs | Phase 1 | **Verified 2026-11-24**: Host rustc 1.100.0, uniffi 0.28.3 installed via Cargo, crates.io latest uniffi 0.32.2 (source: `cargo info uniffi`), rusqlite 0.31.0 installed (latest 0.43.0, source: `cargo info rusqlite`). SQLCipher requires OpenSSL C headers (`OPENSSL_DIR`), which fails on Windows host without MSVC OpenSSL dev package; Android NDK SQLCipher + OpenSSL build remains UNVERIFIED. |
| V6 | Versions: Kotlin, AGP, Gradle, Compose BOM, CameraX, ML Kit Document Scanner, Hilt, target SDK | maven.google.com, developer.android.com | Phase 1 | Open |
| V7 | Versions: Python, FastAPI, SQLAlchemy, Alembic, PyMuPDF, pdfplumber, python-docx, pgvector | pypi.org, project docs | Phase 1 / 3 | **Verified 2026-11-24**: Python 3.14.8, FastAPI 0.165.0, SQLAlchemy 2.1.9, Alembic 1.20.1, aiosqlite 0.22.1, greenlet 3.5.6 (source: local `pip list` / PyPI). |
| V8 | Current Play Console requirements (fee, card, ID, closed-test rule) | Play Console Help at registration | Before Phase 3 | Open (US$25 and 12 testers/14 days: verified) |
| V9 | UniFFI export syntax for the installed version | UniFFI docs for that version | Phase 1 | **Verified 2026-11-24**: For uniffi 0.28.3, proc-macro only mode does not use `uniffi::setup_scaffolding!` in `build.rs`; foreign callbacks take `Arc<dyn Trait>` (source: uniffi compiler diagnostic). |
| V10 | Email delivery service free tier | Provider docs | Phase 1 | Open. Dev uses `ConsoleEmailSender`. |
| V11 | Crash reporting free tier handling native Rust crashes | Crashlytics/Sentry docs | Phase 5 | Open |
| V12 | Font licenses and Android bundling (Plus Jakarta Sans, Inter, JetBrains Mono) | Each font's license | Phase 1 | Open |
| V13 | Philippine Data Privacy Act and Play minors policy implications (not legal advice) | Official texts; consult a professional if needed | Phase 5 | Open |
