# android/

- **Purpose:** Kotlin Android app and SQLCipher/UniFFI spike.
- **Audience:** Android engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-25
- **Related docs:** [ANDROID_CLIENT](../docs/ANDROID_CLIENT.md), [FOLDER_STRUCTURE](../docs/FOLDER_STRUCTURE.md)

The minimal `:app` and `:core:rustbridge` modules are a Gate 2 SQLCipher spike, not the Stage C app shell. CI run [45588157467](https://github.com/luoijin/Ruo-AI/actions/runs/45588157467) compiled the arm64 Rust library with SQLCipher and vendored OpenSSL, generated UniFFI Kotlin bindings, and assembled both APKs. The instrumented SQLCipher checks remain unrun because the physical phone disconnected before installation. See [ANDROID_CLIENT](../docs/ANDROID_CLIENT.md) and [RUST_CORE](../docs/RUST_CORE.md).
