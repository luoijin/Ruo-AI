# Rust Core

- **Purpose:** Define the headless Rust core: crate layout, UniFFI interface, threading, errors, build and testing.
- **Audience:** Rust and Android engineers, agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ANDROID_CLIENT](ANDROID_CLIENT.md), [API_CONTRACT](API_CONTRACT.md), [DATA_MODEL](DATA_MODEL.md), [../skills.md](../skills.md), [ADR-0002](decisions/ADR-0002-rust-uniffi-client-core.md)

## Responsibilities [v1]
SSE client with reconnect and cancel · resumable chunked uploader and hashing · SQLCipher store with migrations · outbox queue · client-side validation. **Headless:** never touches Android lifecycle, content URIs or Compose state. Kotlin passes plain file paths and primitives.

Justification: reliability on the hardest client problems and a skills showcase. Not cross-platform reuse.

## Crate layout
See [FOLDER_STRUCTURE](FOLDER_STRUCTURE.md): `ruo_core` (cdylib, plus lib for tests) with modules `db`, `net`, `upload`, `outbox`, `validate`.

## UniFFI interface (sketch, not final code)
Shape only. **Check syntax against the docs for the UniFFI version you install (UNVERIFIED).** The core is an *object with a constructor*, not a trait. Callback interfaces use `with_foreign`.
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

#[uniffi::export(with_foreign)]
pub trait StreamListener: Send + Sync {
    fn on_delta(&self, message_id: String, text: String);
    fn on_citation(&self, message_id: String, citation_json: String);
    fn on_complete(&self, message_id: String);
    fn on_error(&self, message_id: String, code: String, retryable: bool);
}

#[uniffi::export(with_foreign)]
pub trait UploadProgress: Send + Sync {
    fn on_progress(&self, attachment_id: String, sent: u64, total: u64);
    fn on_state(&self, attachment_id: String, state: String); // uploading | extracting | ready | failed
}

#[derive(uniffi::Object)]
pub struct RuoCore { /* db handle, http client, runtime */ }

#[uniffi::export]
impl RuoCore {
    #[uniffi::constructor]
    pub fn open(db_path: String, key: Vec<u8>) -> Result<std::sync::Arc<Self>, CoreError>;
    pub fn set_session(&self, access_token: String, base_url: String);
    pub fn save_draft(&self, conversation_id: String, text: String) -> Result<(), CoreError>;
    pub fn queue_message(&self, req: OutboundRequest) -> Result<LocalMessage, CoreError>;
    pub fn stream_message(&self, req: OutboundRequest, listener: std::sync::Arc<dyn StreamListener>);
    pub fn upload_attachment(&self, file_path: String, mime: String, progress: std::sync::Arc<dyn UploadProgress>);
    pub fn flush_outbox(&self) -> Result<u32, CoreError>;
    pub fn list_conversations(&self, query: Option<String>) -> Result<Vec<ConversationSummary>, CoreError>;
}
```
`OutboundRequest`, `ConversationSummary` and `CoreError` are defined in Phase 1. **Freeze this interface early** so UI work does not churn the bindings. Cancel support (needed by stop and screen exit) must be added to the interface in Phase 1 (**gap:** the sketch has no cancel method; see [OPEN_QUESTIONS](OPEN_QUESTIONS.md) Q10).

## Threading rules
- The core owns an async runtime; exported calls return quickly or run on it.
- Callbacks run on a Rust thread. The Kotlin wrapper must hop to the main dispatcher before touching UI state.
- Never block the main thread across the FFI boundary.

## Key handling
The DB key is passed as raw bytes from Android Keystore. Rust never stores or persists it; zeroize buffers where practical.

## Error handling
One `CoreError` enum exported over UniFFI; errors carry a stable code and a `retryable` flag. Stream errors surface through `on_error`; never panic across the FFI boundary.

## Build with cargo-ndk
1. Install Rust targets for the ABIs you ship (arm64-v8a first; add others only if needed for disk and testing).
2. Build with `cargo ndk -t <abi> -o <jniLibs dir> build --release` (flags **UNVERIFIED**; check the cargo-ndk README for the installed version).
3. Generate Kotlin bindings with the UniFFI bindgen for the installed version.
4. Output `.so` goes to `android/core/rustbridge/src/main/jniLibs/` (gitignored).
Scripts live in `core-rust/scripts/`.

## Testing
Unit tests: SSE framing, reconnect backoff, outbox transitions, migrations, upload resume. FFI tests (Android instrumented): Taglish, Cebuano and emoji strings, mid-flight cancel, error propagation. See [TESTING_EVALS](TESTING_EVALS.md).

## Risk: SQLCipher with OpenSSL on Android
Highest build risk. Spike in Phase 1 week one ([PHASE-1](phases/PHASE-1-core-spine.md)). Record the working configuration in an ADR once found.
