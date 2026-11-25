# ADR-0009: UniFFI version and toolchain for Android bridge

- **Purpose:** Propose the UniFFI baseline for the Android bridge before the interface is frozen.
- **Audience:** Engineers and agents.
- **Status:** Accepted
- **Last updated:** 2026-11-25
- **Related docs:** [ADR guide](README.md), [AGENTS.md](../../AGENTS.md), [RUST_CORE](../RUST_CORE.md), [VERIFY_BEFORE_BUILD](../VERIFY_BEFORE_BUILD.md)

**ADR status:** Accepted 2026-11-25; use 0.32.2 unless a concrete blocker is proven and documented.

## Context

The official [UniFFI changelog](https://github.com/mozilla/uniffi-rs/blob/v0.32.2/CHANGELOG.md) dates UniFFI 0.28.3 to 2024-11-08 and 0.32.2 to 2026-09-22. The [crates.io package record](https://crates.io/api/v1/crates/uniffi), checked 2026-11-25, reports 0.32.2 as the latest stable version.

The earlier claim that UDL files were deprecated and removed between those releases is incorrect. UniFFI 0.31 changed `uniffi-bindgen` to auto-detect a library source versus a UDL file; 0.32 still documents UDL-specific changes, including `[ByRef]` arguments. The changelog does not say UDL files were removed.

Changes relevant to this Android bridge:

- UniFFI 0.30 changed callback-interface ABI details for external binding authors and updated built-in bindings; Kotlin also switched to JNA direct mapping. The same release requires UDL trait interfaces to use `#[uniffi::trait_interface]`.
- UniFFI 0.32 deprecates `#[uniffi::export(with_foreign)]` in favor of `#[uniffi::export(foreign)]` for foreign-only implementations or `#[uniffi::export(rust, foreign)]` when both sides implement the trait. The current callback sketch uses `with_foreign`.
- UniFFI 0.32.1 fixed a Kotlin checksum failure on AArch64; 0.32.2 fixed Kotlin compiler warnings in generated code.
- The UniFFI release notes do not establish compatibility of generated bindings with Kotlin 2.x. That remains **UNVERIFIED** until the generated bridge is compiled with the selected Kotlin Gradle plugin.

## Decision

Use UniFFI **0.32.2** as the baseline before freezing the interface. Update foreign-only callback annotations to `#[uniffi::export(foreign)]`. Fall back to 0.28.3 only if a concrete blocker is proven and documented.

The case for retaining 0.28.3 is weak: the reported `cargo-ndk` build only establishes that the current core compiles, and its `rusqlite` feature set is plain `bundled` SQLite, not SQLCipher. It does not demonstrate that 0.28.3 is more stable for the Android SQLCipher bridge. Upgrading before interface freeze makes the callback annotation change cheaper to validate and includes the later Kotlin/AArch64 fixes. Generated bindings must still be tested with the chosen Kotlin toolchain.

## Consequences

- UniFFI 0.32.2 is approved for the Android bridge; the dependency and bindings are updated as part of the Gate 2 spike.
- The Gate 2 spike must validate SQLCipher, Kotlin callback execution, and generated bindings on the physical phone.
- UDL remains an available interface format; the claim that it was removed is withdrawn.

## Sources

- [UniFFI changelog at v0.32.2](https://github.com/mozilla/uniffi-rs/blob/v0.32.2/CHANGELOG.md)
- [UniFFI 0.28.3 release](https://github.com/mozilla/uniffi-rs/releases/tag/v0.28.3)
- [UniFFI 0.32.2 release](https://github.com/mozilla/uniffi-rs/releases/tag/v0.32.2)
- [UniFFI 0.28 proc-macro guide](https://mozilla.github.io/uniffi-rs/0.28/proc_macro/index.html)
- [UniFFI crates.io package record](https://crates.io/api/v1/crates/uniffi)
