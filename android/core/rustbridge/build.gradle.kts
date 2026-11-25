plugins {
    id("com.android.library")
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

    sourceSets.named("main") {
        kotlin.directories.add(
            layout.buildDirectory.dir("generated/uniffi").get().asFile.absolutePath,
        )
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.19.1@aar")
}
