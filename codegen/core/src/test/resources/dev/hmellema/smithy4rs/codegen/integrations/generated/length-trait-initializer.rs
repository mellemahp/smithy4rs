use smithy4rs_core::{
    prelude::LengthTrait,
    smithy,
};

smithy!("com.test#StringWithMinAndMax": {
    @LengthTrait::builder().min(2i64).max(4i64).build();
    string STRING_WITH_MIN_AND_MAX
});

smithy!("com.test#StringWithMax": {
    @LengthTrait::builder().max(2i64).build();
    string STRING_WITH_MAX
});

smithy!("com.test#StringWithMin": {
    @LengthTrait::builder().min(1i64).build();
    string STRING_WITH_MIN
});
