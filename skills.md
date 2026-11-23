# skills.md: Agent playbooks

- **Purpose:** Domain playbooks to consult before touching an area.
- **Audience:** AI coding agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [AGENTS.md](AGENTS.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [CONTRIBUTING.md](CONTRIBUTING.md)

Each playbook: **When**, **Conventions**, **Recipe**, **Pitfalls**, **Verify**. All versions and API syntax are UNVERIFIED until checked against official docs.

## 1. Android/Compose (MVI screens, navigation)
- **When:** adding or changing a screen. Docs: [ANDROID_CLIENT](docs/ANDROID_CLIENT.md).
- **Conventions:** `UiState` immutable; sealed `UiIntent`; `UiEffect` for one-offs; type-safe routes; strings in resources (English, Filipino).
- **Recipe:** 1) add route; 2) define state/intent/effect; 3) ViewModel reduces intents, collects Rust-bridged Flows; 4) stateless Composable; 5) strings in `values/` and `values-fil/`; 6) Compose test.
- **Pitfalls:** state in Composables; hard-coded strings; UI work on callback threads; no content descriptions.
- **Verify:** Compose test passes; TalkBack labels; font scale 200%.

## 2. CameraX and ML Kit scanner
- **When:** capture, gallery, scanner. **Conventions:** Photo Picker, no storage permission; camera permission just in time.
- **Recipe:** capture/scan → resize long edge about 2048 px off main thread → strip location metadata → copy to sandbox → hand path to Rust.
- **Pitfalls:** passing content URIs to Rust; skipping EXIF strip; main-thread bitmap work.
- **Verify:** test image has no GPS tags; 4 GB phone does not OOM.

## 3. Audio and voice review flow
- **When:** recording or transcription. **Conventions:** 16 kHz mono, record to file; microphone just in time.
- **Recipe:** record → upload → `/audio/transcribe` → fill composer → banner if low confidence → user edits and sends.
- **Pitfalls:** auto-send (forbidden, [ADR-0006](docs/decisions/ADR-0006-review-before-send-voice.md)); buffering audio in memory.
- **Verify:** Compose test that Send requires a user tap; low-confidence banner test.

## 4. Rust core and UniFFI
- **When:** any exported function or callback. Docs: [RUST_CORE](docs/RUST_CORE.md).
- **Conventions:** core is an object with a constructor; callbacks are `with_foreign` traits; headless.
- **Recipe (add exported fn):** 1) implement in module; 2) export on `RuoCore`; 3) update docs and API/types; 4) rebuild `.so` with cargo-ndk; 5) regenerate bindings; 6) wrap in `:core:rustbridge` with dispatcher hop; 7) tests both sides.
- **Pitfalls:** panics across FFI; blocking main thread; stale bindings; interface churn (freeze early).
- **Verify:** `cargo test`; FFI test with Taglish/emoji; cancel mid-stream.

## 5. SQLCipher and migrations
- **When:** local schema changes. **Conventions:** versioned migrations run on open; key from Keystore passed as bytes.
- **Recipe:** add migration N+1; never edit old ones; test upgrade from every prior version with data.
- **Pitfalls:** key loss on update; OpenSSL build failures; non-idempotent migrations.
- **Verify:** migration tests; app update on a device keeps history.

## 6. FastAPI modules and routers
- **When:** new endpoint or module. Docs: [API_CONTRACT](docs/API_CONTRACT.md).
- **Recipe:** module has `router.py`, `service.py`, `schemas.py`, `models.py`; update the contract doc first; Alembic migration if schema changes; problem-details errors; tests.
- **Pitfalls:** business logic in routers; modules importing each other's internals; logging content or tokens.
- **Verify:** pytest; OpenAPI matches doc.

## 7. SSE streaming
- **When:** message or attachment-status streams.
- **Conventions:** events meta, delta, citation, done, error; idempotent by `client_msg_id`.
- **Recipe:** server yields events; client parses frames, reconnects with backoff, cancels on screen exit.
- **Pitfalls:** buffering proxies; unknown events must be ignored; duplicate messages on retry.
- **Verify:** framing and reconnect tests; kill-network test on a phone.

## 8. ModelProvider adapters
- **When:** provider changes. Docs: [AI_PIPELINE](docs/AI_PIPELINE.md).
- **Recipe:** implement the interface; keep model names in config; map provider errors (429) to retry/fallback; add fake provider cases.
- **Pitfalls:** SDK types leaking into business code; hard-coded model names; unverified limits.
- **Verify:** mocked streaming test; fallback test sets `fallback` flag.

## 9. Document extraction and RAG
- **When:** attachments pipeline.
- **Recipe:** header-check type → size/page caps → job → extract → OCR only scanned pages → store Markdown with page markers → under about 12,000 tokens send full, else chunk (about 500 tokens, overlap), embed, hybrid retrieve → purge raw.
- **Pitfalls:** parsing on the request thread; skipping the isolation wrapper; keeping raw files; embedding dimension mismatch.
- **Verify:** real fixtures; spoofed-extension and zip-bomb tests; retrieval test.

## 10. Multilingual and code-switching
- **When:** language policy work.
- **Recipe:** detect per message; model call only for short, mixed or low-confidence text; reply in user's language and register; honor explicit and global override; cross-language document rule.
- **Pitfalls:** per-conversation detection; claiming quality for Tier 2/3 languages.
- **Verify:** language-adherence evals (Taglish, Bisaya-English, overrides).

## 11. Study mode prompts
- **When:** hint ladder or check-my-answer. Docs: [AI_PIPELINE](docs/AI_PIPELINE.md).
- **Recipe:** structured output with step and final-answer flag; steps 1-2 never include the final answer; step 3 only on request or after two misses.
- **Pitfalls:** answer leakage via worked examples; no automated check.
- **Verify:** leak-test evals pass in CI.

## 12. Security review checklist
Run on any change touching auth, uploads, prompts, storage or logging.
- [ ] No secrets in code, logs or app
- [ ] Provider key only on backend
- [ ] Untrusted text passes through the isolation wrapper
- [ ] Upload type by header; caps enforced; raw purge in place
- [ ] Quotas and invite gate enforced server-side
- [ ] Tokens hashed/rotated; no cleartext traffic in release
- [ ] No message content in logs
- [ ] Deletion cascades cover any new table
- [ ] Docs: [SECURITY_PRIVACY](docs/SECURITY_PRIVACY.md) updated

## 13. Writing tests and evals
- **Recipe:** test next to the code; backend uses pytest with real fixtures; Rust unit tests per module; FFI and Compose tests for boundary and UI; add eval cases in `ai-service/evals/` (grow from about 20 toward 120). Docs: [TESTING_EVALS](docs/TESTING_EVALS.md).
- **Pitfalls:** tests that mock the thing being tested; evals without expected criteria.

## 14. Writing ADRs
See [docs/decisions/README.md](docs/decisions/README.md). Copy an existing ADR; supersede, never rewrite.

## 15. Updating the CHANGELOG
Add an entry under `[Unreleased]` in the right category ([CHANGELOG](CHANGELOG.md)) in the same change. User-visible wording, one line each.

## 16. Verifying facts
Before using a version, model name, quota, price or policy: check the official source, record it in [VERIFY_BEFORE_BUILD](docs/VERIFY_BEFORE_BUILD.md) with date and source. If you cannot check, write **UNVERIFIED**.

## Splitting into per-skill folders later
If your agent tool supports `SKILL.md`, create `skills/<name>/SKILL.md` per playbook (front matter with name and a trigger description), and leave this file as an index.
