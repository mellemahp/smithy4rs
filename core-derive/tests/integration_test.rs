#[test]
fn test_macro_expansion() {
    macrotest::expand("tests/expand/*.rs");
}
