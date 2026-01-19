plugins {
    id("smithy4rs.module-conventions")
}

description = "This module provides a Smithy-build plugin for type-codegen"

extra["displayName"] = "Smithy4rs :: Plugins :: Type Codegen"
extra["moduleName"] = "dev.hmellema.smithy4rs.codegen.types"

dependencies {
    implementation(project(":core"))
}
