# Phase 5: Hardening and release

- **Purpose:** Plan and gate Phase 5: Hardening and release.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-25
- **Related docs:** [ROADMAP](../../ROADMAP.md), [PRODUCT](../PRODUCT.md), [AGENTS.md](../../AGENTS.md), [skills.md](../../skills.md)

## Goal
Make Ruo release-ready and reach production on Google Play. Effort: medium.

## Features covered [v1]
Features 12 (feedback), 13 (deletion, export), 16 (outbox sync), 17 (settings), plus accessibility, privacy and release.

## Ordered tasks
1. Outbox sync with WorkManager flush; idempotent replay; Queued badge. [v1]
2. Feedback buttons and pipeline into evals (consent). [v1]
3. Settings: theme, UI language, response language, privacy controls. [v1]
4. Account deletion (in app and web URL) and data export. [v1]
5. Decide and implement minimum age and consent ([OPEN_QUESTIONS](../OPEN_QUESTIONS.md) Q3). [v1]
6. Accessibility pass (touch targets, descriptions, font scaling to 200%, contrast). [v1]
7. Crash reporting incl. native Rust crashes. [v1]
8. Privacy policy, Data Safety form, store listing, signed builds. [v1]
9. Production access application after the 14-day closed test; staged rollout 10% → 50% → 100%. [v1]
10. README demo video and portfolio packaging.
11. Package third-party license texts and required attribution notices for SQLCipher, OpenSSL, and their bundled source crates.

## Dependencies
Phase 4; closed test completed (14 continuous days, 12+ testers); minors decision made.

## Acceptance criteria
- Queued offline message syncs once, without duplicates.
- Deletion removes all user data; export works.
- Accessibility checklist passed.
- Production access applied for; staged rollout started.

## Test requirements
Outbox state-transition and replay tests; deletion cascade test; Compose a11y tests; instrumented tests on a 4 GB RAM phone; load test before rollout.

## Agent notes (pitfalls)
- Deletion must be real and cascade through chunks and embeddings.
- Keep the invite gate enforced in production.
- Define halt criteria for the staged rollout before starting it.

## Definition of done
- [ ] Acceptance criteria met and demonstrated on a physical phone
- [ ] Tests listed above passing in CI
- [ ] Docs, API_CONTRACT/DATA_MODEL (if changed) and CHANGELOG updated
- [ ] No unverified fact stated as fact; VERIFY_BEFORE_BUILD statuses updated
- [ ] Security checklist in [skills.md](../../skills.md) applied
