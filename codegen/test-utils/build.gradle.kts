plugins {
    id("smithy4rs.java-conventions")
    // TODO(publishing): Add publishing conventions
}

description = "Provides test-runners for checking generated code"

dependencies {
    api(platform(libs.junit.bom))
    api(libs.smithy.build)
    api(libs.smithy.utils)
    api(libs.junit.jupiter.api)
    api(libs.junit.jupiter.engine)
    api(libs.junit.jupiter.params)
}
