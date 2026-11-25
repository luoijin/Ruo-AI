package io.luoijin.ruo.core.rustbridge

import android.content.Context
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.channels.SendChannel
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import java.io.File
import java.util.concurrent.ConcurrentHashMap
import kotlin.coroutines.resume

enum class CoreFailureCode {
    UNAUTHENTICATED,
    QUOTA_EXCEEDED,
    RATE_LIMITED,
    NETWORK,
    CANCELLED,
    VALIDATION,
    NOT_FOUND,
    CONFLICT,
    KEY_UNAVAILABLE,
    INVALID_KEY,
    WRONG_KEY,
    DATABASE_CORRUPT,
    SCHEMA_TOO_NEW,
    INTERNAL,
}

data class SafeCoreError(
    val code: CoreFailureCode,
    val retryable: Boolean,
    val retryAfterSec: Long?,
)

sealed interface LocalChangeEvent {
    data class Changed(val value: LocalChange) : LocalChangeEvent
    data class Error(val failure: SafeCoreError) : LocalChangeEvent
}

sealed interface StreamEvent {
    data class Meta(val value: StreamMeta) : StreamEvent
    data class Delta(val text: String) : StreamEvent
    data class Done(val assistantServerId: String, val replayed: Boolean) : StreamEvent
    data class Error(val failure: SafeCoreError) : StreamEvent
}

class RuoClient private constructor(
    private val databaseFile: File,
    private val keyManager: DatabaseKeyManager,
    private val callbackDispatcher: kotlinx.coroutines.CoroutineDispatcher,
    private val core: RuoCore,
) : AutoCloseable {
    private val callbackScope = CoroutineScope(SupervisorJob())
    private val localChangeChannels =
        ConcurrentHashMap.newKeySet<SendChannel<LocalChangeEvent>>()
    private val streamChannels = ConcurrentHashMap.newKeySet<SendChannel<StreamEvent>>()
    private val subscriptions = ConcurrentHashMap.newKeySet<LocalChangeSubscription>()
    @Volatile
    private var closed = false

    fun setSession(accessToken: String, baseUrl: String, allowCleartextDebug: Boolean) {
        checkOpen()
        core.setSession(accessToken, baseUrl, allowCleartextDebug)
    }

    suspend fun saveDraft(conversationId: String, text: String) = withContext(Dispatchers.IO) {
        checkOpen()
        core.saveDraft(conversationId, text)
    }

    suspend fun getDraft(conversationId: String): String? = withContext(Dispatchers.IO) {
        checkOpen()
        core.getDraft(conversationId)
    }

    suspend fun queueMessage(request: OutboundRequest): LocalMessage = withContext(Dispatchers.IO) {
        checkOpen()
        core.queueMessage(request)
    }

    suspend fun listConversations(
        query: String? = null,
        limit: UInt = 50u,
        beforeUpdatedAtUs: Long? = null,
        beforeId: String? = null,
        includeDeleted: Boolean = false,
    ): List<ConversationSummary> = withContext(Dispatchers.IO) {
        checkOpen()
        core.listConversations(query, limit, beforeUpdatedAtUs, beforeId, includeDeleted)
    }

    suspend fun listMessages(
        conversationId: String,
        limit: UInt = 50u,
        beforeCreatedAtUs: Long? = null,
        beforeId: String? = null,
    ): List<LocalMessage> = withContext(Dispatchers.IO) {
        checkOpen()
        core.listMessages(conversationId, limit, beforeCreatedAtUs, beforeId)
    }

    suspend fun outboxState(conversationId: String? = null): List<OutboxStatus> =
        withContext(Dispatchers.IO) {
        checkOpen()
        core.outboxState(conversationId)
    }

    fun localChanges(): Flow<LocalChangeEvent> = callbackFlow {
        checkOpen()
        localChangeChannels.add(channel)
        val subscription = try {
            core.subscribeLocalChanges(
                object : LocalChangeListener {
                    override fun onChanged(change: LocalChange) {
                        callbackScope.launch(callbackDispatcher) {
                            trySend(LocalChangeEvent.Changed(change))
                        }
                    }
                },
            )
        } catch (error: CoreException) {
            trySend(LocalChangeEvent.Error(error.toSafeError()))
            close()
            return@callbackFlow
        }
        subscriptions.add(subscription)
        awaitClose {
            localChangeChannels.remove(channel)
            subscriptions.remove(subscription)
            subscription.unsubscribe()
        }
    }

    fun streamMessage(request: OutboundRequest): Flow<StreamEvent> = callbackFlow {
        checkOpen()
        streamChannels.add(channel)
        val listener = object : StreamListener {
            override fun onMeta(clientMsgId: String, meta: StreamMeta) {
                callbackScope.launch(callbackDispatcher) {
                    trySend(StreamEvent.Meta(meta))
                }
            }

            override fun onDelta(clientMsgId: String, text: String) {
                callbackScope.launch(callbackDispatcher) {
                    trySend(StreamEvent.Delta(text))
                }
            }

            override fun onDone(
                clientMsgId: String,
                assistantServerId: String,
                replayed: Boolean,
            ) {
                callbackScope.launch(callbackDispatcher) {
                    trySend(StreamEvent.Done(assistantServerId, replayed))
                    close()
                }
            }

            override fun onError(clientMsgId: String, error: CoreException) {
                callbackScope.launch(callbackDispatcher) {
                    trySend(StreamEvent.Error(error.toSafeError()))
                    close()
                }
            }
        }
        try {
            core.streamMessage(request, listener)
        } catch (error: CoreException) {
            trySend(StreamEvent.Error(error.toSafeError()))
            close()
        }
        awaitClose {
            streamChannels.remove(channel)
            core.cancelStream(request.clientMsgId)
        }
    }

    fun cancelStream(clientMsgId: String): Boolean {
        checkOpen()
        return core.cancelStream(clientMsgId)
    }

    suspend fun createConversation(
        conversationId: String,
        title: String,
        activeLanguage: String,
        studyModeDefault: Boolean,
    ): SafeCoreError? = awaitSync { listener ->
        core.createConversation(
            conversationId,
            title,
            activeLanguage,
            studyModeDefault,
            listener,
        )
    }

    suspend fun updateConversation(
        conversationId: String,
        title: String?,
        activeLanguage: String?,
        studyModeDefault: Boolean?,
    ): SafeCoreError? = awaitSync { listener ->
        core.updateConversation(
            conversationId,
            title,
            activeLanguage,
            studyModeDefault,
            listener,
        )
    }

    suspend fun deleteConversation(conversationId: String): SafeCoreError? = awaitSync { listener ->
        core.deleteConversation(conversationId, listener)
    }

    suspend fun syncHistory(config: HistorySyncConfig): SafeCoreError? = awaitSync { listener ->
        core.syncHistory(config, listener)
    }

    suspend fun syncConversation(
        conversationId: String,
        config: HistorySyncConfig,
    ): SafeCoreError? = awaitSync { listener ->
        core.syncConversation(conversationId, config, listener)
    }

    suspend fun signOutAndWipeLocalData() {
        withContext(Dispatchers.IO) {
            close()
            keyManager.wipeLocalData(databaseFile)
        }
    }

    override fun close() {
        if (closed) return
        core.clearSession()
        localChangeChannels.forEach { it.close() }
        streamChannels.forEach { it.close() }
        subscriptions.forEach(LocalChangeSubscription::unsubscribe)
        callbackScope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        core.close()
        closed = true
    }

    private suspend fun awaitSync(
        register: (SyncListener) -> Unit,
    ): SafeCoreError? = suspendCancellableCoroutine { continuation ->
        checkOpen()
        val listener = object : SyncListener {
            override fun onComplete() {
                callbackScope.launch(callbackDispatcher) {
                    if (continuation.isActive) continuation.resume(null)
                }
            }

            override fun onError(error: CoreException) {
                callbackScope.launch(callbackDispatcher) {
                    if (continuation.isActive) continuation.resume(error.toSafeError())
                }
            }
        }
        try {
            register(listener)
        } catch (error: CoreException) {
            if (continuation.isActive) continuation.resume(error.toSafeError())
        }
    }

    private fun checkOpen() {
        check(!closed) { "RuoClient is closed" }
    }

    companion object {
        fun open(
            context: Context,
            callbackDispatcher: kotlinx.coroutines.CoroutineDispatcher = Dispatchers.Main.immediate,
        ): RuoClient {
            val appContext = context.applicationContext
            val databaseFile = File(appContext.noBackupFilesDir, "ruo.db")
            val keyManager = DatabaseKeyManager(appContext)
            val key = keyManager.getOrCreate(databaseFile)
            val core = try {
                RuoCore.open(databaseFile.absolutePath, key.copyOf())
            } finally {
                key.fill(0)
            }
            return RuoClient(databaseFile, keyManager, callbackDispatcher, core)
        }
    }
}

