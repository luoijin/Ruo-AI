# Backend Status

**Current Gate:** Gate 3B (backend sign-in and chat streaming)

**Owner approvals (dated):**
- ADR-0005 (PostgreSQL + pgvector, $0/month infra) accepted
- ADR-0008 (invite-only beta, Google + email-code sign-in) accepted
- Google OAuth clients configured; backend verifies GOOGLE_CLIENT_ID
- Email one-time code verification flow approved

**Open PRs:** PR #10 (auth/provider hardening), this branch -> contract-gap retry tests

**Progress:**
- Verified main already implements client-generated conversation IDs, idempotent create, keyset sync + tombstones, `parent_message_id`, mid-stream retry and failed-stream regeneration; no production code was missing for task 2/3/5.
- Added Postgres tests for the two untested mid-stream retry guarantees: completed replay has exactly `meta` + one full `delta` + `done`; two concurrent first sends create one pair and charge quota once.
- Local evidence (host Windows, Python 3.14, fresh Postgres 16.16 db `ruo_gate3_tests`, `ENVIRONMENT=test`): `17 passed, 74 warnings in 5.82s`; `ruff check .` = All checks passed; `ruff format --check .` = 28 files already formatted.

**Blockers:** None

**Requests to others:**
- Lead: edit `docs/API_CONTRACT.md` to match the Pydantic models (details in PR body): response messages expose `content_md`, not `content`; enumeration of `MessageItem` fields and all stable SSE error codes is missing; replayed `done` omits `usage`; list endpoint defaults are not stated.
- Lead/Rust/Android: document the backend-to-client error mapping `RATE_LIMIT_EXCEEDED` -> `RATE_LIMITED` (daily quota `QUOTA_EXCEEDED` stays `QUOTA_EXCEEDED`). Existing `RUST_CORE.md` lists both codes but not the mapping.

**Next step:** latency investigation (task 7) after this PR is reviewed.