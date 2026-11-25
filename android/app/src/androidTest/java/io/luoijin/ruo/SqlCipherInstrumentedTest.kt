package io.luoijin.ruo

import android.content.Context
import android.content.pm.ApplicationInfo
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.luoijin.ruo.core.rustbridge.CoreException
import io.luoijin.ruo.core.rustbridge.CoreFailureCode
import io.luoijin.ruo.core.rustbridge.DatabaseKeyManager
import io.luoijin.ruo.core.rustbridge.KeyUnavailableException
import io.luoijin.ruo.core.rustbridge.LocalChangeEvent
import io.luoijin.ruo.core.rustbridge.LocalEntity
import io.luoijin.ruo.core.rustbridge.LocalMessageStatus
import io.luoijin.ruo.core.rustbridge.MessageRole
import io.luoijin.ruo.core.rustbridge.OutboundRequest
import io.luoijin.ruo.core.rustbridge.RuoClient
import io.luoijin.ruo.core.rustbridge.RuoCore
import io.luoijin.ruo.core.rustbridge.StreamEvent
import java.io.BufferedInputStream
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.IOException
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.nio.charset.StandardCharsets
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlinx.coroutines.withTimeoutOrNull
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class SqlCipherInstrumentedTest {
    @Test
    fun encryptedStoreCallbacksAndMidStreamCancellationWorkOnDevice() = runBlocking<Unit> {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        assertFalse(
            (context.applicationInfo.flags and ApplicationInfo.FLAG_ALLOW_BACKUP) != 0,
        )
        val databaseFile = File(context.noBackupFilesDir, "ruo.db")
        val keyManager = DatabaseKeyManager(context)
        keyManager.wipeLocalData(databaseFile)
        val server = LoopbackApiServer()
        val client = RuoClient.open(context, Dispatchers.IO)
        val conversationId = UUID.randomUUID().toString()
        val expected = "Taglish: Kumusta, paano ito? Cebuano: Maayong buntag, unsaon ni? 🌱📚"
        val localChanges = Channel<LocalChangeEvent>(Channel.UNLIMITED)
        var localChangeJob: kotlinx.coroutines.Job? = null
        var clientForCleanup: RuoClient? = client

        try {
            client.setSession("test-access-token", server.baseUrl, allowCleartextDebug = true)
            assertEquals(
                null,
                client.createConversation(conversationId, "Study", "en", false),
            )
            localChangeJob = launch(start = CoroutineStart.UNDISPATCHED) {
                client.localChanges().collect { localChanges.send(it) }
            }

            client.saveDraft(conversationId, expected)
            assertEquals(expected, client.getDraft(conversationId))
            val draftChange = awaitLocalChange(localChanges)
            assertTrue(draftChange is LocalChangeEvent.Changed)
            assertEquals(
                LocalEntity.DRAFT,
                (draftChange as LocalChangeEvent.Changed).value.entity,
            )

            val key = keyManager.getOrCreate(databaseFile)
            try {
                val wrongKey = key.copyOf().also { it[0] = (it[0].toInt() xor 0x7f).toByte() }
                try {
                    assertThrows(CoreException.WrongKey::class.java) {
                        RuoCore.open(databaseFile.absolutePath, wrongKey)
                    }
                    assertThrows(CoreException.InvalidKey::class.java) {
                        RuoCore.open(databaseFile.absolutePath, byteArrayOf())
                    }
                } finally {
                    wrongKey.fill(0)
                }
                val header = databaseFile.readBytes().copyOfRange(0, 16)
                assertFalse(
                    header.contentEquals("SQLite format 3\u0000".toByteArray(StandardCharsets.US_ASCII)),
                )
            } finally {
                key.fill(0)
            }

            localChangeJob.cancelAndJoin()
            localChangeJob = null
            client.close()
            clientForCleanup = null

            val reopened = RuoClient.open(context, Dispatchers.IO)
            clientForCleanup = reopened
            assertEquals(expected, reopened.getDraft(conversationId))
            reopened.setSession("test-access-token", server.baseUrl, allowCleartextDebug = true)

            val events = Channel<StreamEvent>(Channel.UNLIMITED)
            val clientMsgId = UUID.randomUUID().toString()
            val streamJob = launch {
                reopened.streamMessage(
                    OutboundRequest(
                        clientMsgId = clientMsgId,
                        conversationId = conversationId,
                        content = "Taglish: explain this please",
                    ),
                ).collect { events.send(it) }
            }
            assertTrue(awaitStreamEvent(events, "meta") is StreamEvent.Meta)
            assertTrue(awaitStreamEvent(events, "delta") is StreamEvent.Delta)
            val conflictEvents = mutableListOf<StreamEvent>()
            reopened.streamMessage(
                OutboundRequest(
                    clientMsgId = UUID.randomUUID().toString(),
                    conversationId = conversationId,
                    content = "second active stream",
                ),
            ).collect { conflictEvents.add(it) }
            assertEquals(1, conflictEvents.size)
            assertTrue(conflictEvents.single() is StreamEvent.Error)
            assertEquals(
                CoreFailureCode.CONFLICT,
                (conflictEvents.single() as StreamEvent.Error).failure.code,
            )
            assertTrue(reopened.cancelStream(clientMsgId))
            val cancelled = awaitStreamEvent(events, "cancelled error")
            assertTrue(cancelled is StreamEvent.Error)
            assertEquals(
                CoreFailureCode.CANCELLED,
                (cancelled as StreamEvent.Error).failure.code,
            )
            withTimeout(5_000) { streamJob.join() }
            assertFalse(reopened.cancelStream(clientMsgId))

            val completedClientMsgId = UUID.randomUUID().toString()
            val completedEvents = mutableListOf<StreamEvent>()
            reopened.streamMessage(
                OutboundRequest(
                    clientMsgId = completedClientMsgId,
                    conversationId = conversationId,
                    content = "complete this response",
                ),
            ).collect { completedEvents.add(it) }
            assertTrue(completedEvents.any { it is StreamEvent.Done })
            val assistant = reopened.listMessages(conversationId, 20u)
                .first { it.clientMsgId == completedClientMsgId && it.role == MessageRole.ASSISTANT }
            assertEquals(LocalMessageStatus.SENT, assistant.status)
            assertEquals("Maayong buntag 🌱", assistant.contentMd)

            context.getSharedPreferences("ruo_database_key", Context.MODE_PRIVATE)
                .edit()
                .clear()
                .commit()
            assertThrows(KeyUnavailableException::class.java) {
                DatabaseKeyManager(context).getOrCreate(databaseFile)
            }
            reopened.signOutAndWipeLocalData()
            clientForCleanup = null
            assertFalse(databaseFile.exists())
        } finally {
            localChangeJob?.cancelAndJoin()
            clientForCleanup?.signOutAndWipeLocalData()
            server.close()
            localChanges.close()
        }
    }
}

