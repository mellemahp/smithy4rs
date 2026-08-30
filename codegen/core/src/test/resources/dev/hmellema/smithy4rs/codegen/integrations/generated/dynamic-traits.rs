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

smithy!(
    /// Schema for [`AppliedTo`]
    @DynamicTrait::from("com.test#floatTrait", 2.0);
    @DynamicTrait::from("com.test#myCustomStruct", doc_map!["a" => "str", "b" => 2]);
    @DynamicTrait::from("com.test#intTrait", 1);
    @DynamicTrait::from("com.test#otherListTrait", vec![1, 2, 3]);
    @DynamicTrait::from("com.test#stringTrait", "stuff");
    @DynamicTrait::from("com.test#stringListTrait", vec!["a", "b", "c"]);
    structure com::test::AppliedTo {
    }
);

#[derive(SmithyShape, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = APPLIED_TO)]
pub struct AppliedTo {
}

smithy!(
    /// Schema for [`MyCustomStructTrait`]
    structure com::test::myCustomStruct {
        a: STRING
        b: INTEGER
    }
);

#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = MY_CUSTOM_STRUCT)]
pub struct MyCustomStructTrait {
    pub a: Option<String>,
    pub b: Option<i32>,
}

smithy!(
    list com::test::otherListTrait {
        member: INTEGER
    }
);
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = OTHER_LIST_TRAIT)]
#[repr(transparent)]
pub struct OtherListTraitTrait(Vec<i32>);

smithy!(
    list com::test::stringListTrait {
        member: STRING
    }
);
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = STRING_LIST_TRAIT)]
#[repr(transparent)]
pub struct StringListTraitTrait(Vec<String>);

smithy!(
    float com::test::floatTrait
);

#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = FLOAT_TRAIT)]
#[repr(transparent)]
pub struct FloatTraitTrait(f32);

smithy!(
    string com::test::stringTrait
);

#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = STRING_TRAIT)]
#[repr(transparent)]
pub struct StringTraitTrait(String);

smithy!(
    integer com::test::intTrait
);

#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = INT_TRAIT)]
#[repr(transparent)]
pub struct IntTraitTrait(i32);
