#[cfg(all(feature = "sqlcipher", feature = "plain-sqlite-tests"))]
compile_error!("Choose either `sqlcipher` or `plain-sqlite-tests`, not both.");

#[cfg(not(any(feature = "sqlcipher", feature = "plain-sqlite-tests")))]
compile_error!("Enable `sqlcipher` or the host-only `plain-sqlite-tests` feature.");

#[cfg(all(feature = "plain-sqlite-tests", target_os = "android"))]
compile_error!("`plain-sqlite-tests` is forbidden on Android targets.");

#[cfg(all(feature = "plain-sqlite-tests", not(debug_assertions)))]
compile_error!("`plain-sqlite-tests` is forbidden in release builds.");

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock, Weak};

use chrono::{DateTime, SecondsFormat, Utc};
use futures_util::StreamExt;
use reqwest::{Client, Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use tokio::runtime::{Handle, Runtime};
use tokio::sync::broadcast;

pub mod db;
pub mod error;
pub mod models;
pub mod net;
pub mod outbox;
pub mod validate;

use db::DbHandle;
pub use error::CoreError;
pub use models::{
    ConversationSummary, HistorySyncConfig, LocalChange, LocalEntity, LocalMessage,
    LocalMessageStatus, MessageRole, OutboundRequest, OutboxState, OutboxStatus, StreamMeta,
};
use net::{ParsedEvent, SseParser};

uniffi::setup_scaffolding!();

static CORE_RUNTIME: OnceLock<Result<Runtime, ()>> = OnceLock::new();

fn core_runtime_handle() -> Result<Handle, CoreError> {
    CORE_RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|_| ())
        })
        .as_ref()
        .map(|runtime| runtime.handle().clone())
        .map_err(|_| CoreError::Internal {
            details: "Unable to initialize the core runtime".to_string(),
            retryable: false,
            retry_after_sec: None,
        })
}

#[uniffi::export(foreign)]
pub trait StreamListener: Send + Sync {
    fn on_meta(&self, client_msg_id: String, meta: StreamMeta);
    fn on_delta(&self, client_msg_id: String, text: String);
    fn on_done(&self, client_msg_id: String, assistant_server_id: String, replayed: bool);
    fn on_error(&self, client_msg_id: String, error: CoreError);
}

#[uniffi::export(foreign)]
pub trait LocalChangeListener: Send + Sync {
    fn on_changed(&self, change: LocalChange);
}

#[uniffi::export(foreign)]
pub trait SyncListener: Send + Sync {
    fn on_complete(&self);
    fn on_error(&self, error: CoreError);
}

#[derive(Default)]
struct ChangeBus {
    listeners: Mutex<HashMap<u64, Arc<dyn LocalChangeListener>>>,
    next_id: AtomicU64,
    revision: AtomicU64,
}

impl ChangeBus {
    fn subscribe(
        self: &Arc<Self>,
        listener: Arc<dyn LocalChangeListener>,
    ) -> Arc<LocalChangeSubscription> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        if let Ok(mut listeners) = self.listeners.lock() {
            listeners.insert(id, listener);
        }
        Arc::new(LocalChangeSubscription {
            id,
            bus: Arc::downgrade(self),
        })
    }

    fn remove(&self, id: u64) {
        if let Ok(mut listeners) = self.listeners.lock() {
            listeners.remove(&id);
        }
    }

    fn dispatch(&self, handle: &Handle, entity: LocalEntity, id: String) {
        let listeners = match self.listeners.lock() {
            Ok(listeners) => listeners.values().cloned().collect::<Vec<_>>(),
            Err(_) => return,
        };
        if listeners.is_empty() {
            return;
        }
        let change = LocalChange {
            entity,
            id,
            revision: self.revision.fetch_add(1, Ordering::Relaxed) + 1,
        };
        handle.spawn_blocking(move || {
            for listener in listeners {
                listener.on_changed(change.clone());
            }
        });
    }
}

#[derive(uniffi::Object)]
pub struct LocalChangeSubscription {
    id: u64,
    bus: Weak<ChangeBus>,
}

#[uniffi::export]
impl LocalChangeSubscription {
    pub fn unsubscribe(&self) {
        if let Some(bus) = self.bus.upgrade() {
            bus.remove(self.id);
        }
    }
}

impl Drop for LocalChangeSubscription {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

#[derive(Default)]
struct StreamState {
    by_client_msg_id: HashMap<String, broadcast::Sender<()>>,
    active_conversations: HashMap<String, String>,
}

#[derive(uniffi::Object)]
pub struct RuoCore {
    db: Arc<DbHandle>,
    runtime: Handle,
    client: Client,
    access_token: RwLock<Option<String>>,
    base_url: RwLock<Option<String>>,
    streams: Arc<Mutex<StreamState>>,
    changes: Arc<ChangeBus>,
}

#[uniffi::export]
impl RuoCore {
    #[uniffi::constructor]
    pub fn open(db_path: String, mut key_bytes: Vec<u8>) -> Result<Arc<Self>, CoreError> {
        let db_result = DbHandle::open(&db_path, &key_bytes);
        key_bytes.fill(0);
        let db = db_result?;
        let runtime = core_runtime_handle()?;
        let client = Client::builder().build().map_err(|_| CoreError::Internal {
            details: "Unable to initialize the HTTP client".to_string(),
            retryable: false,
            retry_after_sec: None,
        })?;
        Ok(Arc::new(Self {
            db: Arc::new(db),
            runtime,
            client,
            access_token: RwLock::new(None),
            base_url: RwLock::new(None),
            streams: Arc::new(Mutex::new(StreamState::default())),
            changes: Arc::new(ChangeBus::default()),
        }))
    }

    pub fn set_session(
        &self,
        access_token: String,
        base_url: String,
        allow_cleartext_debug: bool,
    ) -> Result<(), CoreError> {
        if access_token.trim().is_empty() {
            return Err(validation_error("Access token is required"));
        }
        let url = validate_base_url(&base_url, allow_cleartext_debug)?;
        *self
            .access_token
            .write()
            .map_err(|_| internal_error(false))? = Some(access_token);
        *self.base_url.write().map_err(|_| internal_error(false))? = Some(url);
        Ok(())
    }

