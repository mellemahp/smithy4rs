use smithy4rs_core::IndexMap;
use smithy4rs_core::schema::Document;
use smithy4rs_core::string_map;
use smithy4rs_core_derive::SmithyTraitImpl;
#[smithy_trait_id("com.example#TestMapTrait")]
pub struct TestMapTrait(IndexMap<String, String>, Box<dyn Document>);
impl TestMapTrait {
    ///Create a new [`
    ///TestMapTrait
    ///`] instance
    #[automatically_derived]
    pub fn new(value: IndexMap<String, String>) -> Self {
        TestMapTrait(value.clone(), value.into())
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
        "com.example#TestMapTrait",
    ));
    #[automatically_derived]
    impl _StaticTraitId for TestMapTrait {
        #[inline]
        fn trait_id() -> &'static _ShapeId {
            &ID
        }
    }
    #[automatically_derived]
    impl _SmithyTrait for TestMapTrait {
        fn id(&self) -> &_ShapeId {
            Self::trait_id()
        }
        fn value(&self) -> &Box<dyn _Document> {
            &self.1
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for TestMapTrait {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_tuple_field2_finish(
            f,
            "TestMapTrait",
            &self.0,
            &&self.1,
        )
    }
}
fn test() {
    let _ = TestMapTrait::new(
        ::smithy4rs_core::IndexMap::<String, _>::from_iter([("a".into(), "b".into())]),
    );
}
