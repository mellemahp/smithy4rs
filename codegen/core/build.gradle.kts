plugins {
    id("smithy4rs.java-conventions")
    // TODO(publishing): Add publishing conventions
}

description = "Provides common functionality for rust code generation plugins"

dependencies {
    api(libs.smithy.codegen)

    // ==== Test dependencies ====
    testImplementation(project(":test-utils"))
}
