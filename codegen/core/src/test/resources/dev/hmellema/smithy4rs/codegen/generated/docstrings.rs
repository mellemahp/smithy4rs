use smithy4rs_core::{
    derive::{
        SmithyShape,
        smithy_enum,
        smithy_union,
    },
    prelude::{
        INTEGER,
        STRING,
    },
    smithy,
};

smithy!(
    /// Schema for [`DocumentedEnum`]
    enum com::test::DocumentedEnum {
        One = "one"
        Two = "two"
    }
);

/// A Documented Enum
#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = DOCUMENTED_ENUM)]
pub enum DocumentedEnum {
    One = "one",
    Two = "two",
}

smithy!(
    /// Schema for [`DocumentedIntEnum`]
    intEnum com::test::DocumentedIntEnum {
        One = 1
        Two = 2
    }
);

/// A Documented IntEnum
#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = DOCUMENTED_INT_ENUM)]
pub enum DocumentedIntEnum {
    One = 1,
    Two = 2,
}

smithy!(
    /// Documented List
    list com::test::DocumentedList {
        member: STRING
    }
);

smithy!(
    /// Documented Map
    map com::test::DocumentedMap {
        key: STRING
        value: STRING
    }
);

smithy!(
    /// Schema for [`DocumentedStruct`]
    structure com::test::DocumentedStruct {
        documentedMember: STRING
    }
);

/// A Documented Structure
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = DOCUMENTED_STRUCT)]
pub struct DocumentedStruct {
    /// Documented! Yay!
    pub documented_member: Option<String>,
}

smithy!(
    /// Schema for [`DocumentedUnion`]
    union com::test::DocumentedUnion {
        variantA: STRING
        variantB: INTEGER
    }
);

/// A Documented Union
#[smithy_union]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = DOCUMENTED_UNION)]
pub enum DocumentedUnion {
    /// A String variant
    VariantA(String),
    /// An integer variant
    VariantB(i32),
}

smithy!(
    /// Documented Scalar
    string com::test::DocumentedScalar
);
