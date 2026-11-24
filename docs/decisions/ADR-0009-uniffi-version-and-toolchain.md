# ADR-0009: UniFFI 0.28.3 baseline and toolchain for Android bridge

- **Purpose:** Freeze the UniFFI version and build toolchain baseline prior to the SQLCipher Android spike.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-24
- **Related docs:** [ADR guide](README.md), [AGENTS.md](../../AGENTS.md), [RUST_CORE](../RUST_CORE.md), [VERIFY_BEFORE_BUILD](../VERIFY_BEFORE_BUILD.md)

**ADR status:** Accepted

## Context
UniFFI binds the headless Rust core (`ruo_core`) to Kotlin on Android. Between UniFFI 0.28.3 (released November 2024) and 0.32.2 (released September 2026), several breaking changes occurred, including deprecation and removal of UDL files, changes to proc-macro attribute scopes, and modifications to `uniffi-bindgen` CLI arguments.

We need a proven, stable version that:
1. Compiles with `cargo-ndk 4.1.2` and Android NDK 30 (`aarch64-linux-android`) without toolchain friction.
2. Supports proc-macro-based exports (`#[uniffi::export]`, `#[uniffi::export(with_foreign)]`) for traits and objects.
3. Does not churn bindings or introduce unstable dependencies during the upcoming SQLCipher Android spike.

## Decision
Freeze UniFFI at version **0.28.3** as the baseline for the Android bridge.
- Build tool: `cargo ndk -t arm64-v8a -P 26 build`
- Kotlin binding generation: via internal generator or `uniffi = { version = "0.28", features = ["cli"] }` (`uniffi::uniffi_bindgen_main()`)
- Any upgrade to 0.32.x is deferred until after the SQLCipher on-device spike demonstrates need.

## Consequences
- Proven stability: `ruo_core` compiles cleanly under NDK 30 for `arm64-v8a` in 32 seconds.
- Avoids transitive dependency churn across the workspace during initial Android integration.
- Does not use UDL files; proc-macro only export syntax is standard.
