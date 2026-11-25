# Verify Before Build

- **Purpose:** Checklist of every UNVERIFIED item, how to verify it, and its status.
- **Audience:** Owner and agents.
- **Status:** Draft
- **Last updated:** 2026-11-25
- **Related docs:** [TECH_STACK](TECH_STACK.md), [OPEN_QUESTIONS](OPEN_QUESTIONS.md)

Update the Status column (Open / Verified + date + source). Never record a fact without a source.

| # | Item | How to verify | Needed by | Status |
|---|---|---|---|---|
| V1 | Is the main Gemini model on the free tier; actual per-minute and per-day limits for each model used | Google AI Studio rate-limit page | Phase 1 | Open |
| V2 | Exact model names: chat, lighter fallback, transcription, embeddings | AI Studio model list | Phase 1 / 3 / 4 | Verified 2026-11-23: `gemini-4-hydrogen` exists. Fallback model configurable via `GEMINI_FALLBACK_MODEL` with no invented default (UNVERIFIED / unconfigured). |
| V3 | Embedding dimension | Model docs; confirm before creating `document_chunks` | Phase 3 | Open |
| V4 | Free hosting and Postgres terms: card required, idle sleep, RAM/CPU, storage | Each provider's current pricing and terms pages | Phase 3 | Open |
| V5 | Versions: Rust, UniFFI, cargo-ndk, SQLCipher binding | crates.io, project READMEs | Phase 1 | **Partially verified 2026-11-25**: Host `rustc` 1.100.0, UniFFI 0.28.3 in the current crate manifest, and `cargo-ndk` 4.1.2 / NDK 30 (`30.0.16248370`) were previously reported to compile `ruo_core` for `arm64-v8a`; that result means **compiles**, not SQLCipher works. Current `rusqlite` manifest resolves to 0.31.0 and enables `bundled` (plus defaults), not SQLCipher. The `bundled-sqlcipher-vendored-openssl` feature exists in 0.31.0 ([crate manifest](https://docs.rs/crate/rusqlite/0.31.0/source/Cargo.toml)). Latest stable rusqlite observed is 0.43.0 ([crates.io API](https://crates.io/api/v1/crates/rusqlite), checked 2026-11-25). Upgrade is not required to access that feature; compatibility of 0.43.0 with this Android build is untested. SQLCipher on Android remains **UNVERIFIED** pending the phone spike. |
| V6 | Versions: Kotlin, AGP, Gradle, Compose BOM, CameraX, ML Kit Document Scanner, Hilt, target SDK | maven.google.com, developer.android.com | Phase 1 | **Partially verified 2026-11-25**: Google Play requires new apps and updates to target API 36 from 2026-08-31 ([Android Developers](https://developer.android.com/google/play/requirements/target-sdk)). A compatible exact baseline is AGP 9.3.1 + Gradle 9.5.0 ([AGP 9.3 release notes](https://developer.android.com/build/releases/agp-9-3-0-release-notes)), Kotlin Gradle plugin 2.4.21 (fully supported with AGP 8.5.2–9.3.1 and Gradle 7.6.3–9.7.0; [Kotlin compatibility table](https://kotlinlang.org/docs/gradle-configure-project.html#apply-the-plugin)), and Compose BOM 2026.11.01 ([Google Maven metadata](https://dl.google.com/dl/android/maven2/androidx/compose/compose-bom/maven-metadata.xml)). Min SDK 26 is set by ADR-0001. CameraX, ML Kit Document Scanner, and Hilt versions remain **UNVERIFIED**. |
| V7 | Versions: Python, FastAPI, SQLAlchemy, Alembic, PyMuPDF, pdfplumber, python-docx, pgvector | pypi.org, project docs | Phase 1 / 3 | **Verified 2026-11-24**: Python 3.14.8, FastAPI 0.165.0, SQLAlchemy 2.1.9, Alembic 1.20.1, aiosqlite 0.22.1, greenlet 3.5.6 (source: local `pip list` / PyPI). |
| V8 | Current Play Console requirements (fee, card, ID, closed-test rule) | Play Console Help at registration | Before Phase 3 | Open (US$25 and 12 testers/14 days: verified) |
| V9 | UniFFI export syntax for the installed version | UniFFI docs for that version | Phase 1 | **Verified 2026-11-25**: In proc-macro-only mode, UniFFI 0.28's [procedural macro guide](https://mozilla.github.io/uniffi-rs/0.28/proc_macro/index.html#build-workflow) says to call `uniffi::setup_scaffolding!()` at the top of the crate source (for this crate, `src/lib.rs`), not in `build.rs`. Do not also call `uniffi::include_scaffolding!()`. |
| V10 | Email delivery service free tier | Provider docs | Phase 1 | Open. Dev uses `ConsoleEmailSender`. |
| V11 | Crash reporting free tier handling native Rust crashes | Crashlytics/Sentry docs | Phase 5 | Open |
| V12 | Font licenses and Android bundling (Plus Jakarta Sans, Inter, JetBrains Mono) | Each font's license | Phase 1 | Open |
| V13 | Philippine Data Privacy Act and Play minors policy implications (not legal advice) | Official texts; consult a professional if needed | Phase 5 | Open |