private suspend fun awaitStreamEvent(
    events: Channel<StreamEvent>,
    expected: String,
): StreamEvent = withTimeoutOrNull(5_000) { events.receive() }
    ?: throw AssertionError("Timed out waiting for stream $expected")

private suspend fun awaitLocalChange(
    events: Channel<LocalChangeEvent>,
): LocalChangeEvent = withTimeoutOrNull(5_000) { events.receive() }
    ?: throw AssertionError("Timed out waiting for local draft change callback")

private class LoopbackApiServer : AutoCloseable {
    private val server = ServerSocket(0, 4, InetAddress.getByName("127.0.0.1"))
    private val sockets = ConcurrentHashMap.newKeySet<Socket>()
    @Volatile
    private var running = true
    private val thread = Thread {
        while (running) {
            try {
                val socket = server.accept()
                sockets.add(socket)
                Thread { handle(socket) }.start()
            } catch (_: IOException) {
                if (running) throw IllegalStateException("Loopback test server failed")
            }
        }
    }.apply {
        name = "ruo-loopback-api-test"
        isDaemon = true
        start()
    }

    val baseUrl: String
        get() = "http://127.0.0.1:${server.localPort}"

    private fun handle(socket: Socket) {
        socket.use {
            val input = BufferedInputStream(it.getInputStream())
            val requestLine = readLine(input) ?: return
            val headers = mutableMapOf<String, String>()
            while (true) {
                val line = readLine(input) ?: return
                if (line.isEmpty()) break
                val separator = line.indexOf(':')
                if (separator > 0) {
                    headers[line.substring(0, separator).trim().lowercase()] =
                        line.substring(separator + 1).trim()
                }
            }
            val contentLength = headers["content-length"]?.toIntOrNull() ?: 0
            val bodyBytes = ByteArray(contentLength)
            var read = 0
            while (read < contentLength) {
                val count = input.read(bodyBytes, read, contentLength - read)
                if (count < 0) return
                read += count
            }
            val body = String(bodyBytes, StandardCharsets.UTF_8)
            when {
                requestLine.startsWith("POST /v1/conversations ") -> {
                    val request = JSONObject(body)
                    val now = "2026-11-25T12:00:00Z"
                    writeJson(
                        it,
                        JSONObject()
                            .put("id", request.getString("conversation_id"))
                            .put("title", request.getString("title"))
                            .put("active_language", request.getString("active_language"))
                            .put("study_mode_default", request.getBoolean("study_mode_default"))
                            .put("created_at", now)
                            .put("updated_at", now)
                            .put("deleted_at", JSONObject.NULL)
                            .toString(),
                    )
                }
                requestLine.startsWith("POST /v1/conversations/") &&
                    requestLine.contains("/messages ") -> {
                    val request = JSONObject(body)
                    val clientMsgId = request.getString("client_msg_id")
                    val output = it.getOutputStream()
                    output.write(
                        ("HTTP/1.1 200 OK\r\n" +
                            "Content-Type: text/event-stream\r\n" +
                            "Cache-Control: no-cache\r\n" +
                            "Connection: close\r\n\r\n").toByteArray(StandardCharsets.US_ASCII),
                    )
                    output.write(
                        ("event: meta\ndata: {\"client_msg_id\":\"$clientMsgId\"," +
                            "\"user_message_id\":\"server-user-id\"," +
                            "\"assistant_message_id\":\"server-assistant-id\"}\n\n").toByteArray(),
                    )
                    output.write("event: delta\ndata: {\"text\":\"Maayong buntag 🌱\"}\n\n".toByteArray())
                    if (request.getString("content") == "complete this response") {
                        output.write(
                            "event: done\ndata: {\"assistant_message_id\":\"server-assistant-id\",\"replayed\":false}\n\n"
                                .toByteArray(),
                        )
                        output.flush()
                        return
                    }
                    output.flush()
                    repeat(100) {
                        Thread.sleep(50)
                        try {
                            output.write(": keepalive\n\n".toByteArray())
                            output.flush()
                        } catch (_: IOException) {
                            return
                        }
                    }
                }
                else -> writeJson(it, """{"detail":"not found"}""", status = "404 Not Found")
            }
        }
    }

    private fun writeJson(socket: Socket, body: String, status: String = "200 OK") {
        val bytes = body.toByteArray(StandardCharsets.UTF_8)
        val headers =
            "HTTP/1.1 $status\r\n" +
                "Content-Type: application/json\r\n" +
                "Content-Length: ${bytes.size}\r\n" +
                "Connection: close\r\n\r\n"
        socket.getOutputStream().apply {
            write(headers.toByteArray(StandardCharsets.US_ASCII))
            write(bytes)
            flush()
        }
    }

    private fun readLine(input: BufferedInputStream): String? {
        val result = ByteArrayOutputStream()
        while (true) {
            val value = input.read()
            if (value < 0) return if (result.size() == 0) null else result.toString("UTF-8")
            if (value == '\n'.code) {
                val bytes = result.toByteArray()
                val length = if (bytes.lastOrNull() == '\r'.code.toByte()) bytes.size - 1 else bytes.size
                return String(bytes, 0, length, StandardCharsets.US_ASCII)
            }
            result.write(value)
        }
    }

    override fun close() {
        running = false
        sockets.forEach(Socket::close)
        server.close()
        thread.join(1_000)
    }
}
