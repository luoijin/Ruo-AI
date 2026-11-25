# ADR-0010: SQLCipher Android build configuration

- **Purpose:** Record the verified Gate 2 SQLCipher/OpenSSL Android configuration.
- **Audience:** Engineers and agents.
- **Status:** Proposed
- **Last updated:** 2026-11-25
- **Related docs:** [ADR guide](README.md), [RUST_CORE](../RUST_CORE.md), [ANDROID_CLIENT](../ANDROID_CLIENT.md), [VERIFY_BEFORE_BUILD](../VERIFY_BEFORE_BUILD.md)

**ADR status:** Proposed 2026-11-25; the owner must accept this configuration before Gate 3.

## Context

The local Rust database must use SQLCipher; plain SQLite is not an acceptable fallback. Gate 2 also requires a working Android UniFFI binding and proof on a physical arm64 phone that the key, ciphertext header, multilingual text, and Kotlin callback behave as intended.

CI run [45588157467](https://github.com/luoijin/Ruo-AI/actions/runs/45588157467) built the native library and APKs. The instrumented SQLCipher test passed on a physical API 34 `arm64-v8a` device on 2026-11-25.

## Decision

Use the following configuration for the Gate 2 SQLCipher bridge:

- `rusqlite` 0.43.0 with `default-features = false` and `bundled-sqlcipher-vendored-openssl`; the corresponding `libsqlite3-sys` 0.40.0 feature graph enables vendored OpenSSL.
- Build only `aarch64-linux-android` / `arm64-v8a`, API 26, with NDK `30.0.16248370` and `cargo-ndk` 4.1.2.
- Use UniFFI 0.32.2 proc-macro bindings; generate Kotlin from the Android `.so`.
- Build with Java 17, AGP 9.3.1, Gradle 9.5.0, and Kotlin 2.4.21 as AGP's runtime dependency while using AGP built-in Kotlin. Register generated Kotlin sources through `androidComponents` variant sources.
- Include AndroidX Annotation 1.11.0 for UniFFI's generated `RequiresApi` annotations and JNA 5.19.1 for the generated bindings.
- Generate a 32-byte random database key in the Android layer, wrap it with an Android Keystore key, and pass the unwrapped bytes to Rust only when opening the database. Rust does not persist the key.

The on-device instrumentation test verifies that the correct key reopens the encrypted database, wrong and empty keys fail, the file does not start with the SQLite format header, Taglish/Cebuano/emoji text round-trips, and a Kotlin listener receives the callback.

## Consequences

- The arm64 NDK build, UniFFI Kotlin generation, APK assembly, and the specified phone test have passed.
- This validates one API 34 arm64 device; behavior on other Android versions and hardware remains **UNVERIFIED**.
- SQLCipher remains mandatory; no unencrypted fallback is permitted.
- This ADR does not freeze the complete UniFFI interface or authorize Gate 3 work.

## Sources

- [rusqlite 0.43.0 manifest](https://docs.rs/crate/rusqlite/0.43.0/source/Cargo.toml.orig)
- [libsqlite3-sys 0.40.0 feature graph](https://docs.rs/crate/libsqlite3-sys/0.40.0/features)
- [Android NDK downloads](https://developer.android.com/ndk/downloads)
- [AndroidX Annotation release notes](https://developer.android.com/jetpack/androidx/releases/annotation)
- [UniFFI 0.32 proc-macro guide](https://mozilla.github.io/uniffi-rs/0.32/proc_macro/index.html)
- [Gate 2 CI run](https://github.com/luoijin/Ruo-AI/actions/runs/45588157467)
