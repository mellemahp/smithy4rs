#![allow(dead_code)]

mod shapes_expand;

#[test]
fn test_shape_macro_expansion() {
    // Listed out to avoid expanding `mod.rs`
    macrotest::expand("tests/shapes_expand/enum.rs");
    macrotest::expand("tests/shapes_expand/int_enum.rs");
    macrotest::expand("tests/shapes_expand/simple_struct.rs");
    macrotest::expand("tests/shapes_expand/union.rs");
}
