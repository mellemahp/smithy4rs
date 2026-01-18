plugins {
    base
    idea
}

repositories {
    mavenLocal()
    mavenCentral()
}

val smithy4rsCodegenVersion = project.file("VERSION").readText().replace(System.lineSeparator(), "")
allprojects {
    group = "dev.hmellema.smithy4rs"
    version = smithy4rsCodegenVersion
}
println("Smithy4rs codegen version: '${smithy4rsCodegenVersion}'")

// Apply idea plugin to all integration tests.
subprojects {
    plugins.withId("java") {
        apply(plugin = "idea")
        afterEvaluate {
            val sourceSets = the<SourceSetContainer>()
            sourceSets.findByName("it")?.let {
                idea {
                    module {
                        testSources.from(sourceSets["it"].java.srcDirs)
                    }
                }
            }
        }
    }
}