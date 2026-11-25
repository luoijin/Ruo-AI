import java.io.File
import java.util.Properties

plugins {
    id("com.android.application")
    id("com.google.dagger.hilt.android")
    id("com.google.devtools.ksp")
    id("org.jetbrains.kotlin.plugin.compose")
    id("org.jetbrains.kotlin.plugin.serialization")
}

val signingProperties = Properties().apply {
    val propertiesFile = rootProject.file("keystore.properties")
    if (propertiesFile.isFile) {
        propertiesFile.inputStream().use { load(it) }
    }
}

fun signingValue(environmentName: String, propertyName: String): String? =
    System.getenv(environmentName)?.takeIf { it.isNotBlank() }
        ?: signingProperties.getProperty(propertyName)?.takeIf { it.isNotBlank() }

val debugKeystorePath = signingValue("RUO_DEBUG_KEYSTORE_PATH", "storeFile")
val debugKeystore = debugKeystorePath?.let { path ->
    val candidate = File(path)
    if (candidate.isAbsolute) candidate else rootProject.file(path)
}
val debugStorePassword = signingValue("RUO_DEBUG_STORE_PASSWORD", "storePassword")
val debugKeyAlias = signingValue("RUO_DEBUG_KEY_ALIAS", "keyAlias")
val debugKeyPassword = signingValue("RUO_DEBUG_KEY_PASSWORD", "keyPassword")
val stableDebugSigningAvailable =
    debugKeystore?.isFile == true &&
        debugStorePassword != null &&
        debugKeyAlias != null &&
        debugKeyPassword != null

android {
    namespace = "io.luoijin.ruo"
    compileSdk = 36
    ndkVersion = "30.0.16248370"

    buildFeatures {
        compose = true
    }

    defaultConfig {
        applicationId = "io.luoijin.ruo"
        minSdk = 26
        targetSdk = 36
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        ndk {
            abiFilters += "arm64-v8a"
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    signingConfigs {
        if (stableDebugSigningAvailable) {
            create("stableDebug") {
                storeFile = checkNotNull(debugKeystore)
                storePassword = checkNotNull(debugStorePassword)
                keyAlias = checkNotNull(debugKeyAlias)
                keyPassword = checkNotNull(debugKeyPassword)
            }
        }
    }

    buildTypes {
        debug {
            applicationIdSuffix = ".debug"
            if (stableDebugSigningAvailable) {
                signingConfig = signingConfigs.getByName("stableDebug")
            } else {
                logger.warn(
                    "Stable debug signing is not configured; using the default debug keystore. " +
                        "Set RUO_DEBUG_KEYSTORE_PATH, RUO_DEBUG_STORE_PASSWORD, " +
                        "RUO_DEBUG_KEY_ALIAS, RUO_DEBUG_KEY_PASSWORD, or configure " +
                        "android/keystore.properties.",
                )
            }
        }
    }
}

dependencies {
    val composeBom = platform("androidx.compose:compose-bom:2026.06.01")
    implementation(project(":core:rustbridge"))
    implementation(composeBom)
    implementation("androidx.activity:activity-compose:1.13.0")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.navigation:navigation-compose:2.9.8")
    implementation("com.google.dagger:hilt-android:2.60.1")
    ksp("com.google.dagger:hilt-compiler:2.60.1")
    debugImplementation("androidx.compose.ui:ui-tooling")

    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test:core:1.7.0")
    androidTestImplementation("androidx.test:runner:1.7.0")
    androidTestImplementation("androidx.test.ext:junit:1.3.0")
    androidTestImplementation("junit:junit:4.13.2")
}
