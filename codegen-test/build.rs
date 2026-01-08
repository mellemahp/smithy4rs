extern crate smithy_cargo;

use std::fs::canonicalize;
use smithy_cargo::SmithyBuild;
use std::process::Command;

fn main() {
    // Publish the test code generator to maven local so the
    // Smithy CLI can detect it
    Command::new("./gradlew")
        .current_dir("codegen-link")
        .arg("publishToMavenLocal")
        .output()
        .unwrap();

    println!("cargo::rerun-if-changed=codegen-link");

    SmithyBuild::new()
        .execute()
        .expect("Smithy Build failed");
}