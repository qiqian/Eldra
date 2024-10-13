rootProject.name = "framework"

pluginManagement {
    resolutionStrategy {
        repositories {
            maven("https://maven.qq-home.com:8443/repository/google/")
            maven("https://maven.qq-home.com:8443/repository/maven-central/")
            maven("https://maven.qq-home.com:8443/repository/gradle-plugin-portal/")
        }
    }
}
include("runtime", "editor")
