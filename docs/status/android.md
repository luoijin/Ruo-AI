# Android Status

**Current Gate:** Gate 3B (on-device tests for Rust/Kotlin bridge)

**Owner approvals (dated):**
- ADR-0001 (Android only, minSdk 26) accepted
- ADR-0002 (Kotlin + Compose; headless Rust core) accepted
- Gate 2 SQLCipher/UniFFI Android spike accepted
- Gate 3A interface proposal accepted with 7 must-fix and 4 should-fix conditions
- `android.permission.INTERNET` approved
- One-time uninstall of `io.luoijin.ruo.debug` and its test package allowed on a signature mismatch
- Gate 3B on-device tests passed but the report was lost: recorded as "owner-reported, evidence pending"

**Open PRs:** PR #5 "feat(android): implement Phase 1 Rust bridge" (merged)

**Blockers:** Gate 3B device evidence lost - need to rerun instrumentation tests from PR #5 artifacts

**Requests to others:** Re-run instrumentation tests from PR #5 artifacts and record results