# Phase 2: Vision and share

- **Purpose:** Plan and gate Phase 2: Vision and share.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ROADMAP](../../ROADMAP.md), [PRODUCT](../PRODUCT.md), [AGENTS.md](../../AGENTS.md), [skills.md](../../skills.md)

## Goal
Let users bring images and shared content into chat. Effort: medium.

## Features covered [v1]
Features 2 (camera and gallery, multi-image), 8 (share to Ruo), 9 (scanner cleanup), 11 (quick chips, image side).

## Ordered tasks
1. Attachment tables and `/attachments` init/chunk endpoints (chunks temporarily in Postgres, purge within 24 h). [v1]
2. Rust resumable uploader and hashing, with `UploadProgress`. [v1]
3. CameraX capture; image prep off the main thread: resize long edge about 2048 px, strip location metadata. [v1]
4. ML Kit Document Scanner integration. [v1]
5. Blurry-photo warning before upload. [v1]
6. Multi-image composer tray (up to 4 images), progress ring. [v1]
7. Backend sends images directly to the vision model via `ModelProvider`. [v1]
8. Share intake: copy to sandbox immediately; New chat or Add to recent chat. [v1]
9. Quick action chips and structured `intent` (Summarize, Explain, Translate, Solve). [v1]
10. Extend evals with image prompts.

## Dependencies
Phase 1 complete.

## Acceptance criteria
- Photograph a worksheet and get an answer.
- Interrupted upload resumes.
- Shared image/text from another app lands in a chat.
- No location metadata in uploaded images (verified by test).

## Test requirements
Rust upload-resume tests; backend tests for chunk assembly, size caps, header type check; Compose tests for tray; manual share-intent test on a phone.

## Agent notes (pitfalls)
- Content URIs never cross into Rust: Kotlin copies to sandbox and passes a file path.
- Request only the camera permission, just in time; use the Photo Picker.
- If chunked uploads in Postgres prove too heavy, fall back to a single retryable upload under the cap (ADR).

## Definition of done
- [ ] Acceptance criteria met and demonstrated on a physical phone
- [ ] Tests listed above passing in CI
- [ ] Docs, API_CONTRACT/DATA_MODEL (if changed) and CHANGELOG updated
- [ ] No unverified fact stated as fact; VERIFY_BEFORE_BUILD statuses updated
- [ ] Security checklist in [skills.md](../../skills.md) applied
