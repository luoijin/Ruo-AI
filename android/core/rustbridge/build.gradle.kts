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
}

androidComponents {
    onVariants { variant ->
        variant.sources.kotlin!!.addStaticSourceDirectory(
            layout.buildDirectory.dir("generated/uniffi").get().asFile.absolutePath,
        )
    }
}

dependencies {
    implementation("androidx.annotation:annotation:1.11.0")
    implementation("net.java.dev.jna:jna:5.19.1@aar")
}
