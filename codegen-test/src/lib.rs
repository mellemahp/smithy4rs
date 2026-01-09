
mod shapes {
    use smithy4rs_core::smithy;
    use smithy4rs_core_derive::SmithyShape;
    use smithy4rs_core::prelude::*;

    // Adds generated file from the "example-rust-codegen" plugin in the "source" projection.
    // Note: the "source" projection is the default projection for Smithy.
    include!(concat!(env!("SMITHY_OUTPUT_DIR"),
    "/", "source", // <- Projection name
    "/", "rust-types", // <- Plugin name
    "/", "smithy-generated.rs") // <- Generated file to include
    );
}

#[cfg(test)]
mod tests {
    use smithy4rs_core::serde::{Buildable, ShapeBuilder};
    use crate::shapes::TestStruct;

    #[test]
    fn builds() {
        let x = TestStruct::builder()
            .a("s".to_string())
            .b(21)
            .build()
            .expect("Should Build");
        println!("{:?}", x)
    }
}