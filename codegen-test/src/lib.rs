#![allow(dead_code)]

mod shapes {
    use smithy4rs_core::{generated_shapes, prelude::*, smithy};
    use smithy4rs_core_derive::SmithyShape;

    generated_shapes![];
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
