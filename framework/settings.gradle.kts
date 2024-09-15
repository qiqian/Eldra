rootProject.name = "framework"

pluginManagement {
    resolutionStrategy {
        repositories {
            gradlePluginPortal()
        }
    }
}
include("runtime")
include("editor")
