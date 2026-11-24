# Development Setup

- **Purpose:** Install the toolchain on a low-disk laptop and run the stack with a physical phone. Canonical home for setup steps and disk constraints.
- **Audience:** Owner and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [INFRASTRUCTURE](INFRASTRUCTURE.md), [RUST_CORE](RUST_CORE.md), [TECH_STACK](TECH_STACK.md)

## Constraints
Laptop has limited free disk (about 80 GB). Therefore: **physical Android phone, no emulator images**; one SDK platform; trimmed toolchain; one Rust ABI (arm64-v8a) at first. Versions: take them from [TECH_STACK](TECH_STACK.md) after verification; do not guess.

## Install in stages (only what you need, when you need it)
Install later stages only when the phase task needs them. This saves disk and setup time. Versions come from [TECH_STACK](TECH_STACK.md) after verification (UNVERIFIED until then).

### Stage 0: no Android tooling [v1]
Enough for [Phase 1](phases/PHASE-1-core-spine.md) tasks 1, 4-8 and the non-Android parts of 9.
- [ ] Git and a code editor
- [ ] Docker (for Postgres + pgvector and the API)
- [ ] Python and a virtual environment for `ai-service/`
- [ ] Rust via rustup (host target only; run `cargo test` for SSE parsing, backoff, outbox, SQLCipher logic)
- [ ] Gemini API key; check limits and model names ([VERIFY_BEFORE_BUILD](VERIFY_BEFORE_BUILD.md) V1, V2)

### Stage 1: before the SQLCipher/UniFFI spike (task 2) [v1]
- [ ] Android Studio (or the command-line SDK tools). Skip the emulator and system images.
- [ ] **One** SDK platform (the one your Compose BOM supports; verify)
- [ ] The NDK (version per cargo-ndk docs; verify)
- [ ] The Android Rust target for your phone's ABI only (arm64-v8a first)
- [ ] `cargo-ndk`
- [ ] Phone: Developer options, USB debugging, `adb devices` shows it
- [ ] Do the spike now: it is the riskiest step ([RUST_CORE](RUST_CORE.md))

### Stage 2: Compose screens (tasks 10-11) [v1]
- [ ] Gradle sync works; version catalog filled with verified versions
- [ ] CameraX and ML Kit dependencies are **not** needed until Phase 2; do not add them yet

### Stage 3: later phases [v1]
- [ ] Phase 3: nothing new locally except PDF/DOCX fixture files
- [ ] Phase 5: signing keystore (keep it out of the repo), crash-reporting setup

## Disk-saving rules
Install only the arm64-v8a target at first, one SDK platform, no emulator images. Prune regularly: `docker system prune`, old `cargo` targets, Gradle caches.

## Full toolchain reference [v1]
1. **Git** and a code editor.
2. **Android Studio** (or command-line SDK): install one SDK platform and the NDK (exact versions UNVERIFIED; choose per Compose BOM and cargo-ndk docs). Skip system images.
3. **Rust** via rustup; add the Android target for your phone's ABI; `cargo install cargo-ndk` (check README for current usage).
4. **Python** and a virtual environment for `ai-service/`.
5. **Docker** (Desktop or Engine) for Postgres + pgvector and the API.
6. Keep disk in check: `docker system prune`, `cargo clean` on old targets, prune Gradle caches.

## Physical-device workflow
1. Enable Developer options and USB debugging on the phone.
2. Connect by USB; confirm with `adb devices`.
3. Reach the backend: use `adb reverse tcp:<port> tcp:<port>` so the phone sees `localhost` (debug builds only; release builds are TLS-only).
4. Install and run from Android Studio or `./gradlew installDebug`.

## Run the stack locally
1. Copy `infra/.env.example` to `infra/.env`; fill in values locally (never commit).
2. `docker compose -f infra/docker-compose.yml up` (once the file exists).
3. Apply migrations (`alembic upgrade head`) and run backend tests (`pytest`).
4. Build the Rust core for the phone and the Android app ([RUST_CORE](RUST_CORE.md)).

## Common build problems
| Symptom | Likely cause / fix |
|---|---|
| Linker errors for OpenSSL/SQLCipher on Android | Known risk: see [RUST_CORE](RUST_CORE.md); spike first |
| `UnsatisfiedLinkError` on launch | `.so` missing for the phone's ABI in `jniLibs` |
| Phone cannot reach backend | `adb reverse` missing, or LAN firewall |
| Bindings out of date | Regenerate UniFFI bindings after any interface change |
| Disk full | Prune Docker, Gradle and Cargo caches |
