use std::process::Command;

use smithy_cargo::SmithyBuild;

fn main() {
    // Publish the test code generator to maven local so the
    // Smithy CLI can detect it
    let output = Command::new("./gradlew")
        // Use local simlink as cargo only allows build script
        // access inside project dir
        .current_dir("codegen-link")
        .arg("build")
        .arg("publishToMavenLocal")
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "Gradlew build failed: {}",
            String::from_utf8(output.stderr).unwrap()
        );
    }

    println!("cargo::rerun-if-changed=codegen-link");

    SmithyBuild::new()
        .env("IN_SMITH4RS_CORE", "true")
        .execute()
        .expect("Smithy Build failed");
}
