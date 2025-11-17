//! Macros shared
//! Primarily these macros are used to construct schemas and traits.

/// Helper macro for deserializing required struct members in generated code.
///
/// This macro simplifies the pattern of checking if a member schema matches
/// and deserializing its value into the builder.
#[doc(hidden)]
#[macro_export]
macro_rules! deserialize_member {
    ($member:expr, $schema:expr, $de:expr, $builder:expr, $method:ident, $ty:ty) => {
        if std::sync::Arc::ptr_eq($member, $schema) {
            let value = <$ty as $crate::serde::deserializers::DeserializeWithSchema>::deserialize_with_schema($member, $de)?;
            return Ok($builder.$method(value));
        }
    };
}

/// Helper macro for deserializing optional struct members in generated code.
///
/// This macro handles optional fields by deserializing as Option<T> and only
/// calling the builder method if Some.
#[doc(hidden)]
#[macro_export]
macro_rules! deserialize_optional_member {
    ($member:expr, $schema:expr, $de:expr, $builder:expr, $method:ident, $ty:ty) => {
        if std::sync::Arc::ptr_eq($member, $schema) {
            let value = <Option::<$ty> as $crate::serde::deserializers::DeserializeWithSchema>::deserialize_with_schema($member, $de)?;
            if let Some(v) = value {
                return Ok($builder.$method(v));
            }
            return Ok($builder);
        }
    };
}

// Create a list of traits for use in Schema builders
#[macro_export]
macro_rules! traits {
    () => { Vec::new() };
    ($($x:expr),+ $(,)?) => (
        vec![$($x.into()),*]
    );
}

// Create a lazy, static Schema definition
#[macro_export]
macro_rules! lazy_schema {
    // Internal helper to build member chain - @self recursion case (matches (@self) as single tt)
    (@build_chain $builder:expr, $builder_ref:expr, ($member_ident:literal, (@ self), $member_traits:expr) $(, $rest:tt)*) => {
        $crate::lazy_schema!(@build_chain $builder.put_member($member_ident, $builder_ref, $member_traits), $builder_ref $(, $rest)*)
    };
    // Internal helper to build member chain - normal schema case
    (@build_chain $builder:expr, $builder_ref:expr, ($member_ident:literal, $member_schema:tt, $member_traits:expr) $(, $rest:tt)*) => {
        $crate::lazy_schema!(@build_chain $builder.put_member($member_ident, &$member_schema, $member_traits), $builder_ref $(, $rest)*)
    };
    // Internal helper to build member chain - base case (no more members)
    (@build_chain $builder:expr, $builder_ref:expr $(,)?) => {
        $builder.build()
    };

    // Public API: with member schema names
    (
        $schema_name:ident,
        $builder:expr,
        $(($member_schema_name:ident, $member_ident:literal, $member_schema:tt, $member_traits:expr)),+ $(,)?
    ) => {
        $crate::pastey::paste! {
            pub static [<$schema_name _BUILDER>]: $crate::LazyLock<std::sync::Arc<$crate::schema::SchemaBuilder>> =
                $crate::LazyLock::new(|| std::sync::Arc::new($builder));

            pub static $schema_name: $crate::LazyLock<$crate::schema::SchemaRef> = $crate::LazyLock::new(|| {
                $crate::lazy_schema!(@build_chain (&*[<$schema_name _BUILDER>]), &*[<$schema_name _BUILDER>] $(, ($member_ident, $member_schema, $member_traits))*)
            });

            $(pub static $member_schema_name: $crate::LazyLock<&$crate::schema::SchemaRef> =
                $crate::LazyLock::new(|| $schema_name.expect_member($member_ident));
            )*
        }
    };

    // Public API: without member schema names
    (
        $schema_name:ident,
        $builder:expr,
        $(($member_ident:literal, $member_schema:tt, $member_traits:expr)),+ $(,)?
    ) => {
        $crate::pastey::paste! {
            pub static [<$schema_name _BUILDER>]: $crate::LazyLock<std::sync::Arc<$crate::schema::SchemaBuilder>> =
                $crate::LazyLock::new(|| std::sync::Arc::new($builder));

            pub static $schema_name: $crate::LazyLock<$crate::schema::SchemaRef> = $crate::LazyLock::new(|| {
                $crate::lazy_schema!(@build_chain (&*[<$schema_name _BUILDER>]), &*[<$schema_name _BUILDER>] $(, ($member_ident, $member_schema, $member_traits))*)
            });
        }
    };

    // Public API: no-op (just wraps the builder expression directly, no members)
    (
        $schema_name:ident,
        $builder:expr
    ) => {
        pub static $schema_name: $crate::LazyLock<$crate::schema::SchemaRef> = $crate::LazyLock::new(|| {
            $builder
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
            #[inline]
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
        #[derive(Debug)]
        pub struct $trait_struct;
        impl Default for $trait_struct {
            fn default() -> Self {
                Self
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

// Trait definitions that contain only a string value
#[macro_export]
macro_rules! string_trait {
    ($trait_struct:ident, $id_var:ident, $value_name:ident, $id_name:literal) => {
        #[derive(Debug)]
        pub struct $trait_struct {
            $value_name: String,
            value: DocumentValue,
        }
        impl $trait_struct {
            pub fn $value_name(&self) -> &str {
                &self.$value_name
            }

            #[must_use]
            pub fn new($value_name: &str) -> Self {
                $trait_struct {
                    $value_name: $value_name.to_string(),
                    value: DocumentValue::String($value_name.to_string()),
                }
            }
        }
        static_trait_id!($trait_struct, $id_var, $id_name);
        impl SmithyTrait for $trait_struct {
            fn id(&self) -> &ShapeId {
                $trait_struct::trait_id()
            }

            fn value(&self) -> &DocumentValue {
                &self.value
            }
        }
    };
}
