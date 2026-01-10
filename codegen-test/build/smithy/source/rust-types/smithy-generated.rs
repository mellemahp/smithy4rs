smithy!("com.test#TestStruct": {
    structure TEST_STRUCT_SCHEMA {
        A: STRING = "a"
        B: INTEGER = "b"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(TEST_STRUCT_SCHEMA)]
pub struct TestStruct {
    #[smithy_schema(A)]
    pub a: String,
    #[smithy_schema(B)]
    pub b: i32,
}