    pub fn clear_session(&self) -> Result<(), CoreError> {
        *self
            .access_token
            .write()
            .map_err(|_| internal_error(false))? = None;
        *self.base_url.write().map_err(|_| internal_error(false))? = None;
        let senders = {
            let mut streams = self.streams.lock().map_err(|_| internal_error(false))?;
            streams
                .by_client_msg_id
                .drain()
                .map(|(_, sender)| sender)
                .collect::<Vec<_>>()
        };
        for sender in senders {
            let _ = sender.send(());
        }
        Ok(())
    }

    pub fn create_conversation(
        &self,
        conversation_id: String,
        title: String,
        active_language: String,
        study_mode_default: bool,
        listener: Arc<dyn SyncListener>,
    ) -> Result<(), CoreError> {
        if conversation_id.trim().is_empty() || title.trim().is_empty() {
            return Err(validation_error("Conversation ID and title are required"));
        }
        let (token, base_url) = self.session()?;
        let client = self.client.clone();
        let db = Arc::clone(&self.db);
        let changes = Arc::clone(&self.changes);
        let handle = self.runtime.clone();
        self.runtime.spawn(async move {
            let result = async {
                let conversation: ConversationDto = request_json(
                    &client,
                    &base_url,
                    &token,
                    Method::POST,
                    "/v1/conversations",
                    None,
                    Some(json!({
                        "conversation_id": conversation_id,
                        "title": title,
                        "active_language": active_language,
                        "study_mode_default": study_mode_default,
                    })),
                )
                .await?;
                let summary = conversation.into_summary()?;
                db.upsert_conversation(&summary)?;
                changes.dispatch(&handle, LocalEntity::Conversation, summary.id);
                Ok::<(), CoreError>(())
            }
            .await;
            notify_sync_listener(listener, result);
        });
        Ok(())
    }

    pub fn update_conversation(
        &self,
        conversation_id: String,
        title: Option<String>,
        active_language: Option<String>,
        study_mode_default: Option<bool>,
        listener: Arc<dyn SyncListener>,
    ) -> Result<(), CoreError> {
        if title.is_none() && active_language.is_none() && study_mode_default.is_none() {
            return Err(validation_error(
                "At least one conversation field is required",
            ));
        }
        let (token, base_url) = self.session()?;
        let client = self.client.clone();
        let db = Arc::clone(&self.db);
        let changes = Arc::clone(&self.changes);
        let handle = self.runtime.clone();
        self.runtime.spawn(async move {
            let result = async {
                let mut body = Map::new();
                if let Some(title) = title {
                    body.insert("title".to_string(), Value::String(title));
                }
                if let Some(language) = active_language {
                    body.insert("active_language".to_string(), Value::String(language));
                }
                if let Some(study_mode) = study_mode_default {
                    body.insert("study_mode_default".to_string(), Value::Bool(study_mode));
                }
                let path = format!("/v1/conversations/{conversation_id}");
                let conversation: ConversationDto = request_json(
                    &client,
                    &base_url,
                    &token,
                    Method::PATCH,
                    &path,
                    None,
                    Some(Value::Object(body)),
                )
                .await?;
                let summary = conversation.into_summary()?;
                db.upsert_conversation(&summary)?;
                changes.dispatch(&handle, LocalEntity::Conversation, summary.id);
                Ok::<(), CoreError>(())
            }
            .await;
            notify_sync_listener(listener, result);
        });
        Ok(())
    }

    pub fn delete_conversation(
        &self,
        conversation_id: String,
        listener: Arc<dyn SyncListener>,
    ) -> Result<(), CoreError> {
        let (token, base_url) = self.session()?;
        let client = self.client.clone();
        let db = Arc::clone(&self.db);
        let changes = Arc::clone(&self.changes);
        let handle = self.runtime.clone();
        self.runtime.spawn(async move {
            let result = async {
                let path = format!("/v1/conversations/{conversation_id}");
                let _: Value = request_json(
                    &client,
                    &base_url,
                    &token,
                    Method::DELETE,
                    &path,
                    None,
                    None,
                )
                .await?;
                db.delete_conversation_local(&conversation_id)?;
                changes.dispatch(&handle, LocalEntity::Conversation, conversation_id);
                Ok::<(), CoreError>(())
            }
            .await;
            notify_sync_listener(listener, result);
        });
        Ok(())
    }

    pub fn sync_history(
        &self,
        config: HistorySyncConfig,
        listener: Arc<dyn SyncListener>,
    ) -> Result<(), CoreError> {
        let config = validate_sync_config(config)?;
        let (token, base_url) = self.session()?;
        let client = self.client.clone();
        let db = Arc::clone(&self.db);
        let changes = Arc::clone(&self.changes);
        let handle = self.runtime.clone();
        self.runtime.spawn(async move {
            let context = OperationContext {
                client: &client,
                base_url: &base_url,
                token: &token,
                db: &db,
                changes: &changes,
                handle: &handle,
            };
            let result = sync_history(&context, config).await;
            notify_sync_listener(listener, result);
        });
        Ok(())
    }

    pub fn sync_conversation(
        &self,
        conversation_id: String,
        config: HistorySyncConfig,
        listener: Arc<dyn SyncListener>,
    ) -> Result<(), CoreError> {
        let config = validate_sync_config(config)?;
        let (token, base_url) = self.session()?;
        let client = self.client.clone();
        let db = Arc::clone(&self.db);
        let changes = Arc::clone(&self.changes);
        let handle = self.runtime.clone();
        self.runtime.spawn(async move {
            let context = OperationContext {
                client: &client,
                base_url: &base_url,
                token: &token,
                db: &db,
                changes: &changes,
                handle: &handle,
            };
            let result = sync_conversation(
                &context,
                &conversation_id,
                config.message_page_size,
                config.offline_recent_messages_per_conversation,
            )
            .await;
            notify_sync_listener(listener, result);
        });
        Ok(())
    }

    pub fn list_conversations(
        &self,
        query: Option<String>,
        limit: u32,
        before_updated_at_us: Option<i64>,
        before_id: Option<String>,
    ) -> Result<Vec<ConversationSummary>, CoreError> {
        self.db.list_conversations(
            query.as_deref(),
            limit,
            before_updated_at_us,
            before_id.as_deref(),
            false,
        )
    }

