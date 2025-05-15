#[macro_export]
macro_rules! traits {
    () => { Vec::new() };
    ($($x:expr),+ $(,)?) => (
        vec![$(Ref::new($x)),*]
    );
}

#[macro_export]
macro_rules! lazy_schema {
    ($schema_name:ident, $builder:expr) => {
        pub static $schema_name: LazyLock<Schema> = LazyLock::new(|| {
            $builder
        });
    };
}

#[macro_export]
macro_rules! lazy_member_schema {
    ($member_schema_name:ident, $parent_schema:ident, $identifier:literal) => {
        static $member_schema_name: LazyLock<Ref<Schema>> =
            LazyLock::new(|| $parent_schema.expect_member($identifier));
    };
}

#[macro_export]
macro_rules! lazy_shape_id {
    ($id_name:ident, $identifier:literal) => {
        static $id_name: LazyLock<ShapeId> = LazyLock::new(|| ShapeId::from($identifier));
    };
}
