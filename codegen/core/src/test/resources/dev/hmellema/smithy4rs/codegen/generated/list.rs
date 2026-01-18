use smithy4rs_core::{
    prelude::STRING,
    smithy,
};

smithy!("com.test#MyList": {
    list MY_LIST {
        member: STRING
    }
});