    pub fn list_messages(
        &self,
        conversation_id: String,
        limit: u32,
        before_created_at_us: Option<i64>,
        before_id: Option<String>,
    ) -> Result<Vec<LocalMessage>, CoreError> {
        self.db.list_messages(
            &conversation_id,
            limit,
            before_created_at_us,
            before_id.as_deref(),
        )
    }

    pub fn save_draft(&self, conversation_id: String, text: String) -> Result<(), CoreError> {
        self.db.save_draft(&conversation_id, &text)?;
        self.changes
            .dispatch(&self.runtime, LocalEntity::Draft, conversation_id);
        Ok(())
    }

    pub fn get_draft(&self, conversation_id: String) -> Result<Option<String>, CoreError> {
        self.db.get_draft(&conversation_id)
    }

    pub fn outbox_state(
        &self,
        conversation_id: Option<String>,
    ) -> Result<Vec<OutboxStatus>, CoreError> {
        self.db.outbox_state(conversation_id.as_deref())
    }

    pub fn subscribe_local_changes(
        &self,
        listener: Arc<dyn LocalChangeListener>,
    ) -> Result<Arc<LocalChangeSubscription>, CoreError> {
        Ok(self.changes.subscribe(listener))
    }

    pub fn queue_message(&self, request: OutboundRequest) -> Result<LocalMessage, CoreError> {
        validate::validate_client_msg_id(&request.client_msg_id)?;
        validate::validate_message_content(&request.content)?;
        let message = self.db.queue_message(&request)?;
        self.changes.dispatch(
            &self.runtime,
            LocalEntity::Message,
            request.client_msg_id.clone(),
        );
        self.changes
            .dispatch(&self.runtime, LocalEntity::Outbox, request.client_msg_id);
        Ok(message)
    }

    pub fn stream_message(
        &self,
        request: OutboundRequest,
        listener: Arc<dyn StreamListener>,
    ) -> Result<(), CoreError> {
        validate::validate_client_msg_id(&request.client_msg_id)?;
        validate::validate_message_content(&request.content)?;
        let (token, base_url) = self.session()?;
        let (cancel_sender, cancel_receiver) = broadcast::channel::<()>(1);
        {
            let mut streams = self.streams.lock().map_err(|_| internal_error(false))?;
            if streams
                .active_conversations
                .contains_key(&request.conversation_id)
                || streams
                    .by_client_msg_id
                    .contains_key(&request.client_msg_id)
            {
                return Err(CoreError::Conflict {
                    details: "A stream is already active for this conversation".to_string(),
                });
            }
            streams.active_conversations.insert(
                request.conversation_id.clone(),
                request.client_msg_id.clone(),
            );
            streams
                .by_client_msg_id
                .insert(request.client_msg_id.clone(), cancel_sender);
        }
        if let Err(error) = self.db.begin_stream(&request) {
            cleanup_stream(
                &self.streams,
                &request.client_msg_id,
                &request.conversation_id,
            );
            return Err(error);
        }

        self.changes.dispatch(
            &self.runtime,
            LocalEntity::Message,
            request.client_msg_id.clone(),
        );
        self.changes.dispatch(
            &self.runtime,
            LocalEntity::Outbox,
            request.client_msg_id.clone(),
        );

        let client = self.client.clone();
        let db = Arc::clone(&self.db);
        let streams = Arc::clone(&self.streams);
        let changes = Arc::clone(&self.changes);
        let handle = self.runtime.clone();
        let client_msg_id = request.client_msg_id.clone();
        let conversation_id = request.conversation_id.clone();
        self.runtime.spawn(async move {
            let context = OperationContext {
                client: &client,
                base_url: &base_url,
                token: &token,
                db: &db,
                changes: &changes,
                handle: &handle,
            };
            let result = run_stream(&context, &request, listener.as_ref(), cancel_receiver).await;
            if let Err(error) = result {
                let cancelled = matches!(&error, CoreError::Cancelled { .. });
                let _ = db.fail_stream(&client_msg_id, cancelled);
                changes.dispatch(&handle, LocalEntity::Message, client_msg_id.clone());
                changes.dispatch(&handle, LocalEntity::Outbox, client_msg_id.clone());
                listener.on_error(client_msg_id.clone(), error);
            }
            cleanup_stream(&streams, &client_msg_id, &conversation_id);
        });
        Ok(())
    }

    pub fn cancel_stream(&self, client_msg_id: String) -> bool {
        let sender = self
            .streams
            .lock()
            .ok()
            .and_then(|mut streams| streams.by_client_msg_id.remove(&client_msg_id));
        sender
            .map(|sender| sender.send(()).is_ok())
            .unwrap_or(false)
    }
}

impl RuoCore {
    fn session(&self) -> Result<(String, String), CoreError> {
        let token = self
            .access_token
            .read()
            .map_err(|_| internal_error(false))?
            .clone()
            .ok_or_else(|| CoreError::Unauthenticated {
                details: "Sign-in is required".to_string(),
            })?;
        let base_url = self
            .base_url
            .read()
            .map_err(|_| internal_error(false))?
            .clone()
            .ok_or_else(|| CoreError::Unauthenticated {
                details: "Sign-in is required".to_string(),
            })?;
        Ok((token, base_url))
    }
}

#[derive(Deserialize)]
struct ConversationDto {
    id: String,
    title: String,
    active_language: String,
    study_mode_default: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl ConversationDto {
    fn into_summary(self) -> Result<ConversationSummary, CoreError> {
        Ok(ConversationSummary {
            id: self.id,
            title: self.title,
            active_language: self.active_language,
            study_mode_default: self.study_mode_default,
            created_at_ms: self.created_at.timestamp_millis(),
            updated_at_ms: self.updated_at.timestamp_millis(),
            updated_at_us: self.updated_at.timestamp_micros(),
            deleted_at_ms: self.deleted_at.map(|date| date.timestamp_millis()),
        })
    }
}

#[derive(Deserialize)]
struct MessageDto {
    id: String,
    conversation_id: String,
    client_msg_id: String,
    parent_message_id: Option<String>,
    role: String,
    content_md: String,
    citations_json: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
}

struct OperationContext<'a> {
    client: &'a Client,
    base_url: &'a str,
    token: &'a str,
    db: &'a DbHandle,
    changes: &'a ChangeBus,
    handle: &'a Handle,
}

impl TryFrom<MessageDto> for LocalMessage {
    type Error = CoreError;

