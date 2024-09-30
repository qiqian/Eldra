plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.serialization)
    alias(libs.plugins.undercouchDownload) apply false
}

kotlin {
    mingwX64("native") { // on macOS
        // linuxX64("native") // on Linux
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
        val wasmWasiMain by getting {
            dependencies {
                implementation(project(":runtime"))
            }
        }
        val wasmWasiTest by getting {
            dependencies {
                implementation(libs.kotlin.test)
                implementation(project(":runtime"))
            }
        }
    }
}
