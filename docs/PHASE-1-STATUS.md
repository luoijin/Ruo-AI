# Phase 1 Status

**Current Gate:** Gate 3B (Rust/Kotlin bridge on-device tests)

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

**Open PRs:**
- PR #5 "feat(android): implement Phase 1 Rust bridge" — MERGED

**Blockers board:**
- Gate 3B device evidence lost — need to rerun instrumentation tests from PR #5 artifacts and record results
- REQUEST to Android: rerun instrumentation tests from PR #5 artifacts and record results (evidence pending)

**Gate 4A PLAN (docs only, waiting for owner approval):**
- (a) Stable debug signing key via a GitHub Actions secret: the owner must generate a dedicated CI keystore outside the repo with JDK `keytool`, register the SHA-1 fingerprint for the Android OAuth client package `io.luoijin.ruo.debug`, and base64-encode the keystore into a GitHub Actions secret. The workflow will decode the keystore into `RUNNER_TEMP`, configure only the `debug` build type to use it, verify the APK certificate fingerprint, and delete the temporary keystore after staging artifacts. Release signing stays separate.
- (b) Current official Google OAuth clients: Android client for `io.luoijin.ruo.debug` with the debug SHA-1; a Web client whose ID the backend verifies as `GOOGLE_CLIENT_ID`. Under Play App Signing, the release key fingerprint differs and a different OAuth client configuration is needed.
- (c) CI speed-ups: cache Rust/NDK/OpenSSL and Gradle actions; skip the Android job on docs-only changes. Cite current official caching actions (actions/cache forrust-toolchain, ndk, gradle).
- (d) A local Gradle wrapper path for the Android agent: the `gradlew` wrapper is checked into the repo at `android/gradlew`; the Android agent can run `./gradlew :app:assembleDebug` locally. Ask the owner before downloading any additional tools.

**Next steps:**
- Owner to approve Gate 4A (stable debug signing key via GitHub Actions secret)
- After Gate 4A approval: Android screens (Compose) can start
- Reconcile contract-gaps PR for message `content` field and mid-stream retry semantics