    fn try_from(message: MessageDto) -> Result<Self, Self::Error> {
        let role =
            MessageRole::try_from(message.role.as_str()).map_err(|_| CoreError::Internal {
                details: "Server returned an unsupported message role".to_string(),
                retryable: false,
                retry_after_sec: None,
            })?;
        let status = match message.status.as_str() {
            "complete" => LocalMessageStatus::Sent,
            "streaming" => LocalMessageStatus::Streaming,
            "failed" => LocalMessageStatus::Failed,
            _ => LocalMessageStatus::Failed,
        };
        let client_msg_id = match role {
            MessageRole::User => message.client_msg_id.clone(),
            MessageRole::Assistant => message
                .client_msg_id
                .strip_prefix("asst_")
                .unwrap_or(&message.client_msg_id)
                .to_string(),
        };
        let local_id = match role {
            MessageRole::User => client_msg_id.clone(),
            MessageRole::Assistant => format!("assistant:{client_msg_id}"),
        };
        Ok(Self {
            id: local_id,
            server_id: Some(message.id),
            conversation_id: message.conversation_id,
            client_msg_id: Some(client_msg_id),
            role,
            content_md: message.content_md,
            status,
            citations_json: message.citations_json,
            parent_message_id: message.parent_message_id,
            created_at_ms: message.created_at.timestamp_millis(),
            created_at_us: message.created_at.timestamp_micros(),
        })
    }
}

#[derive(Default, Deserialize)]
struct ErrorPayload {
    code: Option<String>,
    retryable: Option<bool>,
    retry_after_sec: Option<u64>,
}

async fn request_json<T: DeserializeOwned>(
    client: &Client,
    base_url: &str,
    token: &str,
    method: Method,
    path: &str,
    query: Option<Vec<(String, String)>>,
    body: Option<Value>,
) -> Result<T, CoreError> {
    let url = format!("{base_url}{path}");
    let mut request = client.request(method, url).bearer_auth(token);
    if let Some(query) = query {
        request = request.query(&query);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.map_err(|_| CoreError::Network {
        details: "Network request failed".to_string(),
        retryable: true,
        retry_after_sec: None,
    })?;
    decode_response(response).await
}

async fn decode_response<T: DeserializeOwned>(response: Response) -> Result<T, CoreError> {
    if !response.status().is_success() {
        return Err(response_error(response).await);
    }
    response.json().await.map_err(|_| CoreError::Internal {
        details: "Server response could not be decoded".to_string(),
        retryable: false,
        retry_after_sec: None,
    })
}

async fn response_error(response: Response) -> CoreError {
    let status = response.status();
    if status == StatusCode::UNAUTHORIZED {
        return CoreError::Unauthenticated {
            details: "Sign-in is required".to_string(),
        };
    }
    let payload = response.json::<ErrorPayload>().await.unwrap_or_default();
    let code = payload
        .code
        .as_deref()
        .unwrap_or(if status == StatusCode::TOO_MANY_REQUESTS {
            "RATE_LIMITED"
        } else {
            "INTERNAL"
        });
    CoreError::from_code(
        code,
        payload.retryable.unwrap_or(false),
        payload.retry_after_sec,
    )
}

async fn sync_history(
    context: &OperationContext<'_>,
    config: HistorySyncConfig,
) -> Result<(), CoreError> {
    let page_size = config.conversation_page_size.clamp(1, 100);
    let mut cursor: Option<(String, String)> = None;
    let mut recent = Vec::new();
    loop {
        let mut query = vec![
            ("limit".to_string(), page_size.to_string()),
            ("include_deleted".to_string(), "true".to_string()),
        ];
        if let Some((updated_at, id)) = cursor.take() {
            query.push(("before_updated_at".to_string(), updated_at));
            query.push(("before_id".to_string(), id));
        }
        let page: Vec<ConversationDto> = request_json(
            context.client,
            context.base_url,
            context.token,
            Method::GET,
            "/v1/conversations",
            Some(query),
            None,
        )
        .await?;
        if page.is_empty() {
            break;
        }
        let page_len = page.len();
        let mut last_cursor = None;
        for conversation in page {
            let summary = conversation.into_summary()?;
            if summary.deleted_at_ms.is_none()
                && recent.len() < config.offline_recent_conversations as usize
            {
                recent.push(summary.id.clone());
            }
            context.db.upsert_conversation(&summary)?;
            context.changes.dispatch(
                context.handle,
                LocalEntity::Conversation,
                summary.id.clone(),
            );
            last_cursor = Some((
                DateTime::<Utc>::from_timestamp_micros(summary.updated_at_us)
                    .ok_or_else(|| internal_error(false))?
                    .to_rfc3339_opts(SecondsFormat::Micros, true),
                summary.id,
            ));
        }
        if page_len < page_size as usize {
            break;
        }
        cursor = last_cursor;
    }

    for conversation_id in recent {
        sync_conversation(
            context,
            &conversation_id,
            config.message_page_size,
            config.offline_recent_messages_per_conversation,
        )
        .await?;
    }
    Ok(())
}

async fn sync_conversation(
    context: &OperationContext<'_>,
    conversation_id: &str,
    page_size: u32,
    message_limit: u32,
) -> Result<(), CoreError> {
    let mut cursor: Option<(String, String)> = None;
    let mut remaining = message_limit;
    while remaining > 0 {
        let limit = page_size.clamp(1, 100).min(remaining);
        let mut query = vec![("limit".to_string(), limit.to_string())];
        if let Some((created_at, id)) = cursor.take() {
            query.push(("before_created_at".to_string(), created_at));
            query.push(("before_id".to_string(), id));
        }
        let path = format!("/v1/conversations/{conversation_id}/messages");
        let page: Vec<MessageDto> = request_json(
            context.client,
            context.base_url,
            context.token,
            Method::GET,
            &path,
            Some(query),
            None,
        )
        .await?;
        if page.is_empty() {
            break;
        }
        let page_len = page.len();
        let mut last_cursor = None;
        for item in page {
            let created_at_us = item.created_at.timestamp_micros();
            let id = item.id.clone();
            let message = LocalMessage::try_from(item)?;
            context.db.upsert_message(&message)?;
            context
                .changes
                .dispatch(context.handle, LocalEntity::Message, message.id);
            last_cursor = Some((
                DateTime::<Utc>::from_timestamp_micros(created_at_us)
                    .ok_or_else(|| internal_error(false))?
                    .to_rfc3339_opts(SecondsFormat::Micros, true),
                id,
            ));
        }
        remaining = remaining.saturating_sub(page_len as u32);
        if page_len < limit as usize {
            break;
        }
        cursor = last_cursor;
    }
    Ok(())
}

async fn run_stream(
    context: &OperationContext<'_>,
    request: &OutboundRequest,
    listener: &dyn StreamListener,
    mut cancel_rx: broadcast::Receiver<()>,
) -> Result<(), CoreError> {
    let path = format!("/v1/conversations/{}/messages", request.conversation_id);
    let body = json!({
        "client_msg_id": request.client_msg_id,
        "content": request.content,
        "parent_message_id": request.parent_message_id,
        "attachment_ids": request.attachment_ids,
        "intent": request.intent,
        "study_mode": request.study_mode,
    });
    let response = tokio::select! {
        _ = cancel_rx.recv() => return Err(cancelled_error()),
        response = context.client.post(format!("{}{path}", context.base_url))
            .bearer_auth(context.token)
            .json(&body)
            .send() => response.map_err(|_| CoreError::Network {
                details: "Network request failed".to_string(),
                retryable: true,
                retry_after_sec: None,
            })?,
    };
    if !response.status().is_success() {
        return Err(response_error(response).await);
    }

    let mut stream = response.bytes_stream();
    let mut parser = SseParser::new();
    let mut full_response = String::new();
    loop {
        let chunk = tokio::select! {
            _ = cancel_rx.recv() => return Err(cancelled_error()),
            chunk = stream.next() => chunk,
        };
        match chunk {
            Some(Ok(bytes)) => {
                for event in parser.feed_bytes(&bytes) {
                    match event {
                        ParsedEvent::Meta {
                            client_msg_id,
                            user_message_id,
                            assistant_message_id,
                            detected_language,
                            study_step,
                            fallback,
                            replayed,
                        } => {
                            if client_msg_id != request.client_msg_id {
                                return Err(CoreError::Internal {
                                    details: "Server returned a mismatched stream ID".to_string(),
                                    retryable: false,
                                    retry_after_sec: None,
                                });
                            }
                            context.db.set_server_message_ids(
                                &request.client_msg_id,
                                &user_message_id,
                                &assistant_message_id,
                            )?;
                            context.changes.dispatch(
                                context.handle,
                                LocalEntity::Message,
                                request.client_msg_id.clone(),
                            );
                            listener.on_meta(
                                request.client_msg_id.clone(),
                                StreamMeta {
                                    user_server_id: user_message_id,
                                    assistant_server_id: assistant_message_id,
                                    detected_language,
                                    study_step,
                                    fallback,
                                    replayed,
                                },
                            );
                        }
                        ParsedEvent::Delta { text } => {
                            full_response.push_str(&text);
                            context
                                .db
                                .update_assistant_delta(&request.client_msg_id, &full_response)?;
                            context.changes.dispatch(
                                context.handle,
                                LocalEntity::Message,
                                request.client_msg_id.clone(),
                            );
                            listener.on_delta(request.client_msg_id.clone(), text);
                        }
                        ParsedEvent::Done {
                            assistant_message_id,
                            replayed,
                        } => {
                            context.db.finish_stream(
                                &request.client_msg_id,
                                &assistant_message_id,
                                &full_response,
                            )?;
                            context.changes.dispatch(
                                context.handle,
                                LocalEntity::Message,
                                request.client_msg_id.clone(),
                            );
                            context.changes.dispatch(
                                context.handle,
                                LocalEntity::Outbox,
                                request.client_msg_id.clone(),
                            );
                            listener.on_done(
                                request.client_msg_id.clone(),
                                assistant_message_id,
                                replayed,
                            );
                            return Ok(());
                        }
                        ParsedEvent::Error {
                            code,
                            retryable,
                            retry_after_sec,
                        } => {
                            let mut error = CoreError::from_code(&code, retryable, retry_after_sec);
                            if let CoreError::Internal {
                                retryable: value,
                                retry_after_sec: retry_after,
                                ..
                            } = &mut error
                            {
                                *value = retryable;
                                *retry_after = retry_after_sec;
                            }
                            return Err(error);
                        }
                        ParsedEvent::Unknown { .. } => {}
                    }
                }
            }
            Some(Err(_)) => {
                return Err(CoreError::Network {
                    details: "Stream connection failed".to_string(),
                    retryable: true,
                    retry_after_sec: None,
                })
            }
            None => {
                return Err(CoreError::Network {
                    details: "Stream ended before completion".to_string(),
                    retryable: true,
                    retry_after_sec: None,
                })
            }
        }
    }
}

fn cleanup_stream(streams: &Mutex<StreamState>, client_msg_id: &str, conversation_id: &str) {
    if let Ok(mut streams) = streams.lock() {
        streams.by_client_msg_id.remove(client_msg_id);
        if streams
            .active_conversations
            .get(conversation_id)
            .map(String::as_str)
            == Some(client_msg_id)
        {
            streams.active_conversations.remove(conversation_id);
        }
    }
}

fn notify_sync_listener(listener: Arc<dyn SyncListener>, result: Result<(), CoreError>) {
    match result {
        Ok(()) => listener.on_complete(),
        Err(error) => listener.on_error(error),
    }
}

fn validate_sync_config(config: HistorySyncConfig) -> Result<HistorySyncConfig, CoreError> {
    if config.conversation_page_size == 0
        || config.message_page_size == 0
        || config.offline_recent_conversations == 0
        || config.offline_recent_messages_per_conversation == 0
    {
        return Err(validation_error("Sync bounds must be greater than zero"));
    }
    Ok(config)
}

fn validate_base_url(base_url: &str, allow_cleartext_debug: bool) -> Result<String, CoreError> {
    let parsed = reqwest::Url::parse(base_url).map_err(|_| validation_error("Invalid base URL"))?;
    let https = parsed.scheme() == "https";
    let debug_loopback_http = cfg!(debug_assertions)
        && allow_cleartext_debug
        && parsed.scheme() == "http"
        && matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !https && !debug_loopback_http {
        return Err(validation_error(
            "Base URL must use HTTPS outside explicit debug loopback",
        ));
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err(validation_error(
            "Base URL cannot contain a query or fragment",
        ));
    }
    Ok(base_url.trim_end_matches('/').to_string())
}

fn validation_error(details: &str) -> CoreError {
    CoreError::Validation {
        details: details.to_string(),
    }
}

fn internal_error(retryable: bool) -> CoreError {
    CoreError::Internal {
        details: "Core operation failed".to_string(),
        retryable,
        retry_after_sec: None,
    }
}

fn cancelled_error() -> CoreError {
    CoreError::Cancelled {
        details: "The operation was cancelled".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use warp::Filter;

    struct RecordingSyncListener {
        completed: AtomicBool,
        error_code: Mutex<Option<String>>,
    }

    impl SyncListener for RecordingSyncListener {
        fn on_complete(&self) {
            self.completed.store(true, Ordering::SeqCst);
        }

        fn on_error(&self, error: CoreError) {
            if let Ok(mut code) = self.error_code.lock() {
                *code = Some(error.code().to_string());
            }
        }
    }

    struct RecordingStreamListener {
        done: AtomicBool,
        error_code: Mutex<Option<String>>,
    }

    impl StreamListener for RecordingStreamListener {
        fn on_meta(&self, _: String, _: StreamMeta) {}
        fn on_delta(&self, _: String, _: String) {}
        fn on_done(&self, _: String, _: String, _: bool) {
            self.done.store(true, Ordering::SeqCst);
        }
        fn on_error(&self, _: String, error: CoreError) {
            if let Ok(mut code) = self.error_code.lock() {
                *code = Some(error.code().to_string());
            }
        }
    }

    #[test]
    fn core_open_drafts_and_message_queue_use_typed_statuses() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_file = temp_dir.path().join("test_ruo.db");
        let core = RuoCore::open(db_file.to_str().unwrap().to_string(), vec![0x42; 32]).unwrap();
        assert!(matches!(
            core.create_conversation(
                "conv-1".to_string(),
                "Math Review".to_string(),
                "en".to_string(),
                false,
                Arc::new(RecordingSyncListener {
                    completed: AtomicBool::new(false),
                    error_code: Mutex::new(None),
                }),
            ),
            Err(CoreError::Unauthenticated { .. })
        ));
        core.db
            .create_conversation("conv-1", "Math Review", "en", false)
            .unwrap();
        core.save_draft("conv-1".to_string(), "Isang test draft".to_string())
            .unwrap();
        assert_eq!(
            core.get_draft("conv-1".to_string()).unwrap().as_deref(),
            Some("Isang test draft")
        );
        let request = OutboundRequest {
            client_msg_id: "client-id-001".to_string(),
            conversation_id: "conv-1".to_string(),
            content: "Paano mag-factor ng quadratic equation?".to_string(),
            parent_message_id: None,
            attachment_ids: vec![],
            intent: None,
            study_mode: false,
        };
        let queued = core.queue_message(request).unwrap();
        assert_eq!(queued.status, LocalMessageStatus::Queued);
        assert_eq!(queued.role, MessageRole::User);
        assert_eq!(core.outbox_state(None).unwrap().len(), 1);
    }

    #[test]
    fn public_conversation_list_hides_sync_tombstones() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_file = temp_dir.path().join("conversation_tombstones.db");
        let core = RuoCore::open(db_file.to_str().unwrap().to_string(), vec![0x42; 32]).unwrap();
        core.db
            .create_conversation("live", "Live", "en", false)
            .unwrap();
        core.db
            .upsert_conversation(&ConversationSummary {
                id: "deleted".to_string(),
                title: "Deleted".to_string(),
                active_language: "en".to_string(),
                study_mode_default: false,
                created_at_ms: 1,
                updated_at_ms: 2,
                updated_at_us: 2_000,
                deleted_at_ms: Some(2),
            })
            .unwrap();

        let visible = core.list_conversations(None, 50, None, None).unwrap();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, "live");
    }

