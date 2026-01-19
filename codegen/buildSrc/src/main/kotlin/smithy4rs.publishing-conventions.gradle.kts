plugins {
    `maven-publish`
}

interface PublishingConfigExtension {
    var customComponent: SoftwareComponent?
}

val extension = project.extensions.create<PublishingConfigExtension>("configurePublishing").apply {
    customComponent = null
}

/*
 * Staging repository
 * ====================================================
 *
 * Configure publication to staging repo for jreleaser
 */
publishing {
    repositories {
        maven {
            name = "stagingRepository"
            url = rootProject.layout.buildDirectory.dir("staging").get().asFile.toURI()
        }
    }

    // Add license spec to all maven publications
    publications {
        afterEvaluate {
            create<MavenPublication>("mavenJava") {
                from(extension.customComponent ?: components["java"])
                val displayName: String by extra
                pom {
                    name.set(displayName)
                    description.set(project.description)
                    url.set("https://github.com/mellemahp/smithy4rs")
                    licenses {
                        license {
                            name.set("Apache License 2.0")
                            url.set("http://www.apache.org/licenses/LICENSE-2.0.txt")
                            distribution.set("repo")
                        }
                    }
                    developers {
                        developer {
                            id.set("smithy4rs")
                            name.set("smithy4rs")
                            organization.set("Smithy4rs Community")
                            organizationUrl.set("https://github.com/mellemahp/smithy4rs")
                            roles.add("developer")
                        }
                    }
                    scm {
                        url.set("https://github.com/mellemahp/smithy4rs.git")
                    }
                }
            }
        }
    }
}