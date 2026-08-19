use smithy4rs_core::{
    derive::{
        SmithyShape,
        SmithyTrait,
    },
    doc_map,
    prelude::{
        INTEGER,
        STRING,
    },
    schema::DynamicTrait,
    smithy,
};

smithy!("com.test#AppliedTo": {
    /// Schema for [`AppliedTo`]
    @DynamicTrait::from("com.test#floatTrait", 2.0);
    @DynamicTrait::from("com.test#myCustomStruct", doc_map!["a" => "str", "b" => 2]);
    @DynamicTrait::from("com.test#intTrait", 1);
    @DynamicTrait::from("com.test#otherListTrait", vec![1, 2, 3]);
    @DynamicTrait::from("com.test#stringTrait", "stuff");
    @DynamicTrait::from("com.test#stringListTrait", vec!["a", "b", "c"]);
    structure APPLIED_TO_SCHEMA {
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = APPLIED_TO_SCHEMA)]
pub struct AppliedTo {
}

smithy!("com.test#myCustomStruct": {
    /// Schema for [`MyCustomStructTrait`]
    structure MY_CUSTOM_STRUCT_SCHEMA {
        A: STRING = "a"
        B: INTEGER = "b"
    }
});

#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[schema(schema = MY_CUSTOM_STRUCT_SCHEMA)]
pub struct MyCustomStructTrait {
    #[schema(schema = A)]
    pub a: Option<String>,
    #[schema(schema = B)]
    pub b: Option<i32>,
}

smithy!("com.test#otherListTrait": {
    list OTHER_LIST_TRAIT {
        member: INTEGER
    }
});
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = OTHER_LIST_TRAIT)]
#[repr(transparent)]
pub struct OtherListTraitTrait(Vec<i32>);

smithy!("com.test#stringListTrait": {
    list STRING_LIST_TRAIT {
        member: STRING
    }
});
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = STRING_LIST_TRAIT)]
#[repr(transparent)]
pub struct StringListTraitTrait(Vec<String>);

smithy!("com.test#floatTrait": {
    float FLOAT_TRAIT
});

#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = FLOAT_TRAIT)]
#[repr(transparent)]
pub struct FloatTraitTrait(f32);

smithy!("com.test#stringTrait": {
    string STRING_TRAIT
});

#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = STRING_TRAIT)]
#[repr(transparent)]
pub struct StringTraitTrait(String);

smithy!("com.test#intTrait": {
    integer INT_TRAIT
});

#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = INT_TRAIT)]
#[repr(transparent)]
pub struct IntTraitTrait(i32);
