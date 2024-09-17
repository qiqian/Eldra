plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.serialization)
    alias(libs.plugins.undercouchDownload) apply false
}

repositories {
    mavenCentral()
}

kotlin {
    wasmWasi {
        nodejs()
        binaries.executable()
    }

    sourceSets {
        val wasmWasiTest by getting {
            dependencies {
                implementation(libs.kotlin.test)
            }
        }
    }
}

