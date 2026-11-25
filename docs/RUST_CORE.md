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
These are Gate 2 spike signatures, not a frozen contract. The spike requires a 32-byte database key; Gate 3 will review and freeze the Phase 1 surface before app-shell work.

## Gate 3 interface proposal (not frozen)

This is a Phase 1 proposal for owner approval, not an implementation. It covers chat, cached history, drafts, and send state only; uploads, audio, and other later-phase APIs are excluded.

### Ownership and recovery

Kotlin owns sign-in and refresh-token rotation. Rust holds only the current access token in memory and owns authenticated HTTP for conversation/history sync and message streaming; it never stores or logs tokens or message content. On HTTP 401, Rust returns `UNAUTHENTICATED` without refreshing; Kotlin refreshes, calls `set_session` again, then retries once with the same `client_msg_id`. This keeps refresh credentials out of Rust while centralizing SSE parsing, retries, and local persistence. Kotlin owning all content HTTP instead would duplicate those concerns across the bridge.

```rust
pub fn open(db_path: String, key_bytes: Vec<u8>) -> Result<std::sync::Arc<RuoCore>, CoreError>;
pub fn set_session(&self, access_token: String, base_url: String) -> Result<(), CoreError>;
pub fn sync_history(&self, listener: std::sync::Arc<dyn SyncListener>) -> Result<(), CoreError>;
pub fn create_conversation(&self, title: String, active_language: String, study_mode_default: bool, listener: std::sync::Arc<dyn SyncListener>) -> Result<(), CoreError>;
pub fn update_conversation(&self, conversation_id: String, title: Option<String>, active_language: Option<String>, study_mode_default: Option<bool>, listener: std::sync::Arc<dyn SyncListener>) -> Result<(), CoreError>;
pub fn delete_conversation(&self, conversation_id: String, listener: std::sync::Arc<dyn SyncListener>) -> Result<(), CoreError>;
pub fn list_conversations(&self, query: Option<String>, limit: u32, before_updated_at_ms: Option<i64>) -> Result<Vec<ConversationSummary>, CoreError>;
pub fn list_messages(&self, conversation_id: String, limit: u32, before_created_at_ms: Option<i64>) -> Result<Vec<LocalMessage>, CoreError>;
pub fn get_draft(&self, conversation_id: String) -> Result<Option<String>, CoreError>;
pub fn save_draft(&self, conversation_id: String, text: String) -> Result<(), CoreError>;
pub fn outbox_state(&self, conversation_id: Option<String>) -> Result<Vec<OutboxStatus>, CoreError>;
pub fn subscribe_local_changes(&self, listener: std::sync::Arc<dyn LocalChangeListener>) -> Result<std::sync::Arc<LocalChangeSubscription>, CoreError>;
pub fn stream_message(&self, request: OutboundRequest, listener: std::sync::Arc<dyn StreamListener>) -> Result<(), CoreError>;
pub fn cancel_stream(&self, message_id: String) -> bool;
pub struct OutboundRequest { client_msg_id: String, conversation_id: String, content: String, parent_message_id: Option<String> }
pub enum OutboxState { Queued, Sending, Failed }
pub struct OutboxStatus { client_msg_id: String, state: OutboxState, retry_after_sec: Option<u64> }
pub enum LocalEntity { Conversation, Message, Draft, Outbox }
pub struct LocalChange { entity: LocalEntity, id: String, revision: u64 }
pub struct LocalChangeSubscription;
pub struct StreamMeta { detected_language: Option<String>, study_step: Option<u32>, fallback: bool, replay: bool }
pub enum CoreError { Unauthenticated { details: String }, QuotaExceeded { details: String, retry_after_sec: Option<u64> }, RateLimited { details: String, retry_after_sec: Option<u64> }, Network { details: String, retryable: bool, retry_after_sec: Option<u64> }, Cancelled { details: String }, Validation { details: String }, NotFound { details: String }, Conflict { details: String }, KeyUnavailable { details: String }, InvalidKey { details: String }, WrongKey { details: String }, DatabaseCorrupt { details: String }, SchemaTooNew { details: String }, Internal { details: String, retryable: bool, retry_after_sec: Option<u64> } }
#[uniffi::export(foreign)] pub trait StreamListener: Send + Sync { fn on_meta(&self, message_id: String, meta: StreamMeta); fn on_delta(&self, message_id: String, text: String); fn on_done(&self, message_id: String); fn on_error(&self, message_id: String, error: CoreError); }
#[uniffi::export(foreign)] pub trait LocalChangeListener: Send + Sync { fn on_changed(&self, change: LocalChange); }
#[uniffi::export(foreign)] pub trait SyncListener: Send + Sync { fn on_complete(&self); fn on_error(&self, error: CoreError); }
```

`sync_history` pulls server conversations and messages into the encrypted local cache. The conversation mutation methods make their corresponding backend request and commit the response locally before notifying listeners. Releasing `LocalChangeSubscription` unregisters the listener. It reports entity IDs and a revision, not message content; Kotlin reads an updated snapshot after a notification and adapts notifications to `Flow`. This callback approach avoids polling; missed updates are recovered by rereading the snapshot. All callbacks arrive on Rust-owned threads; the Kotlin bridge must hop to its chosen dispatcher.

