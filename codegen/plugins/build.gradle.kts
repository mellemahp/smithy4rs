plugins {
    id("smithy4rs.module-conventions")
}

description = "This module provides Rust code generation plugins for Smithy"

extra["displayName"] = "Smithy4rs :: Plugins :: Type Codegen"
extra["moduleName"] = "dev.hmellema.smithy4rs.codegen.plugins"

// Aggregate dependencies to create a single plugin JAR
dependencies {
    subprojects.forEach { api(project(it.path)) }
}
