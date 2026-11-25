# Rust Core

- **Purpose:** Define the headless Rust core: crate layout, UniFFI interface, threading, errors, build and testing.
- **Audience:** Rust and Android engineers, agents.
- **Status:** Draft
- **Last updated:** 2026-11-25
- **Related docs:** [ANDROID_CLIENT](ANDROID_CLIENT.md), [API_CONTRACT](API_CONTRACT.md), [DATA_MODEL](DATA_MODEL.md), [../skills.md](../skills.md), [ADR-0002](decisions/ADR-0002-rust-uniffi-client-core.md)

## Responsibilities [v1]
SSE client with reconnect and cancel · resumable chunked uploader and hashing · SQLCipher store with migrations · outbox queue · client-side validation. **Headless:** never touches Android lifecycle, content URIs or Compose state. Kotlin passes plain file paths and primitives.

Justification: reliability on the hardest client problems and a skills showcase. Not cross-platform reuse.

## Crate layout
See [FOLDER_STRUCTURE](FOLDER_STRUCTURE.md): `ruo_core` (cdylib, plus lib for tests) with modules `db`, `net`, `upload`, `outbox`, `validate`.

## UniFFI interface (spike, not frozen)
The Gate 2 spike uses UniFFI 0.32.2 proc-macro mode; syntax follows the [0.32 proc-macro guide](https://mozilla.github.io/uniffi-rs/0.32/proc_macro/index.html). The core is an *object with a constructor*, not a trait. The Gate 3 interface freeze may refine this surface.
```rust
#[derive(uniffi::Record)]
pub struct LocalMessage {
    pub id: String, pub conversation_id: String,
    pub role: String,        // "user" | "assistant"
    pub content_md: String,
    pub status: String,      // "queued" | "streaming" | "sent" | "failed"
    pub citations_json: Option<String>,
    pub created_at_ms: i64,
}

#[uniffi::export(foreign)]
pub trait StreamListener: Send + Sync {
    fn on_meta(&self, message_id: String, detected_language: Option<String>, fallback: bool);
    fn on_delta(&self, message_id: String, text: String);
    fn on_citation(&self, message_id: String, attachment_id: String, snippet: Option<String>);
    fn on_complete(&self, message_id: String);
    fn on_error(&self, message_id: String, code: String, retryable: bool);
}

#[uniffi::export(foreign)]
pub trait DraftReadListener: Send + Sync {
    fn on_draft_read(&self, conversation_id: String, text: String);
}

#[derive(uniffi::Object)]
pub struct RuoCore { /* db handle, http client, runtime */ }

#[uniffi::export]
impl RuoCore {
    #[uniffi::constructor]
    pub fn open(db_path: String, key: Vec<u8>) -> Result<std::sync::Arc<Self>, CoreError>;
    pub fn set_session(&self, access_token: String, base_url: String);
    pub fn create_conversation(&self, id: String, title: String, active_language: String, study_mode_default: bool) -> Result<(), CoreError>;
    pub fn save_draft(&self, conversation_id: String, text: String) -> Result<(), CoreError>;
    pub fn get_draft(&self, conversation_id: String) -> Result<Option<String>, CoreError>;
    pub fn read_draft_with_listener(&self, conversation_id: String, listener: std::sync::Arc<dyn DraftReadListener>) -> Result<(), CoreError>;
    pub fn queue_message(&self, req: OutboundRequest) -> Result<LocalMessage, CoreError>;
    pub fn stream_message(&self, req: OutboundRequest, listener: std::sync::Arc<dyn StreamListener>);
    pub fn cancel_stream(&self, message_id: String) -> bool;
    pub fn list_conversations(&self, query: Option<String>) -> Result<Vec<ConversationSummary>, CoreError>;
}
```
`OutboundRequest`, `ConversationSummary` and `CoreError` are defined in Phase 1. The spike requires a 32-byte database key; Gate 3 will review and freeze the complete interface before app-shell work.

## Threading rules
- The core owns an async runtime; exported calls return quickly or run on it.
- Asynchronous stream callbacks run on a Tokio worker thread; synchronous callbacks may run on the calling thread. The Kotlin wrapper must hop to the main dispatcher before touching UI state.
- Never block the main thread across the FFI boundary.

## Key handling
The DB key is passed as raw bytes from Android Keystore. Production builds default to the `sqlcipher` feature, which rejects keys not exactly 32 bytes, applies SQLCipher's raw-key pragma before the first DB read, and probes `sqlite_master` so a wrong key fails before migrations. Rust does not persist the key and clears the owned constructor buffer after opening; zeroize additional temporary buffers where practical.

For host-only logic testing on machines without Perl/OpenSSL build tools, run `cargo test --all --no-default-features --features plain-sqlite-tests` from `core-rust`. This explicitly uses bundled plain SQLite and does not test encryption. Compile-time guards reject this feature on Android and in release builds; do not use it for application data. The Linux CI SQLCipher job runs the full encryption tests separately.

## Error handling
One `CoreError` enum exported over UniFFI; error payload text is exposed as `details` (not `message`, which collides with Kotlin's `Throwable.message`) alongside a stable code and `retryable` flag. Stream errors surface through `on_error`; never panic across the FFI boundary.

## Build with cargo-ndk
The Gate 2 CI job pins `cargo-ndk` 4.1.2 and NDK `30.0.16248370`, targets only `aarch64-linux-android` at API 26, and places the `.so` in `android/core/rustbridge/src/main/jniLibs/arm64-v8a/` (gitignored). Its commands follow the [cargo-ndk 4.1.2 README](https://github.com/bbqsrc/cargo-ndk/blob/v4.1.2/README.md). CI run [45588157467](https://github.com/luoijin/Ruo-AI/actions/runs/45588157467) successfully compiled the arm64 SQLCipher/OpenSSL library, generated UniFFI 0.32.2 Kotlin bindings, and assembled debug and instrumentation APKs. The on-device test passed on API 34 arm64; the exact verified configuration is recorded in [ADR-0010](decisions/ADR-0010-sqlcipher-android-build.md). See [VERIFY_BEFORE_BUILD](VERIFY_BEFORE_BUILD.md).

## Testing
Unit tests: SSE framing, reconnect backoff, outbox transitions, migrations, upload resume, and SQLCipher key/header behavior. `cargo test --all --no-default-features --features sqlcipher` runs the full encryption suite; `cargo test --all --no-default-features --features plain-sqlite-tests` runs host-only logic tests against plain SQLite. The Gate 2 Android instrumented test passed on API 34 arm64, covering Keystore-wrapped key round-trip, wrong/missing keys, encrypted header, multilingual draft persistence, and a Kotlin callback. Mid-stream cancel remains part of Gate 3. See [TESTING_EVALS](TESTING_EVALS.md).

## Risk: SQLCipher with OpenSSL on Android
The vendored-OpenSSL SQLCipher build and physical-phone checks passed for the configuration recorded in [ADR-0010](decisions/ADR-0010-sqlcipher-android-build.md). Do not fall back to unencrypted storage. The ADR remains Proposed pending owner approval; Gate 3 is not authorized by this verification.
