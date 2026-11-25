# ADR-0010: SQLCipher Android build configuration

- **Purpose:** Record the verified Gate 2 SQLCipher/OpenSSL Android configuration.
- **Audience:** Engineers and agents.
- **Status:** Accepted
- **Last updated:** 2026-11-25
- **Related docs:** [ADR guide](README.md), [RUST_CORE](../RUST_CORE.md), [ANDROID_CLIENT](../ANDROID_CLIENT.md), [VERIFY_BEFORE_BUILD](../VERIFY_BEFORE_BUILD.md)

**ADR status:** Accepted 2026-11-25.

## Context

Application data must use SQLCipher; plain SQLite is not an acceptable app fallback. A separate, explicit host-debug feature is permitted only to run logic tests without SQLCipher and is guarded against Android and release builds. Gate 2 also requires a working Android UniFFI binding and proof on a physical arm64 phone that the key, ciphertext header, multilingual text, and Kotlin callback behave as intended.

CI run [45594145222](https://github.com/luoijin/Ruo-AI/actions/runs/45594145222) passed the Android, backend, SQLCipher, and plain-SQLite jobs. The instrumented SQLCipher test passed on a physical API 34 `arm64-v8a` device on 2026-11-25.

## Decision

Use the following configuration for the Gate 2 SQLCipher bridge:

- Use `rusqlite` 0.43.0 with `default-features = false`; the crate's `sqlcipher` feature enables the exact rusqlite feature `bundled-sqlcipher-vendored-openssl`. The lockfile resolves `libsqlite3-sys` 0.40.0, `sqlcipher-src` `420.0.0+sqlcipher-4.20.0-sqlite-3.54.1-libtomcrypt-1.18.2`, `openssl-sys` 0.9.118, and `openssl-src` `300.6.1+3.6.3` (OpenSSL 3.6.3).
- SQLCipher is the default Rust feature for normal, Android, and release builds. The explicit `plain-sqlite-tests` feature (`rusqlite/bundled`) is only for host debug logic tests; compile-time guards reject it on Android and in release builds.
- The CI runner uses NDK `30.0.16248370` and `cargo-ndk` 4.1.2 to build only `aarch64-linux-android` / `arm64-v8a`, API 26.
- Use UniFFI 0.32.2 proc-macro bindings; generate Kotlin from the Android `.so`.
- Build with Java 17, AGP 9.3.1, Gradle 9.5.0, and Kotlin 2.4.21 as AGP's runtime dependency. Use AGP built-in Kotlin rather than applying `org.jetbrains.kotlin.android`, and register generated Kotlin sources through the `androidComponents` variant API.
- Include AndroidX Annotation 1.11.0 for UniFFI's generated `RequiresApi` annotations and JNA 5.19.1 for the generated bindings.
- Generate a 32-byte random database key in the Android layer, wrap it with an Android Keystore key, and pass the unwrapped bytes to Rust only when opening the database. Rust does not persist the key.

The on-device instrumentation test verifies that the correct key reopens the encrypted database, wrong and empty keys fail, the file does not start with the SQLite format header, Taglish/Cebuano/emoji text round-trips, and a Kotlin listener receives the callback.

The downloaded `arm64-v8a` library from CI run [45594145222](https://github.com/luoijin/Ruo-AI/actions/runs/45594145222) is 12,072,936 bytes (12.07 MB decimal; 11.51 MiB).

## Consequences

- The arm64 NDK build, UniFFI Kotlin generation, APK assembly, and the specified phone test have passed.
- This validates one API 34 arm64 device; behavior on other Android versions and hardware remains **UNVERIFIED**.
- The device test was on API 34 only. Runtime behavior at minSdk 26 and on low-RAM devices remains **UNVERIFIED**.
- SQLCipher remains mandatory for Android and release builds; no unencrypted fallback is permitted. Plain bundled SQLite is isolated to the guarded host debug test feature and is not suitable for application data.
- This ADR does not freeze the complete UniFFI interface or authorize Gate 3 work.

## Third-party licenses and notices

- SQLCipher 4.20.0's upstream `LICENSE.md` is BSD 3-Clause: retain its copyright, conditions, and disclaimer in source distributions; reproduce them in documentation or other materials for binary distributions; do not use Zetetic LLC or contributor names for endorsement without permission. The upstream `SQLITE_LICENSE.md` identifies SQLite's core as public domain. The locked `sqlcipher-src` package also declares `MIT AND BSD-3-Clause AND blessing AND WTFPL`; preserve the applicable package notices when redistributing its source or substantial portions.
- OpenSSL 3.6.3's upstream `LICENSE.txt` is Apache-2.0. Include a copy of the license, preserve applicable notices, mark modified files, and reproduce any bundled NOTICE attributions. The locked `openssl-src` build package declares MIT/Apache-2.0; include its applicable license notices when redistributing covered material.
- Phase 5 must package the applicable license texts and attribution notices with the app distribution.

## Sources

- [rusqlite 0.43.0 manifest](https://docs.rs/crate/rusqlite/0.43.0/source/Cargo.toml.orig)
- [libsqlite3-sys 0.40.0 feature graph](https://docs.rs/crate/libsqlite3-sys/0.40.0/features)
- [sqlcipher-src 420.0.0 manifest](https://docs.rs/crate/sqlcipher-src/420.0.0/source/Cargo.toml.orig)
- [SQLCipher 4.20.0 license](https://github.com/sqlcipher/sqlcipher/blob/v4.20.0/LICENSE.md) and [SQLite license](https://github.com/sqlcipher/sqlcipher/blob/v4.20.0/SQLITE_LICENSE.md)
- [OpenSSL 3.6.3 license](https://github.com/openssl/openssl/blob/openssl-3.6.3/LICENSE.txt)
- [openssl-src 300.6.1+3.6.3 manifest](https://docs.rs/crate/openssl-src/300.6.1+3.6.3/source/Cargo.toml.orig)
- [Android NDK downloads](https://developer.android.com/ndk/downloads)
- [AndroidX Annotation release notes](https://developer.android.com/jetpack/androidx/releases/annotation)
- [UniFFI 0.32 proc-macro guide](https://mozilla.github.io/uniffi-rs/0.32/proc_macro/index.html)
- [Gate 2 CI run](https://github.com/luoijin/Ruo-AI/actions/runs/45588157467)
