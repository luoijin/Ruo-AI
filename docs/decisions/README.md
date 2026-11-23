# Architecture Decision Records

- **Purpose:** Explain when and how to write an ADR.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [AGENTS.md](../../AGENTS.md), [FOLDER_STRUCTURE](../FOLDER_STRUCTURE.md)

## Index
- [ADR-0001](ADR-0001-android-only.md) Android only
- [ADR-0002](ADR-0002-rust-uniffi-client-core.md) Rust core via UniFFI
- [ADR-0003](ADR-0003-fastapi-only-backend.md) FastAPI-only backend; Axum deferred
- [ADR-0004](ADR-0004-single-gemini-provider.md) Single Gemini provider
- [ADR-0005](ADR-0005-zero-cost-infrastructure.md) $0 infrastructure
- [ADR-0006](ADR-0006-review-before-send-voice.md) Review-before-send voice
- [ADR-0007](ADR-0007-postgres-queue-and-quotas.md) Postgres queue and quotas
- [ADR-0008](ADR-0008-invite-only-beta.md) Invite-only beta

## Write a new ADR when
- Changing or reopening a locked decision (needs owner approval).
- Choosing a library or service with lasting impact (e.g. the working SQLCipher/OpenSSL build config).
- Changing the API contract shape, data model approach, or a security control.
- Adding a permission, paid service, or new data category sent to the provider.

## Don't write one for
Bug fixes, routine refactors, or choices easily reversed in one PR.

## How
Copy an existing ADR. Name `ADR-NNNN-slug.md` (next number). Sections: Context, Decision, Consequences, ADR status (Proposed, Accepted, Superseded by ADR-NNNN). Never edit an accepted ADR's decision; supersede it. Update this index and [CHANGELOG](../../CHANGELOG.md).
