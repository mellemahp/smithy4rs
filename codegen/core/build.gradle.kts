plugins {
    id("smithy4rs.module-conventions")
}

description = "Provides common functionality for rust code generation plugins"

extra["displayName"] = "Smithy4rs :: Codegen :: Core"
extra["moduleName"] = "dev.hmellema.smithy4rs.codegen"

dependencies {
    api(libs.smithy.codegen)

    // ==== Test dependencies ====
    testImplementation(project(":test-utils"))
}
