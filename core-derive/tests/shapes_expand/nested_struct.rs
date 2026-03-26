use smithy4rs_core::{
    prelude::{INTEGER, STRING},
    smithy,
};
use smithy4rs_core_derive::SmithyShape;

smithy!("test#Address": {
    structure ADDRESS_SCHEMA {
        STREET: STRING = "street"
        ZIP: STRING = "zip"
    }
});

smithy!("test#Person": {
    structure PERSON_SCHEMA {
        NAME: STRING = "name"
        AGE: INTEGER = "age"
        HOME: ADDRESS_SCHEMA = "home"
        WORK: ADDRESS_SCHEMA = "work"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(ADDRESS_SCHEMA)]
pub struct Address {
    #[smithy_schema(STREET)]
    pub street: String,
    #[smithy_schema(ZIP)]
    pub zip: String,
}

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(PERSON_SCHEMA)]
pub struct Person {
    #[smithy_schema(NAME)]
    pub name: String,
    #[smithy_schema(AGE)]
    pub age: i32,
    #[smithy_schema(HOME)]
    pub home: Address,
    #[smithy_schema(WORK)]
    pub work: Option<Address>,
}