    #[test]
    fn clear_session_removes_authentication_state() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_file = temp_dir.path().join("clear_session.db");
        let core = RuoCore::open(db_file.to_str().unwrap().to_string(), vec![0x42; 32]).unwrap();
        core.set_session(
            "token".to_string(),
            "https://api.example.test".to_string(),
            false,
        )
        .unwrap();

        core.clear_session().unwrap();

        assert!(matches!(
            core.session(),
            Err(CoreError::Unauthenticated { .. })
        ));
    }

    #[test]
    fn set_session_rejects_non_https_except_explicit_debug_loopback() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_file = temp_dir.path().join("test_ruo.db");
        let core = RuoCore::open(db_file.to_str().unwrap().to_string(), vec![0x42; 32]).unwrap();
        assert!(core
            .set_session("token".to_string(), "http://example.test".to_string(), true)
            .is_err());
        if cfg!(debug_assertions) {
            assert!(core
                .set_session(
                    "token".to_string(),
                    "http://127.0.0.1:8000".to_string(),
                    true
                )
                .is_ok());
        }
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn release_build_rejects_cleartext_even_when_override_is_true() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_file = temp_dir.path().join("release_ruo.db");
        let core = RuoCore::open(db_file.to_str().unwrap().to_string(), vec![0x42; 32]).unwrap();

        assert!(matches!(
            core.set_session(
                "token".to_string(),
                "http://127.0.0.1:8000".to_string(),
                true
            ),
            Err(CoreError::Validation { .. })
        ));
    }

    #[tokio::test]
    async fn server_error_body_echo_is_never_returned_in_core_error() {
        let secret = "user-private-input-that-server-echoed";
        let body = json!({
            "code": "INVALID_TOKEN",
            "message": secret,
            "detail": secret,
        });
        let route = warp::any().map(move || {
            warp::reply::with_status(
                warp::reply::json(&body),
                warp::http::StatusCode::BAD_REQUEST,
            )
        });
        let (address, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
        let server_task = tokio::spawn(server);
        let response = Client::new()
            .get(format!("http://{address}"))
            .send()
            .await
            .unwrap();
        let error = response_error(response).await;
        server_task.abort();
        assert_eq!(error.code(), "UNAUTHENTICATED");
        assert!(!format!("{error:?}").contains(secret));
    }

    #[tokio::test]
    async fn unauthorized_response_is_unauthenticated_without_server_body() {
        let secret = "unauthorized-response-echo";
        let body = json!({
            "code": "UNAUTHENTICATED",
            "message": secret,
            "detail": secret,
        });
        let route = warp::any().map(move || {
            warp::reply::with_status(
                warp::reply::json(&body),
                warp::http::StatusCode::UNAUTHORIZED,
            )
        });
        let (address, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
        let server_task = tokio::spawn(server);
        let response = Client::new()
            .get(format!("http://{address}"))
            .send()
            .await
            .unwrap();
        let error = response_error(response).await;
        server_task.abort();

        assert!(matches!(
            error,
            CoreError::Unauthenticated { ref details } if details == "Sign-in is required"
        ));
        assert_eq!(error.code(), "UNAUTHENTICATED");
        assert!(!format!("{error:?}").contains(secret));
    }

    #[tokio::test]
    async fn history_sync_requests_and_persists_server_tombstones() {
        use std::collections::HashMap;

        let requested_tombstones = Arc::new(AtomicBool::new(false));
        let requested_tombstones_route = Arc::clone(&requested_tombstones);
        let conversations = warp::path!("v1" / "conversations")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .map(move |query: HashMap<String, String>| {
                requested_tombstones_route.store(
                    query.get("include_deleted").map(String::as_str) == Some("true"),
                    Ordering::SeqCst,
                );
                warp::reply::json(&json!([
                    {
                        "id": "deleted",
                        "title": "Deleted",
                        "active_language": "en",
                        "study_mode_default": false,
                        "created_at": "2026-01-01T00:00:00Z",
                        "updated_at": "2026-01-03T00:00:00Z",
                        "deleted_at": "2026-01-03T00:00:00Z"
                    },
                    {
                        "id": "live",
                        "title": "Live",
                        "active_language": "en",
                        "study_mode_default": false,
                        "created_at": "2026-01-01T00:00:00Z",
                        "updated_at": "2026-01-02T00:00:00Z",
                        "deleted_at": null
                    }
                ]))
            });
        let messages = warp::path!("v1" / "conversations" / String / "messages")
            .and(warp::get())
            .map(|_| warp::reply::json(&json!([])));
        let (address, server) =
            warp::serve(conversations.or(messages)).bind_ephemeral(([127, 0, 0, 1], 0));
        let server_task = tokio::spawn(server);
        let temp_dir = tempfile::tempdir().unwrap();
        let db_file = temp_dir.path().join("history_sync.db");
        let core = RuoCore::open(db_file.to_str().unwrap().to_string(), vec![0x42; 32]).unwrap();
        core.set_session("test-token".to_string(), format!("http://{address}"), true)
            .unwrap();
        let listener = Arc::new(RecordingSyncListener {
            completed: AtomicBool::new(false),
            error_code: Mutex::new(None),
        });

        core.sync_history(
            HistorySyncConfig {
                conversation_page_size: 10,
                message_page_size: 10,
                offline_recent_conversations: 10,
                offline_recent_messages_per_conversation: 10,
            },
            listener.clone(),
        )
        .unwrap();
        for _ in 0..50 {
            if listener.completed.load(Ordering::SeqCst)
                || listener.error_code.lock().unwrap().is_some()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }

        assert!(listener.completed.load(Ordering::SeqCst));
        assert!(listener.error_code.lock().unwrap().is_none());
        assert!(requested_tombstones.load(Ordering::SeqCst));
        let all_local = core
            .db
            .list_conversations(None, 10, None, None, true)
            .unwrap();
        assert_eq!(all_local.len(), 2);
        assert!(all_local.iter().any(|conversation| {
            conversation.id == "deleted" && conversation.deleted_at_ms.is_some()
        }));
        let visible = core.list_conversations(None, 10, None, None).unwrap();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, "live");
        server_task.abort();
    }

    #[tokio::test]
    async fn sync_history_respects_configured_offline_cache_bounds() {
        use std::collections::HashMap;

        let requested_messages = Arc::new(Mutex::new(Vec::new()));
        let requested_messages_route = Arc::clone(&requested_messages);
        let request_limit_is_one = Arc::new(AtomicBool::new(false));
        let request_limit_route = Arc::clone(&request_limit_is_one);
        let conversations = warp::path!("v1" / "conversations")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .map(|_: HashMap<String, String>| {
                warp::reply::json(&json!([
                    {
                        "id": "recent",
                        "title": "Recent",
                        "active_language": "en",
                        "study_mode_default": false,
                        "created_at": "2026-01-01T00:00:00Z",
                        "updated_at": "2026-01-03T00:00:00Z",
                        "deleted_at": null
                    },
                    {
                        "id": "older",
                        "title": "Older",
                        "active_language": "en",
                        "study_mode_default": false,
                        "created_at": "2026-01-01T00:00:00Z",
                        "updated_at": "2026-01-02T00:00:00Z",
                        "deleted_at": null
                    }
                ]))
            });
        let messages = warp::path!("v1" / "conversations" / String / "messages")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .map(
                move |conversation_id: String, query: HashMap<String, String>| {
                    requested_messages_route
                        .lock()
                        .unwrap()
                        .push(conversation_id.clone());
                    request_limit_route.store(
                        query.get("limit").map(String::as_str) == Some("1"),
                        Ordering::SeqCst,
                    );
                    warp::reply::json(&json!([{
                        "id": format!("server-{conversation_id}"),
                        "conversation_id": conversation_id,
                        "client_msg_id": format!("client-{conversation_id}"),
                        "parent_message_id": null,
                        "role": "user",
                        "content_md": "cached",
                        "citations_json": null,
                        "status": "complete",
                        "created_at": "2026-01-01T00:00:00Z"
                    }]))
                },
            );
        let (address, server) =
            warp::serve(conversations.or(messages)).bind_ephemeral(([127, 0, 0, 1], 0));
        let server_task = tokio::spawn(server);
        let temp_dir = tempfile::tempdir().unwrap();
        let db_file = temp_dir.path().join("offline_bounds.db");
        let core = RuoCore::open(db_file.to_str().unwrap().to_string(), vec![0x42; 32]).unwrap();
        core.set_session("test-token".to_string(), format!("http://{address}"), true)
            .unwrap();
        let listener = Arc::new(RecordingSyncListener {
            completed: AtomicBool::new(false),
            error_code: Mutex::new(None),
        });

        core.sync_history(
            HistorySyncConfig {
                conversation_page_size: 10,
                message_page_size: 10,
                offline_recent_conversations: 1,
                offline_recent_messages_per_conversation: 1,
            },
            listener.clone(),
        )
        .unwrap();
        for _ in 0..50 {
            if listener.completed.load(Ordering::SeqCst)
                || listener.error_code.lock().unwrap().is_some()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }

        assert!(listener.completed.load(Ordering::SeqCst));
        assert!(listener.error_code.lock().unwrap().is_none());
        assert!(request_limit_is_one.load(Ordering::SeqCst));
        assert_eq!(*requested_messages.lock().unwrap(), vec!["recent"]);
        assert_eq!(
            core.list_messages("recent".to_string(), 10, None, None)
                .unwrap()
                .len(),
            1
        );
        assert!(core
            .list_messages("older".to_string(), 10, None, None)
            .unwrap()
            .is_empty());
        server_task.abort();
    }

    #[test]
    fn sync_configuration_rejects_zero_offline_cache_bounds() {
        let config = HistorySyncConfig {
            conversation_page_size: 1,
            message_page_size: 1,
            offline_recent_conversations: 0,
            offline_recent_messages_per_conversation: 1,
        };
        assert!(matches!(
            validate_sync_config(config),
            Err(CoreError::Validation { .. })
        ));

        let config = HistorySyncConfig {
            conversation_page_size: 1,
            message_page_size: 1,
            offline_recent_conversations: 1,
            offline_recent_messages_per_conversation: 0,
        };
        assert!(matches!(
            validate_sync_config(config),
            Err(CoreError::Validation { .. })
        ));
    }

    #[tokio::test]
    async fn stream_cancel_notifies_listener_and_releases_conversation_slot() {
        let route = warp::any().map(|| {
            let stream = async_stream::stream! {
                yield Ok::<_, warp::Error>(warp::sse::Event::default().event("meta").data(
                    r#"{"client_msg_id":"client-1","user_message_id":"user-1","assistant_message_id":"assistant-1"}"#,
                ));
                futures_util::future::pending::<()>().await;
            };
            warp::reply::with_header(
                warp::sse::reply(warp::sse::keep_alive().stream(stream)),
                "cache-control",
                "no-cache",
            )
        });
        let (address, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
        let server_task = tokio::spawn(server);
        let temp_dir = tempfile::tempdir().unwrap();
        let db_file = temp_dir.path().join("stream.db");
        let core = RuoCore::open(db_file.to_str().unwrap().to_string(), vec![0x42; 32]).unwrap();
        core.db
            .create_conversation("conv-1", "Study", "en", false)
            .unwrap();
        core.set_session("test-token".to_string(), format!("http://{address}"), true)
            .unwrap();
        let listener = Arc::new(RecordingStreamListener {
            done: AtomicBool::new(false),
            error_code: Mutex::new(None),
        });
        core.stream_message(
            OutboundRequest {
                client_msg_id: "client-1".to_string(),
                conversation_id: "conv-1".to_string(),
                content: "hello".to_string(),
                parent_message_id: None,
                attachment_ids: vec![],
                intent: None,
                study_mode: false,
            },
            listener.clone(),
        )
        .unwrap();
        assert!(matches!(
            core.stream_message(
                OutboundRequest {
                    client_msg_id: "client-2".to_string(),
                    conversation_id: "conv-1".to_string(),
                    content: "second stream".to_string(),
                    parent_message_id: None,
                    attachment_ids: vec![],
                    intent: None,
                    study_mode: false,
                },
                listener.clone(),
            ),
            Err(CoreError::Conflict { .. })
        ));
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(core.cancel_stream("client-1".to_string()));
        for _ in 0..20 {
            if listener.error_code.lock().unwrap().as_deref() == Some("CANCELLED") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(
            listener.error_code.lock().unwrap().as_deref(),
            Some("CANCELLED")
        );
        assert!(!listener.done.load(Ordering::SeqCst));
        assert!(!core.cancel_stream("client-1".to_string()));
        server_task.abort();
    }

    #[tokio::test]
    async fn stream_persists_assistant_message_after_done() {
        let route = warp::any().map(|| {
            let stream = async_stream::stream! {
                yield Ok::<_, warp::Error>(warp::sse::Event::default().event("meta").data(
                    r#"{"client_msg_id":"client-done","user_message_id":"user-1","assistant_message_id":"assistant-1"}"#,
                ));
                yield Ok::<_, warp::Error>(warp::sse::Event::default().event("delta").data(
                    r#"{"text":"answer"}"#,
                ));
                yield Ok::<_, warp::Error>(warp::sse::Event::default().event("done").data(
                    r#"{"assistant_message_id":"assistant-1","replayed":false}"#,
                ));
            };
            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        });
        let (address, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
        let server_task = tokio::spawn(server);
        let temp_dir = tempfile::tempdir().unwrap();
        let db_file = temp_dir.path().join("stream_done.db");
        let core = RuoCore::open(db_file.to_str().unwrap().to_string(), vec![0x42; 32]).unwrap();
        core.db
            .create_conversation("conv-1", "Study", "en", false)
            .unwrap();
        core.set_session("test-token".to_string(), format!("http://{address}"), true)
            .unwrap();
        let listener = Arc::new(RecordingStreamListener {
            done: AtomicBool::new(false),
            error_code: Mutex::new(None),
        });

        core.stream_message(
            OutboundRequest {
                client_msg_id: "client-done".to_string(),
                conversation_id: "conv-1".to_string(),
                content: "hello".to_string(),
                parent_message_id: None,
                attachment_ids: vec![],
                intent: None,
                study_mode: false,
            },
            listener.clone(),
        )
        .unwrap();

        for _ in 0..50 {
            if listener.done.load(Ordering::SeqCst) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(listener.done.load(Ordering::SeqCst));
        let messages = core.db.list_messages("conv-1", 10, None, None).unwrap();
        let assistant = messages
            .iter()
            .find(|message| {
                message.role == MessageRole::Assistant
                    && message.client_msg_id.as_deref() == Some("client-done")
            })
            .unwrap();
        assert_eq!(assistant.role, MessageRole::Assistant);
        assert_eq!(assistant.status, LocalMessageStatus::Sent);
        assert_eq!(assistant.content_md, "answer");
        server_task.abort();
    }
}
