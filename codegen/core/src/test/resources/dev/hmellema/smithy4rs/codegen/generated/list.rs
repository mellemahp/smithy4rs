use smithy4rs_core::{
    prelude::STRING,
    smithy,
};

smithy!(
    list com::test::MyList {
        member: STRING
    }
);
