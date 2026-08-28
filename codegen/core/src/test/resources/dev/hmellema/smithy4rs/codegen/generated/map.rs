use smithy4rs_core::{
    prelude::{
        INTEGER,
        STRING,
    },
    smithy,
};

smithy!(
    map com::test::MyMap {
        key: STRING
        value: INTEGER
    }
);
