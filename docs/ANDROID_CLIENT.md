# Android Client

- **Purpose:** Specify the client's modules, patterns, navigation, composer, offline behavior, permissions, accessibility and visual direction.
- **Audience:** Android engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-25
- **Related docs:** [RUST_CORE](RUST_CORE.md), [ARCHITECTURE](ARCHITECTURE.md), [FOLDER_STRUCTURE](FOLDER_STRUCTURE.md), [../skills.md](../skills.md)

## Baseline [v1]
Min SDK 26 (Android 8.0). The Gate 2 spike uses compile/target SDK 36. Google Play's 2026 target requirement and the exact AGP 9.3.1 / Gradle 9.5.0 / Kotlin 2.4.21 baseline are recorded in [VERIFY_BEFORE_BUILD](VERIFY_BEFORE_BUILD.md); generated Kotlin bindings compiled in CI and the SQLCipher integration test passed on a physical API 34 arm64 phone. See [ADR-0010](decisions/ADR-0010-sqlcipher-android-build.md).

## Modules [v1]
`:app` (application, activities, navigation) · `:core:ui` (design tokens, Markdown and code renderer, attachment preview, citation chips) · `:core:rustbridge` (UniFFI wrapper, Keystore key handling, Flow adapters) · `:feature:chat` · `:feature:camera` · `:feature:auth` · `:feature:settings`. Tree: [FOLDER_STRUCTURE](FOLDER_STRUCTURE.md).

## Pattern: unidirectional MVI [v1]
Each screen has `UiState` (immutable), a sealed `UiIntent`, and one-off `UiEffect` (snackbars, navigation). ViewModels collect Kotlin Flows bridged from the Rust core. Hilt for DI (verify it still suits).

## Navigation (type-safe routes) [v1]
`Auth` · `Chat(conversationId?)` · `Scanner` · `Settings` · `AttachmentViewer(attachmentId, page)`.

## Gate 4: auth and stable CI debug signing
Google sign-in's Android OAuth client uses the debug package name `io.luoijin.ruo.debug` and the SHA-1 of the certificate that signs the debug APK. Keep a dedicated CI debug key stable across runs; never reuse it for release signing or commit the keystore.

Owner setup, before wiring Google sign-in:
1. Generate a dedicated keystore outside the repository with the JDK `keytool` utility: `New-Item -ItemType Directory -Force "$HOME\.android" | Out-Null; keytool -genkeypair -v -keystore "$HOME\.android\ruo-debug-ci.keystore" -alias ruo-debug-ci`. Retain the keystore and key passwords locally.
2. Read its SHA-1 with `keytool -list -v -keystore <keystore-path> -alias ruo-debug-ci` and register that fingerprint for the Android OAuth client with package `io.luoijin.ruo.debug` in the Google Cloud project.
3. In PowerShell, copy a base64 encoding of the keystore to the clipboard without printing it: `$path = "$HOME\.android\ruo-debug-ci.keystore"; [Convert]::ToBase64String([IO.File]::ReadAllBytes($path)) | Set-Clipboard`.
4. In repository **Settings → Secrets and variables → Actions**, create `RUO_DEBUG_KEYSTORE_BASE64` (paste the clipboard value), `RUO_DEBUG_STORE_PASSWORD`, `RUO_DEBUG_KEY_ALIAS`, and `RUO_DEBUG_KEY_PASSWORD`. Do not paste secret values into a terminal transcript, issue, or pull request.

CI changes planned for Gate 4: expose those secrets only to the trusted Android build job; decode the keystore into `RUNNER_TEMP`; configure only the `debug` build type to use it; verify the APK certificate fingerprint; and delete the temporary keystore after staging artifacts. Keep release signing separate. Fork pull requests receive no secrets, so CI must omit the device-test APK artifact when the key is unavailable rather than publish an APK with a changing signature. The workflow must never print secret values.

Sources checked 2026-11-25: [GitHub Actions secrets](https://docs.github.com/en/actions/security-for-github-actions/security-guides/using-secrets-in-github-actions), [Android app signing](https://developer.android.com/studio/publish/app-signing), and [Google Android client authentication and SHA-1 fingerprints](https://developers.google.com/android/guides/client-auth).

## Composer tray [v1]
Horizontal row of thumbnails or file cards, each with a remove button; page-number badges or drag handles for multi-image messages; a progress ring: Uploading → Extracting → Ready or Failed. Quick action chips (Summarize, Explain, Translate, Solve) appear on attachments.

## Offline behavior [v1]
- Drafts save per conversation about every 500 ms (via Rust).
- A message sent offline is stored as Queued with a "waiting for network" badge.
- A WorkManager job with a network constraint triggers the Rust outbox flush on reconnect.
- A persistent offline banner shows while disconnected.

## Permissions (just in time) [v1]
Camera when the user taps Camera; microphone when they tap Mic; optional notifications (Android 13+) for long extractions. No broad storage permissions: use the system Photo Picker and Storage Access Framework. Adding any permission needs owner approval.

## Accessibility [v1]
48 dp minimum touch targets; content descriptions on icon buttons and attachment previews; polite live-region announcement when a reply finishes streaming; font scaling to 200% without clipping; WCAG AA contrast.

## Visual direction (starting point, not locked) [v1]
Calm, warm, academic-notebook feel for long study sessions; no neon gradients.
| Token | Light | Dark |
|---|---|---|
| background | #FAF9F6 | #111315 |
| surface | #FFFFFF | #1A1D21 |
| surface variant | #F1EFEA | #252930 |
| primary | #1E4D4B | #5EEAD4 |
| accent (study hints) | #D97706 | #FBBF24 |
| error | #B91C1C | not specified |

Type: Plus Jakarta Sans (headings), Inter (body), JetBrains Mono (code, math). Check each font's license and Android bundling before committing. Motion: subtle streaming cursor, collapsible "reading your PDF" indicator, light haptic tick on record start/stop, swipe on bubbles to reply or edit.

## Kotlin vs Rust
Kotlin owns: Compose UI, CameraX, ML Kit scanner, file and photo pickers, audio record/playback, permissions, share intents, image resize and metadata strip. Rust owns: encrypted DB, outbox, SSE and reconnects, chunked uploader and hashing, client validation. The Rust callback threads are not the main thread: the wrapper hops to the main dispatcher before touching UI state ([RUST_CORE](RUST_CORE.md)).

## Mid-range phone efficiency [v1]
Compress images off the main thread; record audio to a file, not memory; cancel in-flight streams when the user leaves the screen or taps stop; cap the thumbnail cache (about 100 MB).

## Strings [v1]
All user-facing text in resource files; English (`values/`) and Filipino (`values-fil/`) first. UI language and response language are separate settings.
