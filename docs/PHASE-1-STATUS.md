# Phase 1 Status

**Current Gate:** Gate 3B (Rust/Kotlin bridge on-device tests)

**IMPORTANT — REPO STATE correction (2026-11-26, Lead verified via git):**
- `origin/main` = `8da164c` (NOT `9da9b6c` as earlier stated). Local `main` = `9da9b6c`; they diverged. Local main is 2 commits behind origin/main.
- `ac7a8ad` (idempotent conversation create / stream retries, tip of `docs/gate3-interface-proposal`, PR #4): NOT an ancestor of local main or origin/main → PR #4 content NOT on main, not merged.
- `7d37f43` (conversation sync contract) and `2cdae61` (Gate 3B tests): NOT on local main or origin/main. Earlier claim "main contains these" is inaccurate.
- PR #5 recorded as MERGED but no merge commit touching `android/` found in the graph — verify on GitHub.

**Owner approvals (dated, quoted):**
- 2026-11-25: ADR-0009 (UniFFI 0.32.2) accepted
- 2026-11-25: ADR-0010 (SQLCipher Android build) accepted
- 2026-11-25: Gate 2 SQLCipher/UniFFI Android spike accepted
- 2026-11-25: Gate 3A interface proposal approved WITH conditions:
  - 7 must-fix: idempotent conversation create with client-generated id; message_id means client_msg_id; sync scope with keyset pagination and deletions; clear_session and sign-out policy; key-loss recovery and backup exclusion; error details never contain content/tokens/keys; enums for role/status
  - 4 should-fix: UniFFI defaults, https enforcement, one active stream per conversation, Phase 1 edits server-first
- 2026-11-25: `android.permission.INTERNET` approved
- 2026-11-25: One-time uninstall of `io.luoijin.ruo.debug` and its test package allowed on a signature mismatch
- 2026-11-26: Gate 3B on-device tests passed but the report was lost: recorded as "owner-reported, evidence pending"
- **Pending (Task 0):** Reviewer role addendum — roles table row, merge rule, review protocol. Now applied to docs/MULTI_AGENT.md and docs/status/reviewer.md (this PR).

**Roles:** Lead (docs, CI, merges), Reviewer (independent code/PR review, `docs/status/reviewer.md` only), Backend (`ai-service/`, `infra/`), Rust (`core-rust/`), Android (`android/`). No PR merges without an APPROVE verdict for its head commit.

**Open PRs:**
- PR #5 "feat(android): implement Phase 1 Rust bridge" — MERGED (per status; verify merger commit on GitHub)
- PR #6 (this PR, docs: add Reviewer role) — OPEN, awaiting CI + Reviewer APPROVE

**Reviewer queue:** none open

**Audit table (agent | branch | commits | verified where | UNVERIFIED | verdict):**
| Lead | agent/lead/status-docs | 45b3766 (3 unpushed docs commits) | forensics + secret scan run on host | origin/main divergence, PR #4/#5 merger on GitHub | in progress |
| Reviewer | agent/review/<topic> | none yet | n/a | n/a | ready |
| Backend | agent/backend/phase1-hardening | 0 commits; uncommitted edits to auth/service.py, providers/gemini.py, tests/test_backend_core.py (restored from C:\Github\backend-wip.patch) | not run (patch only, not committed/tests not executed) | Gemini `x-goog-api-key` header vs official docs | pending commit |
| Rust | agent/rust/phase1-interface | new branch at 6c1d2fe; earlier uncommitted work LOST | not run | interface_version()=1 claim, 24 tests (lost, not re-run) | blocked (freeze awaits owner go) |
| Android | agent/android/gate4-prep | b7332b5 (1 docs commit) | not run | gradle-9.5.0 download approval; no unapproved adb | planning mode only |

**Blockers board:**
- No PR merges without an APPROVE verdict for its head commit (Reviewer gate)
- Gate 3B device evidence lost — need to rerun instrumentation tests from PR #5 artifacts and record results
- PR #4 (docs/gate3-interface-proposal) content not on main; ac7a8ad not merged
- Contract-gaps reconciliation (message `content` field, mid-stream retry semantics) pending

**Gate 4A PLAN (docs only, waiting for owner approval):**
- (a) Stable debug signing key via a GitHub Actions secret: owner generates dedicated CI keystore outside repo with JDK keytool, registers SHA-1 for Android OAuth client `io.luoijin.ruo.debug`, base64-encodes keystore into secret; workflow decodes into `RUNNER_TEMP`, configures only `debug` build type, verifies fingerprint, deletes temp after staging
- (b) Google OAuth clients: Android client for `io.luoijin.ruo.debug` with debug SHA-1 `ac504953ca79d0824cb816985514d090be25ec38`; Web client whose ID backend verifies as `GOOGLE_CLIENT_ID`; under Play App Signing, release key fingerprint differs
- (c) CI speed-ups: cache Rust/NDK/OpenSSL and Gradle; skip Android job on docs-only changes
- (d) Local Gradle wrapper: `android/gradlew` exists; ask owner before downloading additional tools

**Next steps:**
- Owner to provide Reviewer addendum (Task 0)
- After addendum: create docs/status/reviewer.md, update MULTI_AGENT.md + PHASE-1-STATUS.md + AGENTS.md DoD, push branch, open docs PR, CI green, merge
- Owner to approve Gate 4A; then Android Compose screens can start
- Reconcile contract-gaps PR for message `content` field and mid-stream retry semantics
