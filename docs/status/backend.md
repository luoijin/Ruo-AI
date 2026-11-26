# Backend Status

**Current Gate:** Gate 3B (backend sign-in and chat streaming)

**Owner approvals (dated):**
- ADR-0005 (PostgreSQL + pgvector, $0/month infra) accepted
- ADR-0008 (invite-only beta, Google + email-code sign-in) accepted
- Google OAuth clients configured; backend verifies GOOGLE_CLIENT_ID
- Email one-time code verification flow approved

**Open PRs:** PR #12 (branch `agent/backend/gate3-retry-tests-v2`; PR #11 closed as stale); PR #10 hardening MERGED 2026-11-26

**Progress:**
- Committed Stage A hardening: dev-only `test:` token guard, Gemini key moved to `x-goog-api-key` header, no raw provider bodies/network text in logs. Added tests for error hygiene, interrupted-generation regenerate, `parent_message_id`, dev OTP log, and header auth.
- Local evidence (host Windows, Python 3.14, Postgres 16.16 `ruo-db`, `ENVIRONMENT=test`): `20 passed, 85 warnings in 5.41s`; `ruff check .` = All checks passed; `ruff format --check .` = 28 files already formatted.
- CI evidence (PR #10, run 45807740119): backend job 139867328439 `20 passed in 4.60s`, `All checks passed!`, `28 files already formatted`; all 4 checks green.

**Blockers:** None

**Progress (task 2):**
- Verified main already implements client-generated conversation IDs, idempotent create, keyset sync + tombstones, `parent_message_id`, mid-stream retry and failed-stream regeneration; no production code was missing.
- Added Postgres tests for the two untested guarantees: completed replay is exactly `meta` + one full `delta` + `done`; two concurrent first sends create one pair and charge quota once.
- Local evidence (host Windows, Python 3.14, Postgres 16.16 db `ruo_gate3_tests`, `ENVIRONMENT=test`): `22 passed, 105 warnings in 6.93s`; `ruff check .` = All checks passed; `ruff format --check .` = 28 files already formatted.
- CI evidence (PR #12, run 45828281894): backend job 139938036265 `22 passed in 5.37s`, `All checks passed!`, `28 files already formatted`; all 4 checks green.

**Requests to others:**
- Lead: edit `docs/API_CONTRACT.md` to match the Pydantic models (full mismatch list in PR #11): message responses expose `content_md`, not `content`; `MessageItem` fields and stable SSE error codes are not enumerated; replayed `done` omits `usage`; list defaults are not stated.
- Lead/Rust/Android: document the backend-to-client error mapping `RATE_LIMIT_EXCEEDED` -> `RATE_LIMITED` (daily quota `QUOTA_EXCEEDED` stays `QUOTA_EXCEEDED`).

**Next step:** latency investigation (task 7) after PR #11 is reviewed.