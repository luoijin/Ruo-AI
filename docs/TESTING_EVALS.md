# Testing and Evals

- **Purpose:** Define the test pyramid per layer, FFI and load tests, the evaluation suite, feedback-to-eval workflow and latency targets.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [AI_PIPELINE](AI_PIPELINE.md), [RUST_CORE](RUST_CORE.md), [PRODUCT](PRODUCT.md)

## Test pyramid [v1]
| Layer | Tests |
|---|---|
| Rust | Unit: SSE framing, reconnect backoff, outbox transitions, DB migrations, upload resume |
| FFI boundary | Instrumented Android: Taglish, Cebuano and emoji strings through bindings; cancel streams mid-flight; error propagation |
| Backend | pytest + async HTTP client: auth and token rotation, quota enforcement, file-type rejection, injection delimiter escaping, mocked provider streaming, real PDF and DOCX fixtures |
| UI | Compose tests: attachment tray, voice review-before-send, study-mode button states, offline queued badge |
| Load | k6 or Locust: concurrent SSE streams survive on a small-memory instance without out-of-memory |

## Evaluation suite [v1]
Start with about **20** multilingual prompts; grow toward **120** (English, Taglish, Bisaya-English, cross-language file questions), plus real voice clips for transcription checks. Stored in `ai-service/evals/`.
| Category | Measures |
|---|---|
| Extraction accuracy | Scans, multi-column PDFs, tables, handwriting |
| Language adherence | Taglish, Bisaya-English, explicit overrides, cross-language files |
| Study-mode compliance | Steps 1 and 2 never reveal the answer (automated, blocking in CI) |
| Voice transcription | Error rates on real recordings |
| Injection resistance | Fixture documents with hostile instructions |

## Feedback-to-eval workflow [v1]
Thumbs-down feedback (stored with reason code). With the user's consent, a reviewed thumbs-down becomes a new regression case. The owner reviews and strips personal data before adding.

## Latency targets (warm; to be measured, not promises) [v1]
Text first token under about 1 s · image question under about 1.5 s · 10-page PDF parsed in about 4 s · 15 s of audio transcribed in about 2 s. Free hosting may miss these: measure early and record results here.

## Rules
Every change ships with tests. Real fixtures over mocks where cheap. CI runs Rust, backend and Android unit tests; instrumented and load tests run before each release.
