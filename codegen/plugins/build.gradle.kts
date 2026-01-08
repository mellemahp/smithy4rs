plugins {
    id("smithy4rs.java-conventions")
    // TODO(publishing): Add publishing conventions
}

description = "This module provides Rust code generation plugins for Smithy"

// Aggregate dependencies to create a single plugin JAR
dependencies {
    subprojects.forEach { api(project(it.path)) }
}
