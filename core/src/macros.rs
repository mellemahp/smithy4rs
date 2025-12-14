//! Macros shared
//! Primarily these macros are used to construct schemas and traits.

use std::process::id;
use crate::prelude::{DefaultTrait, RequiredTrait};
use crate::schema::{DocumentValue, Schema};

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

/// Creates a lazily-resolved smithy schema.
#[macro_export]
macro_rules! smithy {
    // Hide implementation details.
    ($($smithy:tt)+) => {
        $crate::smithy_internal!($($smithy)+);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! smithy_internal {
    //////////////////////////////////////////////////////////////////////////
    // Main implementation.
    //
    // Must be invoked as: smithy_internal!($($json)+)
    //////////////////////////////////////////////////////////////////////////
    // === Simple types ===
    ($id:literal: {
        $(@$t:expr;)*
        boolean $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_boolean($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        byte $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_byte($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        short $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_short($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        integer $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_integer($id, $crate::traits!($($t),*))
        );
    );

     ($id:literal: {
        $(@$t:expr;)*
        long $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_long($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        float $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_float($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        double $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_double($id, $crate::traits!($($t),*))
        );
    );


    ($id:literal: {
        $(@$t:expr;)*
        bigInteger $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_big_integer($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        bigDecimal $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_big_decimal($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        timestamp $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_timestamp($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        string $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_string($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        blob $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::create_blob($id, $crate::traits!($($t),*))
        );
    );

    // TODO: Document

    // === Enums ===
    // TODO: ENUMS!

    // === Structure ===
    // May or may not have members
    ($id:literal: {
        $(@$t:expr;)*
        structure $name:ident {$(
            $(@$m:expr;)*
            $member_ident:ident : $member_schema:tt = $member_name:literal
        )*}
    }) => (
       $crate::smithy!(@inner
            $name,
            Schema::structure_builder($id, $crate::traits!($($t),*)),
            $(($member_ident, $member_name, $member_schema, $crate::traits!($($m),*))),*
        );
    );

    // === Union ===

    // === List ===
    // Lists must have member named "member" that may also have traits applied.
    ($id:literal: {
        $(@$t:expr;)*
        list $name:ident {
            $(@$m:expr;)* member: $member:ident
        }
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::list_builder($id, $crate::traits!($($t),*)),
            ("member", $member, $crate::traits!($($m),*))
        );
    );

    // === Map ===
   // Maps must have members named "key" and "value that may also have traits applied.
    ($id:literal: {
        $(@$t:expr;)*
        map $name:ident {
            $(@$k:expr;)*
            key: $key:ident
            $(@$v:expr;)*
            value: $value:ident
        }
    }) => (
        $crate::smithy!(@inner
            $name,
            Schema::map_builder($id, $crate::traits!($($t),*)),
            ("key", $key, $crate::traits!($($k),*)),
            ("value", $value, $crate::traits!($($v),*))
        );
    );

    // === Service Shapes ===
    // TODO: Operation, Resource, Service

    //////////////////////////////////////////////////////////////////////////
    // Actual impl of schema
    //
    // PRIVATE API
    //////////////////////////////////////////////////////////////////////////
    // Schema with members and generated static member variables (i.e. structure)
    (
        @inner
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

    // Schema that does not generate static member schema variables (i.e. List and Map)
    (
        @inner
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

    // No-op (i.e. no members)
    (
        @inner
        $schema_name:ident,
        $builder:expr
    ) => {
        pub static $schema_name: $crate::LazyLock<$crate::schema::SchemaRef> = $crate::LazyLock::new(|| {
            $builder
        });
    };

    //////////////////////////////////////////////////////////////////////////
    // Internal helpers to build chain of member `put` statements
    //
    // INTERNAL API
    //////////////////////////////////////////////////////////////////////////
    // Case - @self recursion case (matches (@self) as single tt)
    (@build_chain $builder:expr, $builder_ref:expr, ($member_ident:literal, (@ self), $member_traits:expr) $(, $rest:tt)*) => {
        $crate::smithy!(@build_chain $builder.put_member($member_ident, $builder_ref, $member_traits), $builder_ref $(, $rest)*)
    };
    // Case - members to add to chain.
    (@build_chain $builder:expr, $builder_ref:expr, ($member_ident:literal, $member_schema:tt, $member_traits:expr) $(, $rest:tt)*) => {
        $crate::smithy!(@build_chain $builder.put_member($member_ident, &$member_schema, $member_traits), $builder_ref $(, $rest)*)
    };
    // Case - No more members to process so schema can be built.
    (@build_chain $builder:expr, $builder_ref:expr $(,)?) => {
        $builder.build()
    };
}


#[cfg(test)]
mod test {
    use crate::schema::{SchemaRef, SmithyTrait, TraitRef};
    use super::*;
    use crate::prelude::{RequiredTrait, STRING};

    smithy!["my.com#MyStructure": {
        structure MY_STRUCT {
            @RequiredTrait;
            A: STRING = "a"
            B: STRING = "b"
        }
    }];

    smithy!["my.com#MyMap": {
        map MY_MAP {
            @RequiredTrait;
            key: STRING
            value: STRING
        }
    }];

    smithy!["my.com#MyList": {
        list MY_LIST {
            member: STRING
        }
    }];

    #[test]
    fn test() {
        println!("Schema: {:#?}", &*MY_STRUCT);
    }
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
