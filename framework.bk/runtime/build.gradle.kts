plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.serialization)
    alias(libs.plugins.undercouchDownload) apply false
}

kotlin {
    mingwX64("native-win") { // on macOS
        // linuxX64("native") // on Linux
        // mingwX64("native") // on Windows
        binaries {
            executable()
        }
    }

    linuxX64("native-linux") { // on Linux
        // mingwX64("native") // on Windows
        binaries {
            executable()
        }
    }

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