`stream_message` persists the user message/outbox item before network work and uses the caller's stable `client_msg_id` for every retry. A cancelled active stream returns `true` and reports `CANCELLED`; an unknown or completed ID returns `false`. Reconnect uses bounded backoff and the same request ID, never enqueuing a second local user message. The current backend replays a completed response in full, so a replay replaces—not appends to—the partial assistant bubble. The SSE contract has no event cursor; true delta-offset resume is not promised. The API contract says a retry resumes the stream, but current backend tests cover completed-response replay only; define and test in-progress retry behavior before implementation. Also document the existing backend request field `content` in [API_CONTRACT](API_CONTRACT.md), whose current send-body table omits it.

Conversation creation is not automatically retried after an ambiguous network failure until the API supports an idempotency key; refresh-and-retry after an explicit 401 is safe. History sync reconciles server state.

`retryable` means Rust can safely retry without host intervention, immediately or after `retry_after_sec`. `UNAUTHENTICATED` is not retryable inside Rust; Kotlin refreshes and retries after a 401. Map the backend's `RATE_LIMIT_EXCEEDED` to stable Rust code `RATE_LIMITED`. `QUOTA_EXCEEDED` is not automatically retried; the UI may offer retry after its supplied retry time. `RATE_LIMITED` is retryable after `retry_after_sec`; `NETWORK` and transient `INTERNAL` errors use bounded backoff. `CANCELLED`, `VALIDATION`, `NOT_FOUND`, `CONFLICT`, `KEY_UNAVAILABLE`, `INVALID_KEY`, `WRONG_KEY`, `DATABASE_CORRUPT`, and `SCHEMA_TOO_NEW` are not automatically retryable. `retry_after_sec` is optional and is surfaced to the UI.

`open` creates a database only when the path does not exist and a valid 32-byte key is supplied. An absent key for an existing database returns `KEY_UNAVAILABLE`; a wrong key returns `WRONG_KEY`. Corrupt or unsupported databases return distinct errors without deletion or replacement. Key rotation is not part of this interface; losing or rotating a key must never silently open a new empty database.

Local schema migrations are numbered, forward-only, and transactional. If a database schema is newer than the running core supports, `open` returns `SCHEMA_TOO_NEW` without modifying it. Freeze this interface as v1 after approval; later-phase features add methods or optional fields only. Each installed app keeps its bundled interface version; any breaking ABI change requires a separately versioned object/namespace and regenerated bindings shipped with the matching app release.

## Threading rules
- `open`, cached reads, and local writes are synchronous database operations; Kotlin calls them on an IO dispatcher, never the main thread.
- Sync and stream entry points return quickly; callbacks run only on Rust-owned worker threads. Kotlin hops to the appropriate dispatcher before touching UI state.

## Key handling
The DB key is passed as raw bytes from Android Keystore. Production builds default to the `sqlcipher` feature, which rejects keys not exactly 32 bytes, applies SQLCipher's raw-key pragma before the first DB read, and probes `sqlite_master` so a wrong key fails before migrations. Rust does not persist the key and clears the owned constructor buffer after opening; zeroize additional temporary buffers where practical.

For host-only logic testing on machines without Perl/OpenSSL build tools, run `cargo test --all --no-default-features --features plain-sqlite-tests` from `core-rust`. This explicitly uses bundled plain SQLite and does not test encryption. Compile-time guards reject this feature on Android and in release builds; do not use it for application data. The Linux CI SQLCipher job runs the full encryption tests separately.

## Error handling
One `CoreError` enum is exported over UniFFI; error payload text is exposed as `details` (not `message`, which collides with Kotlin's `Throwable.message`) alongside a stable code, `retryable` flag, and optional `retry_after_sec`. Stream errors surface through `on_error`; never panic across the FFI boundary.

## Build with cargo-ndk
The Gate 2 CI job pins `cargo-ndk` 4.1.2 and NDK `30.0.16248370`, targets only `aarch64-linux-android` at API 26, and places the `.so` in `android/core/rustbridge/src/main/jniLibs/arm64-v8a/` (gitignored). Its commands follow the [cargo-ndk 4.1.2 README](https://github.com/bbqsrc/cargo-ndk/blob/v4.1.2/README.md). CI run [45599927769](https://github.com/luoijin/Ruo-AI/actions/runs/45599927769) successfully compiled the arm64 SQLCipher/OpenSSL library, generated UniFFI 0.32.2 Kotlin bindings, and assembled debug and instrumentation APKs. The on-device test passed on API 34 arm64; the exact verified configuration is recorded in [ADR-0010](decisions/ADR-0010-sqlcipher-android-build.md). See [VERIFY_BEFORE_BUILD](VERIFY_BEFORE_BUILD.md).

## Testing
Unit tests: SSE framing, reconnect backoff, outbox transitions, migrations, upload resume, and SQLCipher key/header behavior. `cargo test --all --no-default-features --features sqlcipher` runs the full encryption suite; `cargo test --all --no-default-features --features plain-sqlite-tests` runs host-only logic tests against plain SQLite. The Gate 2 Android instrumented test passed on API 34 arm64, covering Keystore-wrapped key round-trip, wrong/missing keys, encrypted header, multilingual draft persistence, and a Kotlin callback. Mid-stream cancel remains part of Gate 3. See [TESTING_EVALS](TESTING_EVALS.md).

## Risk: SQLCipher with OpenSSL on Android
The vendored-OpenSSL SQLCipher build and physical-phone checks passed for the configuration recorded in the accepted [ADR-0010](decisions/ADR-0010-sqlcipher-android-build.md). Do not fall back to unencrypted storage. This proposal is not frozen and does not authorize Gate 3 implementation.
