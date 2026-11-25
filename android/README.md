# android/

- **Purpose:** Kotlin Android app and SQLCipher/UniFFI spike.
- **Audience:** Android engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-25
- **Related docs:** [ANDROID_CLIENT](../docs/ANDROID_CLIENT.md), [FOLDER_STRUCTURE](../docs/FOLDER_STRUCTURE.md)

The minimal `:app` and `:core:rustbridge` modules are a Gate 2 SQLCipher spike, not the Stage C app shell. The Android CI job builds the arm64 Rust library with SQLCipher and vendored OpenSSL, generates UniFFI Kotlin bindings, and assembles debug and instrumentation APKs. The instrumented checks still need to run on the owner's physical phone. See [ANDROID_CLIENT](../docs/ANDROID_CLIENT.md) and [RUST_CORE](../docs/RUST_CORE.md).
