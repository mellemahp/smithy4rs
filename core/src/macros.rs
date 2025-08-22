//! Macros shared
//! Primarily these macros are used to construct schemas and traits.

// Create a list of traits for use in Schema builders
#[macro_export]
macro_rules! traits {
    () => { Vec::new() };
    ($($x:expr),+ $(,)?) => (
        vec![$(Ref::new($x)),*]
    );
}

// Create a lazy, static Schema definition
#[macro_export]
macro_rules! lazy_schema {
    (
        $schema_name:ident,
        $builder:expr,
        $((
            $member_schema_name:ident,
            $member_ident:literal,
            $member_schema:ident,
            $member_traits:expr
        )), +
    ) => {
        pub static $schema_name: LazyLock<SchemaRef> = LazyLock::new(|| {
            $builder
            $(.put_member($member_ident, &$member_schema, $member_traits))
            *
            .build()
        });
        $(static $member_schema_name: LazyLock<&SchemaRef> =
            LazyLock::new(|| $schema_name.expect_member($member_ident));
        )*
    };
    (
        $schema_name:ident,
        $builder:expr,
        $((
            $member_ident:literal,
            $member_schema:ident,
            $member_traits:expr
        )), +
    ) => {
        pub static $schema_name: LazyLock<SchemaRef> = LazyLock::new(|| {
            $builder
            $(.put_member($member_ident, &$member_schema, $member_traits))
            *
            .build()
        });
    };
}

// Create a lazy, static ShapeId
#[macro_export]
macro_rules! lazy_shape_id {
    ($id_name:ident, $identifier:literal) => {
        static $id_name: LazyLock<ShapeId> = LazyLock::new(|| ShapeId::from($identifier));
    };
}

// Add a StaticTraitId implementation for a SmithyTrait.
#[macro_export]
macro_rules! static_trait_id {
    ($trait_struct:ident, $id_var:ident, $id_name:literal) => {
        lazy_shape_id!($id_var, $id_name);
        impl StaticTraitId for $trait_struct {
            fn trait_id() -> &'static ShapeId {
                &$id_var
            }
        }
    };
}

// Creates an implementation for a "marker" trait that contains no data
#[macro_export]
macro_rules! annotation_trait {
    ($trait_struct:ident, $id_var:ident, $id_name:literal) => {
        pub struct $trait_struct {}
        impl $trait_struct {
            #[must_use]
            pub const fn new() -> Self {
                Self {}
            }
        }
        impl Default for $trait_struct {
            fn default() -> Self {
                Self::new()
            }
        }
        static_trait_id!($trait_struct, $id_var, $id_name);
        impl SmithyTrait for $trait_struct {
            fn id(&self) -> &ShapeId {
                &$id_var
            }

            fn value(&self) -> &DocumentValue {
                &DocumentValue::Null
            }
        }
    };
}
