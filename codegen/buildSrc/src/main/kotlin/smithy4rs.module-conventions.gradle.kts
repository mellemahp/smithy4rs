plugins {
    id("smithy4rs.java-conventions")
    id("smithy4rs.publishing-conventions")
}

val codegenVersion = project.file("${project.rootDir}/VERSION").readText().replace(System.lineSeparator(), "")

group = "dev.hmellema.smithy4rs"
version = codegenVersion

/*
 * Licensing
 * ============================
 */
// Reusable license copySpec
val licenseSpec = copySpec {
    from("../${project.rootDir}/LICENSE")
    from("../${project.rootDir}/NOTICE")
}

/*
 * Extra Jars
 * ============================
 */
java {
    withJavadocJar()
    withSourcesJar()
}

// Suppress warnings in javadocs
tasks.withType<Javadoc>() {
    (options as StandardJavadocDocletOptions).addStringOption("Xdoclint:-html", "-quiet")
}

// Include an Automatic-Module-Name in all JARs.
afterEvaluate {
    val moduleName: String by extra
    tasks.withType<Jar> {
        metaInf.with(licenseSpec)
        inputs.property("moduleName", moduleName)
        manifest {
            attributes(mapOf("Automatic-Module-Name" to moduleName))
        }
    }
}

// Always run javadoc after build.
tasks["build"].dependsOn(tasks["javadoc"])