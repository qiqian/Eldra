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
