plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "io.luoijin.ruo.core.rustbridge"
    compileSdk = 36
    ndkVersion = "30.0.16248370"

    defaultConfig {
        minSdk = 26
        ndk {
            abiFilters += "arm64-v8a"
        }
    }

    sourceSets["main"].java.srcDir(layout.buildDirectory.dir("generated/uniffi"))
}

dependencies {
    implementation("net.java.dev.jna:jna:5.19.1@aar")
}
