use smithy4rs_core::schema::Document;
use smithy4rs_core_derive::SmithyTraitImpl;
#[smithy_trait_id("com.example#TestListTrait")]
pub struct TestListTrait(Vec<String>, Box<dyn Document>);
impl TestListTrait {
    ///Create a new [`
    ///TestListTrait
    ///`] instance
    #[automatically_derived]
    pub fn new(value: Vec<String>) -> Self {
        TestListTrait(value.clone(), value.into())
    }
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::StaticTraitId as _StaticTraitId;
    use _smithy4rs::schema::ShapeId as _ShapeId;
    use _smithy4rs::LazyLock as _LazyLock;
    use _smithy4rs::schema::SmithyTrait as _SmithyTrait;
    use _smithy4rs::schema::Document as _Document;
    static ID: _LazyLock<_ShapeId> = _LazyLock::new(|| _ShapeId::from(
        "com.example#TestListTrait",
    ));
    #[automatically_derived]
    impl _StaticTraitId for TestListTrait {
        #[inline]
        fn trait_id() -> &'static _ShapeId {
            &ID
        }
    }
    #[automatically_derived]
    impl _SmithyTrait for TestListTrait {
        fn id(&self) -> &_ShapeId {
            Self::trait_id()
        }
        fn value(&self) -> &Box<dyn _Document> {
            &self.1
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for TestListTrait {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_tuple_field2_finish(
            f,
            "TestListTrait",
            &self.0,
            &&self.1,
        )
    }
}
