mod shapes;

#[test]
fn test_shape_macro_expansion() {
    macrotest::expand("tests/shapes/expand/*.rs");
}

#[test]
fn test_trait_macro_expansion() {
    macrotest::expand("tests/traits/expand/*.rs");
}
