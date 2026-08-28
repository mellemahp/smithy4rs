use smithy4rs_core::{
    prelude::LengthTrait,
    smithy,
};

smithy!(
    @LengthTrait::builder().min(2i64).max(4i64).build();
    string com::test::StringWithMinAndMax
);

smithy!(
    @LengthTrait::builder().max(2i64).build();
    string com::test::StringWithMax
);

smithy!(
    @LengthTrait::builder().min(1i64).build();
    string com::test::StringWithMin
);
