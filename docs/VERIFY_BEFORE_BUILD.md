# Verify Before Build

- **Purpose:** Checklist of every UNVERIFIED item, how to verify it, and its status.
- **Audience:** Owner and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [TECH_STACK](TECH_STACK.md), [OPEN_QUESTIONS](OPEN_QUESTIONS.md)

Update the Status column (Open / Verified + date + source). Never record a fact without a source.

| # | Item | How to verify | Needed by | Status |
|---|---|---|---|---|
| V1 | Is the main Gemini model on the free tier; actual per-minute and per-day limits for each model used | Google AI Studio rate-limit page | Phase 1 | Open |
| V2 | Exact model names: chat, lighter fallback, transcription, embeddings | AI Studio model list | Phase 1 / 3 / 4 | Open (gemini-4-hydrogen exists: verified) |
| V3 | Embedding dimension | Model docs; confirm before creating `document_chunks` | Phase 3 | Open |
| V4 | Free hosting and Postgres terms: card required, idle sleep, RAM/CPU, storage | Each provider's current pricing and terms pages | Phase 3 | Open |
| V5 | Versions: Rust, UniFFI, cargo-ndk, SQLCipher binding | crates.io, project READMEs | Phase 1 | Open |
| V6 | Versions: Kotlin, AGP, Gradle, Compose BOM, CameraX, ML Kit Document Scanner, Hilt, target SDK | maven.google.com, developer.android.com | Phase 1 | Open |
| V7 | Versions: Python, FastAPI, SQLAlchemy, Alembic, PyMuPDF, pdfplumber, python-docx, pgvector | pypi.org, project docs | Phase 1 / 3 | Open |
| V8 | Current Play Console requirements (fee, card, ID, closed-test rule) | Play Console Help at registration | Before Phase 3 | Open (US$25 and 12 testers/14 days: verified) |
| V9 | UniFFI export syntax for the installed version | UniFFI docs for that version | Phase 1 | Open |
| V10 | Email delivery service free tier | Provider docs | Phase 1 | Open |
| V11 | Crash reporting free tier handling native Rust crashes | Crashlytics/Sentry docs | Phase 5 | Open |
| V12 | Font licenses and Android bundling (Plus Jakarta Sans, Inter, JetBrains Mono) | Each font's license | Phase 1 | Open |
| V13 | Philippine Data Privacy Act and Play minors policy implications (not legal advice) | Official texts; consult a professional if needed | Phase 5 | Open |
