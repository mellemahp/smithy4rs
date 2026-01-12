use smithy4rs_core::{
    prelude::{
        INTEGER,
        STRING,
    },
    smithy,
};

smithy!("com.test#MyMap": {
    map MY_MAP {
        key: STRING
        value: INTEGER
    }
});
