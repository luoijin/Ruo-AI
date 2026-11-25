# android/

- **Purpose:** Kotlin Android app and SQLCipher/UniFFI spike.
- **Audience:** Android engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-26
- **Related docs:** [ANDROID_CLIENT](../docs/ANDROID_CLIENT.md), [FOLDER_STRUCTURE](../docs/FOLDER_STRUCTURE.md)

The `:app` contains the Gate 4A Compose/Hilt shell and typed placeholder navigation; it has no authentication or chat UI yet. The `:core:rustbridge` module remains the Gate 2/3 SQLCipher and UniFFI bridge. CI uses the stable debug key only for debug APKs when all signing secrets are available, checks the APK certificate fingerprints, and never uploads the key; APK artifacts are omitted when secrets are unavailable. See [ANDROID_CLIENT](../docs/ANDROID_CLIENT.md), [ADR-0010](../docs/decisions/ADR-0010-sqlcipher-android-build.md), and [RUST_CORE](../docs/RUST_CORE.md).
