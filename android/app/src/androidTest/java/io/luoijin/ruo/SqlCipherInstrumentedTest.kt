package io.luoijin.ruo

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.luoijin.ruo.core.rustbridge.DraftReadListener
import io.luoijin.ruo.core.rustbridge.RuoCore
import java.io.File
import java.security.KeyStore
import java.security.SecureRandom
import java.util.UUID
import java.util.concurrent.atomic.AtomicReference
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class SqlCipherInstrumentedTest {
    @Test
    fun keystoreKeyRoundTripsEncryptedDraftAndCallsKotlinListener() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        val dbFile = File(context.cacheDir, "ruo-${UUID.randomUUID()}.db")
        val conversationId = UUID.randomUUID().toString()
        val expected = "Taglish: Kumusta, paano ito? Cebuano: Maayong buntag, unsaon ni? 🌱📚"
        val key = DatabaseKeyStore(context).getOrCreate()

        try {
            val first = RuoCore.open(dbFile.absolutePath, key.copyOf())
            first.createConversation(conversationId, "Study", "en", false)
            first.saveDraft(conversationId, expected)

            val reopened = RuoCore.open(dbFile.absolutePath, key.copyOf())
            assertEquals(expected, reopened.getDraft(conversationId))

            val callbackResult = AtomicReference<Pair<String, String>?>()
            reopened.readDraftWithListener(
                conversationId,
                object : DraftReadListener {
                    override fun onDraftRead(conversationId: String, text: String) {
                        callbackResult.set(conversationId to text)
                    }
                },
            )
            assertEquals(conversationId to expected, callbackResult.get())

            val bytes = dbFile.readBytes()
            assertTrue(bytes.size >= 16)
            assertFalse(
                bytes.copyOfRange(0, 16).contentEquals(
                    "SQLite format 3\u0000".toByteArray(Charsets.US_ASCII),
                ),
            )

            val wrongKey = key.copyOf().also {
                it[0] = (it[0].toInt() xor 0x7f).toByte()
            }
            try {
                assertThrows(Exception::class.java) {
                    RuoCore.open(dbFile.absolutePath, wrongKey)
                }
                assertThrows(Exception::class.java) {
                    RuoCore.open(dbFile.absolutePath, byteArrayOf())
                }
            } finally {
                wrongKey.fill(0)
            }
        } finally {
            key.fill(0)
            dbFile.delete()
        }
    }
}

private class DatabaseKeyStore(context: Context) {
    private val preferences =
        context.getSharedPreferences("ruo_test_db_key", Context.MODE_PRIVATE)
    private val keyStore =
        KeyStore.getInstance("AndroidKeyStore").apply { load(null) }

    fun getOrCreate(): ByteArray {
        val wrappingKey = getWrappingKey()
        val wrapped = preferences.getString(WRAPPED_KEY, null)
        val iv = preferences.getString(IV, null)
        if (wrapped != null || iv != null) {
            check(wrapped != null && iv != null) { "Incomplete wrapped database key" }
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            cipher.init(
                Cipher.DECRYPT_MODE,
                wrappingKey,
                GCMParameterSpec(128, Base64.decode(iv, Base64.NO_WRAP)),
            )
            return cipher.doFinal(Base64.decode(wrapped, Base64.NO_WRAP))
        }

        val databaseKey = ByteArray(32).also(SecureRandom()::nextBytes)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, wrappingKey)
        val encryptedKey = cipher.doFinal(databaseKey)
        check(
            preferences.edit()
                .putString(WRAPPED_KEY, Base64.encodeToString(encryptedKey, Base64.NO_WRAP))
                .putString(IV, Base64.encodeToString(cipher.iv, Base64.NO_WRAP))
                .commit(),
        ) { "Failed to persist wrapped database key" }
        return databaseKey
    }

    private fun getWrappingKey(): SecretKey {
        val existing = keyStore.getKey(KEY_ALIAS, null) as? SecretKey
        if (existing != null) return existing

        val generator = KeyGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_AES,
            "AndroidKeyStore",
        )
        generator.init(
            KeyGenParameterSpec.Builder(
                KEY_ALIAS,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
            )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setRandomizedEncryptionRequired(true)
                .build(),
        )
        return generator.generateKey()
    }

    private companion object {
        const val KEY_ALIAS = "ruo-test-db-wrapping-key"
        const val WRAPPED_KEY = "wrapped_key"
        const val IV = "iv"
    }
}
