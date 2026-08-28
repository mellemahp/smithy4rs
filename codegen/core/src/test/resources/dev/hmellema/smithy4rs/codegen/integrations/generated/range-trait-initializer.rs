use smithy4rs_core::{
    prelude::RangeTrait,
    smithy,
};

smithy!(
    @RangeTrait::builder().min(1).build();
    integer com::test::NumWithMin
);

smithy!(
    @RangeTrait::builder().min(2).max(4).build();
    bigDecimal com::test::NumWithMinAndMax
);

smithy!(
    @RangeTrait::builder().max(2).build();
    float com::test::NumWithMax
);
