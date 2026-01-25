use smithy4rs_core::{
    prelude::RangeTrait,
    smithy,
};

smithy!("com.test#NumWithMin": {
    @RangeTrait::builder().min("1").build();
    integer NUM_WITH_MIN
});

smithy!("com.test#NumWithMinAndMax": {
    @RangeTrait::builder().min("2").max("4").build();
    bigdecimal NUM_WITH_MIN_AND_MAX
});

smithy!("com.test#NumWithMax": {
    @RangeTrait::builder().max("2").build();
    float NUM_WITH_MAX
});
