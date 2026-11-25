# Lead Status

**Current Gate:** Gate 3B (Rust/Kotlin bridge on-device tests) — in progress

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

**Open PRs:** PR #5 "feat(android): implement Phase 1 Rust bridge" — MERGED

**Blockers board:**
- Gate 3B device evidence lost — REQUEST to Android: rerun instrumentation tests from PR #5 artifacts and record results (evidence pending)

**Gate 4A PLAN (docs only, waiting for owner approval):**
- (a) Stable debug signing key via GitHub Actions secret: owner generates dedicated CI keystore outside repo with JDK keytool, registers SHA-1 for Android OAuth client `io.luoijin.ruo.debug`, base64-encodes keystore into secret; workflow decodes into `RUNNER_TEMP`, configures only `debug` build type, verifies fingerprint, deletes temp after staging
- (b) Google OAuth clients: Android client for `io.luoijin.ruo.debug` with debug SHA-1; Web client whose ID backend verifies as `GOOGLE_CLIENT_ID`; under Play App Signing, release key fingerprint differs
- (c) CI speed-ups: cache Rust/NDK/OpenSSL and Gradle; skip Android job on docs-only changes
- (d) Local Gradle wrapper: `android/gradlew` exists; Android agent can run `./gradlew :app:assembleDebug` locally. Ask owner before downloading additional tools.

**Next steps:**
- Owner to approve Gate 4A
- After Gate 4A: Android Compose screens can start
- Reconcile contract-gaps PR for message `content` field and mid-stream retry semantics