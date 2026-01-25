use smithy4rs_core::{
    prelude::LengthTrait,
    smithy,
};

smithy!("com.test#StringWithMinAndMax": {
    @LengthTrait::builder().min(2L).max(4L).build();
    string STRING_WITH_MIN_AND_MAX
});

smithy!("com.test#StringWithMax": {
    @LengthTrait::builder().max(2L).build();
    string STRING_WITH_MAX
});

smithy!("com.test#StringWithMin": {
    @LengthTrait::builder().min(1L).build();
    string STRING_WITH_MIN
});
