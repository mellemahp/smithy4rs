import org.jreleaser.model.Active

plugins {
    base
    idea
    alias(libs.plugins.jreleaser)
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


/*
 * Publishing to maven central with Jreleaser (https://jreleaser.org)
 */
jreleaser {
    dryrun = false
    gitRootSearch.set(true)

    // Used for creating a tagged release, uploading files and generating changelog.
    // In the future we can set this up to push release tags to GitHub, but for now it's
    // set up to do nothing.
    // https://jreleaser.org/guide/latest/reference/release/index.html
    release {
        generic {
            enabled = true
            skipRelease = true
        }
    }

    // Used to announce a release to configured announcers.
    // https://jreleaser.org/guide/latest/reference/announce/index.html
    announce {
        active = Active.NEVER
    }

    // Signing configuration.
    // https://jreleaser.org/guide/latest/reference/signing.html
    signing {
        active = Active.ALWAYS
        armored = true
    }

    // Configuration for deploying to Maven Central.
    // https://jreleaser.org/guide/latest/examples/maven/maven-central.html#_gradle
    deploy {
        maven {
            mavenCentral {
                create("maven-central") {
                    active = Active.ALWAYS
                    url = "https://central.sonatype.com/api/v1/publisher"
                    stagingRepository(rootProject.layout.buildDirectory.dir("staging").get().asFile.path)
                }
            }
        }
    }
}