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

smithy!("com.test#DocumentedEnum": {
    /// Schema for [`DocumentedEnum`]
    enum DOCUMENTED_ENUM_SCHEMA {
        One = "one"
        Two = "two"
    }
});

/// A Documented Enum
#[smithy_enum]
#[derive(SmithyShape)]
#[smithy_schema(DOCUMENTED_ENUM_SCHEMA)]
pub enum DocumentedEnum {
    One = "one",
    Two = "two",
}

smithy!("com.test#DocumentedIntEnum": {
    /// Schema for [`DocumentedIntEnum`]
    enum DOCUMENTED_INT_ENUM_SCHEMA {
        One = 1
        Two = 2
    }
});

/// A Documented IntEnum
#[smithy_enum]
#[derive(SmithyShape)]
#[smithy_schema(DOCUMENTED_INT_ENUM_SCHEMA)]
pub enum DocumentedIntEnum {
    One = 1,
    Two = 2,
}

smithy!("com.test#DocumentedList": {
    /// Documented List
    list DOCUMENTED_LIST {
        member: STRING
    }
});

smithy!("com.test#DocumentedMap": {
    /// Documented Map
    map DOCUMENTED_MAP {
        key: STRING
        value: STRING
    }
});

smithy!("com.test#DocumentedStruct": {
    /// Schema for [`DocumentedStruct`]
    structure DOCUMENTED_STRUCT_SCHEMA {
        DOCUMENTED_MEMBER: STRING = "documentedMember"
    }
});

/// A Documented Structure
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(DOCUMENTED_STRUCT_SCHEMA)]
pub struct DocumentedStruct {
    /// Documented! Yay!
    #[smithy_schema(DOCUMENTED_MEMBER)]
    pub documentedMember: String,
}

smithy!("com.test#DocumentedUnion": {
    /// Schema for [`DocumentedUnion`]
    union DOCUMENTED_UNION_SCHEMA {
        VARIANT_A: STRING = "variantA"
        VARIANT_B: INTEGER = "variantB"
    }
});

/// A Documented Union
#[smithy_union]
#[derive(SmithyShape)]
#[smithy_schema(DOCUMENTED_UNION_SCHEMA)]
pub enum DocumentedUnion {
    /// A String variant
    #[smithy_schema(VARIANT_A)]
    Varianta(String),
    /// An integer variant
    #[smithy_schema(VARIANT_B)]
    Variantb(i32),
}

smithy!("com.test#DocumentedScalar": {
    /// Documented Scalar
    string DOCUMENTED_SCALAR
});