private fun CoreException.toSafeError(): SafeCoreError = when (this) {
    is CoreException.Unauthenticated -> safe(CoreFailureCode.UNAUTHENTICATED)
    is CoreException.QuotaExceeded -> safe(CoreFailureCode.QUOTA_EXCEEDED, false, retryAfterSec)
    is CoreException.RateLimited -> safe(CoreFailureCode.RATE_LIMITED, true, retryAfterSec)
    is CoreException.Network -> safe(CoreFailureCode.NETWORK, retryable, retryAfterSec)
    is CoreException.Cancelled -> safe(CoreFailureCode.CANCELLED)
    is CoreException.Validation -> safe(CoreFailureCode.VALIDATION)
    is CoreException.NotFound -> safe(CoreFailureCode.NOT_FOUND)
    is CoreException.Conflict -> safe(CoreFailureCode.CONFLICT)
    is CoreException.KeyUnavailable -> safe(CoreFailureCode.KEY_UNAVAILABLE)
    is CoreException.InvalidKey -> safe(CoreFailureCode.INVALID_KEY)
    is CoreException.WrongKey -> safe(CoreFailureCode.WRONG_KEY)
    is CoreException.DatabaseCorrupt -> safe(CoreFailureCode.DATABASE_CORRUPT)
    is CoreException.SchemaTooNew -> safe(CoreFailureCode.SCHEMA_TOO_NEW)
    is CoreException.Internal -> safe(CoreFailureCode.INTERNAL, retryable, retryAfterSec)
}

private fun CoreException.safe(
    code: CoreFailureCode,
    retryable: Boolean = false,
    retryAfterSec: ULong? = null,
) = SafeCoreError(code, retryable, retryAfterSec.toSafeLong())

private fun ULong?.toSafeLong(): Long? = when {
    this == null -> null
    this > Long.MAX_VALUE.toULong() -> Long.MAX_VALUE
    else -> toLong()
}
