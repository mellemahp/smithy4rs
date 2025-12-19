/// # Smithy Schema Macro
/// Creates a lazily-resolved smithy [`Schema`](crate::schema::Schema) from a user-friend DSL
/// that tries to mimic the Smithy IDL syntax.
///
/// Generated schemas can be used by `Smithy4rs` proc macros to automatically implement
/// schema-guided (de)serialization for structures and enums.
///
/// ```rust
/// use smithy4rs_core::smithy;
/// use smithy4rs_core_derive::SmithyStruct;
///
/// smithy!("test#SimpleStruct": {
///     structure SIMPLE_STRUCT_SCHEMA {
///         SIMPLE_FIELD_A: STRING = "field_a"
///         SIMPLE_FIELD_B: INTEGER = "field_b"
///     }
/// });
///
/// #[derive(SmithyStruct, Debug, PartialEq)]
/// #[smithy_schema(SIMPLE_STRUCT_SCHEMA)]
/// pub struct SimpleStruct {
///     #[smithy_schema(SIMPLE_FIELD_A)]
///     pub field_a: String,
///     #[smithy_schema(SIMPLE_FIELD_B)]
///     pub field_b: i32,
/// }
/// ```
#[macro_export]
macro_rules! smithy {
    // Hide implementation details.
    ($($smithy:tt)+) => {
        $crate::smithy_internal!{$($smithy)+}
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! smithy_internal {
    // ============================================================================
    // Main implementation.
    //
    // Must be invoked as: smithy_internal!($($smithy)+)
    // ============================================================================
    // === Simple types ===
    ($id:literal: {
        $(@$t:expr;)*
        boolean $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_boolean($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        byte $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_byte($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        short $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_short($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        integer $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_integer($id, $crate::traits!($($t),*))
        );
    );

     ($id:literal: {
        $(@$t:expr;)*
        long $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_long($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        float $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_float($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        double $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_double($id, $crate::traits!($($t),*))
        );
    );


    ($id:literal: {
        $(@$t:expr;)*
        bigInteger $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_big_integer($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        bigDecimal $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_big_decimal($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        timestamp $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_timestamp($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        string $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_string($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        blob $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_blob($id, $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        document $name:ident
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_document($id, $crate::traits!($($t),*))
        );
    );

    // === Enums ===
    ($id:literal: {
        $(@$t:expr;)*
        enum $name:ident {
            $($_variant:tt = $value:literal),*
        }
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_enum($id, vec!($($value),*), $crate::traits!($($t),*))
        );
    );

    ($id:literal: {
        $(@$t:expr;)*
        intEnum $name:ident {
            $($_variant:tt = $value:literal),*
        }
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::create_int_enum($id, vec!($($value),*), $crate::traits!($($t),*))
        );
    );


    // === Collections ====

    // Lists must have member named "member" that may also have traits applied.
    ($id:literal: {
        $(@$t:expr;)*
        list $name:ident {
            $(@$m:expr;)* member: $member:ident
        }
    }) => (
        $crate::smithy!(@inner
            $name,
            $crate::schema::Schema::list_builder($id, $crate::traits!($($t),*)),
            ("member", $member, $crate::traits!($($m),*))
        );
    );

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
            $crate::schema::Schema::map_builder($id, $crate::traits!($($t),*)),
            ("key", $key, $crate::traits!($($k),*)),
            ("value", $value, $crate::traits!($($v),*))
        );
    );
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
            $crate::schema::Schema::structure_builder($id, $crate::traits!($($t),*)),
            $(($member_ident, $member_name, $member_schema, $crate::traits!($($m),*))),*
        );
    );

    // === Union ===
    // TODO(union): Add union shape macro


    // === Service Shapes ===
    // TODO(service shapes): Add Operation, Resource, Service schema macros

    // ============================================================================
    // Actual impl of schema
    //
    // PRIVATE API
    // ============================================================================

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
                $crate::smithy!(@build_chain (&*[<$schema_name _BUILDER>]), &*[<$schema_name _BUILDER>] $(, ($member_ident, $member_schema, $member_traits))*)
            });

            $(pub static [<_$schema_name _MEMBER_$member_schema_name>]: $crate::LazyLock<&$crate::schema::SchemaRef> =
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
                $crate::smithy!(@build_chain (&*[<$schema_name _BUILDER>]), &*[<$schema_name _BUILDER>] $(, ($member_ident, $member_schema, $member_traits))*)
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

    // ============================================================================
    // Internal helpers to build chain of member `put` statements
    //
    // INTERNAL API
    // ============================================================================

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

/// # Smithy Annotation Trait Macro
/// Creates a [`SmithyTrait`](crate::schema::SmithyTrait) implementation for a marker trait that contains no data.
///
/// An example of a Smithy "annotation" trait is the [@idempotent](https://smithy.io/2.0/spec/behavior-traits.html#smithy-api-idempotent-trait) trait:
/// ```smithy
/// @trait
/// structure idempotent {}
/// ```
///
/// ## Example
/// The following example generates an empty struct, `IdempotencyTokenTrait`,
/// that implements [`SmithyTrait`](crate::schema::SmithyTrait) and has a
/// [`StaticTraitId`](crate::schema::StaticTraitId) of `"smithy.api#IdempotencyToken"`.
///
/// ```rust
/// use smithy4rs_core::annotation_trait;
///
/// annotation_trait!(IdempotencyTokenTrait, "smithy.api#IdempotencyToken");
/// ```
#[macro_export]
macro_rules! annotation_trait {
    ($trait_struct:ident, $id:literal) => {
        #[derive(Debug)]
        pub struct $trait_struct;
        impl Default for $trait_struct {
            fn default() -> Self {
                Self
            }
        }
        $crate::static_trait_id!($trait_struct, $id);
        impl SmithyTrait for $trait_struct {
            fn id(&self) -> &ShapeId {
                $trait_struct::trait_id()
            }

            fn value(&self) -> &DocumentValue {
                &DocumentValue::Null
            }
        }
    };
}

/// # Smithy String Trait Macro
/// Trait definitions that contain only a string value.
///
/// An example of a `string` trait is the [`@documentation`](https://smithy.io/2.0/spec/documentation-traits.html#smithy-api-documentation-trait) trait:
/// ```smithy
/// @trait
/// string documentation
/// ```
///
/// ### Example
/// The following example creates a struct, `MediaTypeTrait`, with a single public
/// field (`media_type`) that implements [`SmithyTrait`](crate::schema::SmithyTrait) and has a
/// [`StaticTraitId`](crate::schema::StaticTraitId) of `"smithy.api#mediaType"`
///
/// ```rust
/// use smithy4rs_core::string_trait;
///
/// string_trait!(MediaTypeTrait, media_type, "smithy.api#mediaType");
/// ```
#[macro_export]
macro_rules! string_trait {
    ($trait_struct:ident, $value_name:ident, $id:literal) => {
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
        $crate::static_trait_id!($trait_struct, $id);
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

// ============================================================================
// Helper Macros
// ----------------------------------------------------------------------------
// These macros are generally should not be used directly
// ============================================================================

/// Helper macro to add [`crate::schema::StaticTraitId`] implementation for a SmithyTrait.
#[doc(hidden)]
#[macro_export]
macro_rules! static_trait_id {
    ($trait_struct:ident, $id:literal) => {
        impl StaticTraitId for $trait_struct {
            #[inline]
            fn trait_id() -> &'static ShapeId {
                static ID: $crate::LazyLock<$crate::schema::ShapeId> =
                    $crate::LazyLock::new(|| $crate::schema::ShapeId::from($id));
                &ID
            }
        }
    };
}

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

/// Helper macro that creates a list of traits for use in Schema builders
///
/// <div class ="note">
/// **NOTE**: Unlike the `vec!` macro, the default here creates a _unallocated_ vec
/// so there is no added overhead from always using it in schema macros.
/// </div>
#[doc(hidden)]
#[macro_export]
macro_rules! traits {
    () => { Vec::new() };
    ($($x:expr),+ $(,)?) => (
        vec![$($x.into()),*]
    );
}
