
#[macro_export]
macro_rules! lazy_member_schema {
    ($member_schema_name:ident, $parent_schema:ident, $identifier:literal) => {
        static $member_schema_name: LazyLock<&Schema> = LazyLock::new(|| $parent_schema.expect_member($identifier));
    };
}

