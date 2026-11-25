# Roadmap

- **Purpose:** Define the five v1 phases: goals, deliverables, exit criteria, relative effort.
- **Audience:** Owner and agents.
- **Status:** Draft
- **Last updated:** 2026-11-25
- **Related docs:** [Phase 1](docs/phases/PHASE-1-core-spine.md), [docs/RELEASE_PLAYBOOK.md](docs/RELEASE_PLAYBOOK.md)

Effort is relative for a solo developer (no dates). Every phase ends release-quality with tests; nothing is throwaway. Feature numbers follow [PRODUCT](docs/PRODUCT.md).

| Phase | Effort | Goal | Features |
|---|---|---|---|
| [1 Core spine](docs/phases/PHASE-1-core-spine.md) | Large | Sign in, stream replies, read cached chats offline | 1, 13 (sign-in), 14, 15 (moderation basics), 16 (reading), 17 (theme) |
| [2 Vision and share](docs/phases/PHASE-2-vision-share.md) | Medium | Images in, share intake | 2, 8, 9, 11 |
| [3 Documents and citations](docs/phases/PHASE-3-documents-citations.md) | Medium | Extraction, retrieval, citations, injection defense | 3, 10, 15 (injection defense) |
| [4 Voice, languages, study](docs/phases/PHASE-4-voice-languages-study.md) | Medium | Language policy, review-before-send voice, TTS, hint ladder | 4, 5, 6, 7 |
| [5 Hardening and release](docs/phases/PHASE-5-hardening-release.md) | Medium | Outbox sync, offline edits/conflict resolution, feedback, settings, deletion/export, a11y, Play production | 12, 13 (delete/export), 16 (outbox), 17 |

## Deliverables and exit criteria
- **Phase 1** [v1]: monorepo and CI, UniFFI bridge, SQLCipher, SSE client, FastAPI auth/chat/quotas, offline reading. *Exit:* sign in, stream a reply, read cached chats offline.
- **Phase 2** [v1]: scanner, camera, multi-image composer, resumable upload, share intake, quick chips. *Exit:* photograph a worksheet, ask a question, get an answer.
- **Phase 3** [v1]: extraction worker, hybrid retrieval, citations, injection/spoofing defenses. *Exit:* ask about a 50-page PDF and tap a citation to the page. **Start the Play closed test after this phase** ([RELEASE_PLAYBOOK](docs/RELEASE_PLAYBOOK.md)).
- **Phase 4** [v1]: language policy, transcription with review, Android TTS, hint ladder, answer checking. *Exit:* Taglish voice question in study mode; steps 1-2 never leak answers (automated check).
- **Phase 5** [v1]: outbox sync, feedback pipeline, settings, delete/export, accessibility pass, production access application, staged rollout. *Exit:* production access applied for; staged rollout started.

## Cross-phase tracks
Verify items in [VERIFY_BEFORE_BUILD](docs/VERIFY_BEFORE_BUILD.md) before the phase that needs them. Grow evals from about 20 prompts toward 120 ([TESTING_EVALS](docs/TESTING_EVALS.md)).
