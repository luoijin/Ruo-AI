# Phase 4: Voice, languages and study mode

- **Purpose:** Plan and gate Phase 4: Voice, languages and study mode.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ROADMAP](../../ROADMAP.md), [PRODUCT](../PRODUCT.md), [AGENTS.md](../../AGENTS.md), [skills.md](../../skills.md)

## Goal
Add review-before-send voice, language policy, TTS, hint ladder and answer checking. Effort: medium.

## Features covered [v1]
Features 4 (voice), 5 (multilingual), 6 (study mode), 7 (TTS).

## Ordered tasks
1. Record 16 kHz mono audio to a file; upload; `/audio/transcribe` with multilingual prompt; confidence. [v1]
2. Composer fills with transcript; low-confidence banner; never auto-send. [v1]
3. Per-message language detection; active language on conversation; explicit and global overrides. [v1]
4. Cross-language document rule; support tiers shown honestly in app. [v1]
5. Study mode toggle and hint ladder with structured output; `study_step` stored. [v1]
6. Check my answer flow. [v1]
7. Automated check that steps 1-2 never leak final answers (blocking). [v1]
8. Android built-in TTS reading replies; label Bisaya voice availability honestly. [v1]
9. Filipino UI strings completed. [v1]
10. Grow evals toward 120; collect real voice clips.

## Dependencies
Phase 3; transcription model name verified.

## Acceptance criteria
- Taglish voice question appears in the composer for editing.
- Reply language follows the user; overrides work.
- Study steps 1-2 pass leak tests.
- TTS reads a reply.

## Test requirements
Compose test for voice review flow; backend tests for language policy; eval runs for adherence, study compliance, transcription; FFI Taglish/Cebuano strings.

## Agent notes (pitfalls)
- Never auto-send transcripts.
- Detect per message, not per conversation.
- Microphone permission just in time.
- Do not claim TTS quality for Bisaya if the device voice is missing.

## Definition of done
- [ ] Acceptance criteria met and demonstrated on a physical phone
- [ ] Tests listed above passing in CI
- [ ] Docs, API_CONTRACT/DATA_MODEL (if changed) and CHANGELOG updated
- [ ] No unverified fact stated as fact; VERIFY_BEFORE_BUILD statuses updated
- [ ] Security checklist in [skills.md](../../skills.md) applied
