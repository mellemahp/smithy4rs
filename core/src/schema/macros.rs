#[macro_export]
macro_rules! traits {
    () => { None };
    // Some(vec!(Arc::new(HttpCode::new(10))))
    ($($x:expr),+ $(,)?) => (
        Some(vec![$(Arc::new($x)),*])
    );
}

#[macro_export]
macro_rules! lazy_member_schema {
    ($member_schema_name:ident, $parent_schema:ident, $identifier:literal) => {
        static $member_schema_name: LazyLock<&Schema> =
            LazyLock::new(|| $parent_schema.expect_member($identifier));
    };
}

#[macro_export]
macro_rules! lazy_shape_id {
    ($id_name:ident, $identifier:literal) => {
        static $id_name: LazyLock<ShapeId> = LazyLock::new(|| ShapeId::from($identifier));
    };
}
