package io.luoijin.ruo.core.rustbridge

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.io.File
import java.security.GeneralSecurityException
import java.security.KeyStore
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

class KeyUnavailableException : Exception("Encrypted local data cannot be opened with the available key")

class DatabaseKeyManager(context: Context) {
    private val appContext = context.applicationContext
    private val preferences =
        appContext.getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE)
    private val keyStore =
        KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }

    fun getOrCreate(databaseFile: File): ByteArray = synchronized(KEY_LOCK) {
        val wrappedKey = preferences.getString(WRAPPED_KEY, null)
        val iv = preferences.getString(IV, null)
        if (wrappedKey != null || iv != null) {
            if (wrappedKey == null || iv == null) throw KeyUnavailableException()
            val wrappingKey = existingWrappingKey() ?: throw KeyUnavailableException()
            val unwrapped = try {
                Cipher.getInstance(TRANSFORMATION).run {
                    init(
                        Cipher.DECRYPT_MODE,
                        wrappingKey,
                        GCMParameterSpec(TAG_LENGTH_BITS, Base64.decode(iv, Base64.NO_WRAP)),
                    )
                    doFinal(Base64.decode(wrappedKey, Base64.NO_WRAP))
                }
            } catch (_: GeneralSecurityException) {
                throw KeyUnavailableException()
            } catch (_: IllegalArgumentException) {
                throw KeyUnavailableException()
            }
            if (unwrapped.size != DATABASE_KEY_SIZE) {
                unwrapped.fill(0)
                throw KeyUnavailableException()
            }
            return@synchronized unwrapped
        }

        if (databaseFile.exists() || sidecarFiles(databaseFile).any(File::exists)) {
            throw KeyUnavailableException()
        }

        val databaseKey = ByteArray(DATABASE_KEY_SIZE).also(SecureRandom()::nextBytes)
        try {
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.ENCRYPT_MODE, getOrCreateWrappingKey())
            val wrapped = cipher.doFinal(databaseKey)
            val saved = preferences.edit()
                .putString(WRAPPED_KEY, Base64.encodeToString(wrapped, Base64.NO_WRAP))
                .putString(IV, Base64.encodeToString(cipher.iv, Base64.NO_WRAP))
                .commit()
            if (!saved) throw IllegalStateException("Unable to persist the encrypted database key")
        } catch (error: GeneralSecurityException) {
            databaseKey.fill(0)
            throw IllegalStateException("Unable to protect the encrypted database key", error)
        } catch (error: RuntimeException) {
            databaseKey.fill(0)
            throw error
        }
        databaseKey
    }

    fun wipeLocalData(databaseFile: File) = synchronized(KEY_LOCK) {
        if (!preferences.edit().clear().commit()) {
            throw IllegalStateException("Unable to remove encrypted database key data")
        }
        if (keyStore.containsAlias(KEY_ALIAS)) {
            keyStore.deleteEntry(KEY_ALIAS)
        }
        for (file in listOf(databaseFile) + sidecarFiles(databaseFile)) {
            if (file.exists() && !file.delete()) {
                throw IllegalStateException("Unable to remove encrypted local database")
            }
        }
    }

    private fun existingWrappingKey(): SecretKey? =
        keyStore.getKey(KEY_ALIAS, null) as? SecretKey

    private fun getOrCreateWrappingKey(): SecretKey {
        existingWrappingKey()?.let { return it }
        val generator = KeyGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_AES,
            ANDROID_KEYSTORE,
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

    private fun sidecarFiles(databaseFile: File): List<File> = listOf(
        File(databaseFile.parentFile, "${databaseFile.name}-wal"),
        File(databaseFile.parentFile, "${databaseFile.name}-shm"),
    )

    private companion object {
        const val ANDROID_KEYSTORE = "AndroidKeyStore"
        const val DATABASE_KEY_SIZE = 32
        const val IV = "iv"
        const val KEY_ALIAS = "ruo-database-wrapping-key"
        const val PREFERENCES_NAME = "ruo_database_key"
        const val TAG_LENGTH_BITS = 128
        const val TRANSFORMATION = "AES/GCM/NoPadding"
        const val WRAPPED_KEY = "wrapped_key"
        val KEY_LOCK = Any()
    }
}
