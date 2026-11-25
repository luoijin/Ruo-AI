# Lead Status

**Current Gate:** Gate 3B (Rust/Kotlin bridge on-device tests)

**Owner approvals (dated):**
- ADR-0009 (UniFFI 0.32.2) accepted
- ADR-0010 (SQLCipher Android build) accepted
- Gate 2 SQLCipher/UniFFI Android spike accepted
- Gate 3A interface proposal approved WITH conditions:
  - 7 must-fix: idempotent conversation create with client-generated id; message_id means client_msg_id; sync scope with keyset pagination and deletions; clear_session and sign-out policy; key-loss recovery and backup exclusion; error details never contain content/tokens/keys; enums for role/status
  - 4 should-fix: UniFFI defaults, https enforcement, one active stream per conversation, Phase 1 edits server-first
- `android.permission.INTERNET` approved
- One-time uninstall of `io.luoijin.ruo.debug` and its test package allowed on a signature mismatch
- Gate 3B on-device tests passed but the report was lost: recorded as "owner-reported, evidence pending"

**Open PRs:** PR #5 "feat(android): implement Phase 1 Rust bridge" (merged)

**Blockers:** Gate 3B device evidence lost - need to rerun instrumentation tests from PR #5 artifacts

**Requests to others:** Android - rerun instrumentation tests from PR #5 artifacts and record results (evidence pending)