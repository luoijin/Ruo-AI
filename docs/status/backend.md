# Backend Status

**Current Gate:** Gate 3B (backend sign-in and chat streaming)

**Owner approvals (dated):**
- ADR-0005 (PostgreSQL + pgvector, $0/month infra) accepted
- ADR-0008 (invite-only beta, Google + email-code sign-in) accepted
- Google OAuth clients configured; backend verifies GOOGLE_CLIENT_ID
- Email one-time code verification flow approved

**Open PRs:** Security-hardening PR (branch `agent/backend/phase1-hardening`)

**Progress:**
- Committed Stage A hardening: dev-only `test:` token guard, Gemini key moved to `x-goog-api-key` header, no raw provider bodies/network text in logs. Added tests for error hygiene, interrupted-generation regenerate, `parent_message_id`, dev OTP log, and header auth.
- Local evidence (host Windows, Python 3.14, Postgres 16.16 `ruo-db`, `ENVIRONMENT=test`): `20 passed, 85 warnings in 5.41s`; `ruff check .` = All checks passed; `ruff format --check .` = 28 files already formatted.
- CI evidence (PR #10, run 45807740119): backend job 139867328439 `20 passed in 4.60s`, `All checks passed!`, `28 files already formatted`; all 4 checks green.

**Blockers:** None

**Requests to others:** Lead: review contract-gaps reconciliation (task 2) scope against `PHASE-1-STATUS.md`.

**Next step:** task 2 (message `content` field + mid-stream retry semantics) once the hardening PR is up.