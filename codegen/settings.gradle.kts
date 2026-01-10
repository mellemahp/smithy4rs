pluginManagement {
    repositories {
        mavenLocal()
        mavenCentral()
        gradlePluginPortal()
    }
}

rootProject.name = "smithy4rs-codegen"

// Core library
include(":core")

// Test runners for codegen
include(":test-utils")

// Plugins
include(":plugins")
include(":plugins:type-codegen")