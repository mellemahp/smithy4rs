#![allow(dead_code)]

mod shapes_expand;
mod traits_expand;

#[test]
fn test_shape_macro_expansion() {
    // Listed out to avoid expanding `mod.rs`
    macrotest::expand("tests/shapes_expand/enum.rs");
    macrotest::expand("tests/shapes_expand/int_enum.rs");
    macrotest::expand("tests/shapes_expand/simple_struct.rs");
    macrotest::expand("tests/shapes_expand/union.rs");
}

#[test]
fn test_trait_macro_expansion() {
    // // Listed out to avoid expanding `mod.rs`
    macrotest::expand("tests/traits_expand/list_trait.rs");
    macrotest::expand("tests/traits_expand/map_trait.rs");
    // macrotest::expand("tests/traits_expand/struct_trait.rs");
}
