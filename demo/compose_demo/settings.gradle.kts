rootProject.name = "compose_demo"
enableFeaturePreview("TYPESAFE_PROJECT_ACCESSORS")

pluginManagement {
    repositories {
        maven("https://maven.qq-home.com:8443/repository/google/") {
            mavenContent {
                includeGroupAndSubgroups("androidx")
                includeGroupAndSubgroups("com.android")
                includeGroupAndSubgroups("com.google")
            }
        }
        maven("https://maven.qq-home.com:8443/repository/maven-central/")
        maven("https://maven.qq-home.com:8443/repository/gradle-plugin-portal/")
    }
}

dependencyResolutionManagement {
    repositories {
        mavenLocal()
        maven("https://maven.qq-home.com:8443/repository/jetbrains-compose-dev/")
        maven("https://maven.qq-home.com:8443/repository/google/") {
            mavenContent {
                includeGroupAndSubgroups("androidx")
                includeGroupAndSubgroups("com.android")
                includeGroupAndSubgroups("com.google")
            }
        }
        maven("https://maven.qq-home.com:8443/repository/maven-central/")
    }
}

include(":composeApp")