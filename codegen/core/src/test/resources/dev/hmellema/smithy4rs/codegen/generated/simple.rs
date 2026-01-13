use smithy4rs_core::{
    schema::DynamicTrait,
    smithy,
};

smithy!("com.test#MyShort": {
    @DynamicTrait::from("smithy.api#default", 0);
    short MY_SHORT
});

smithy!("com.test#MyBlob": {
    blob MY_BLOB
});

smithy!("com.test#MyFloat": {
    @DynamicTrait::from("smithy.api#default", 0);
    float MY_FLOAT
});

smithy!("com.test#MyByte": {
    @DynamicTrait::from("smithy.api#default", 0);
    byte MY_BYTE
});

smithy!("com.test#MyBigDecimal": {
    bigdecimal MY_BIG_DECIMAL
});

smithy!("com.test#MyBigInteger": {
    biginteger MY_BIG_INTEGER
});

smithy!("com.test#MyBoolean": {
    @DynamicTrait::from("smithy.api#default", false);
    boolean MY_BOOLEAN
});

smithy!("com.test#MyInteger": {
    @DynamicTrait::from("smithy.api#default", 0);
    integer MY_INTEGER
});

smithy!("com.test#MyDouble": {
    @DynamicTrait::from("smithy.api#default", 0);
    double MY_DOUBLE
});

smithy!("com.test#MyLong": {
    @DynamicTrait::from("smithy.api#default", 0);
    long MY_LONG
});

smithy!("com.test#MyTimestamp": {
    timestamp MY_TIMESTAMP
});

smithy!("com.test#MyString": {
    string MY_STRING
});
