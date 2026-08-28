/// # Smithy Schema Macro
/// Creates a lazily-resolved smithy [`Schema`](crate::schema::SchemaValue) from a user-friend DSL
/// that tries to mimic the Smithy IDL syntax.
///
/// Generated schemas can be used by `smithy4rs` proc macros to automatically implement
/// schema-guided (de)serialization for structures and enums.
///
/// ```rust, ignore
/// smithy!("test#SimpleStruct": {
///     structure SIMPLE_STRUCT {
///         SIMPLE_FIELD_A: STRING = "field_a"
///         SIMPLE_FIELD_B: INTEGER = "field_b"
///     }
/// });
///
/// #[derive(SmithyShape, PartialEq)]
/// #[schema(schema = SIMPLE_STRUCT)]
/// pub struct SimpleStruct {
///     #[schema(schema = SIMPLE_FIELD_A)]
///     pub field_a: String,
///     #[schema(schema = SIMPLE_FIELD_B)]
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
    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        boolean $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_boolean(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        byte $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_byte(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        short $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_short(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        integer $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_integer(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        long $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_long(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        float $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_float(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        double $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_double(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        bigInteger $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_big_integer(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        bigDecimal $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_big_decimal(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        timestamp $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_timestamp(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        string $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_string(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        blob $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_blob(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        document $($path:ident)::+
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_document(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )]
        );
    );

    // === Enums ===
    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        enum $($path:ident)::+ {$(
            $_variant:ident = $value:literal
        )*}
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_enum(
                $crate::smithy!(@id $($path)::+),
                Box::new([$($value),*]),
                $crate::traits!($($t),*)
            )]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        intEnum $($path:ident)::+ {$(
            $_variant:ident = $value:literal
        )*}
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::create_int_enum(
                $crate::smithy!(@id $($path)::+),
                Box::new([$($value),*]),
                $crate::traits!($($t),*)
            )]
        );
    );

    // === Collections ====

    // Lists must have member named "member" that may also have traits applied.
    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        list $($path:ident)::+ {
            $(@$m:expr;)*
            member: $member:ident
        }
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::list_builder(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )],
            [(member, $member, $crate::traits!($($m),*))]
        );
    );

    // Maps must have members named "key" and "value that may also have traits applied.
    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        map $($path:ident)::+ {
            $(@$k:expr;)*
            key: $key:ident
            $(@$v:expr;)*
            value: $value:ident
        }
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::map_builder(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )],
            [(key, $key, $crate::traits!($($k),*)), (value, $value, $crate::traits!($($v),*))]
        );
    );

    // === Structure & Unions ===

    // Empty structure
    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        structure $($path:ident)::+ {}
    ) => (
        $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::structure_builder(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            ).build()]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        structure $($path:ident)::+ {$(
            $(@$m:expr;)*
            $member_name:ident : $member_schema:tt
        )*}
    ) => (
       $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::structure_builder(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )],
            [$(($member_name, $member_schema, $crate::traits!($($m),*))),*]
        );
    );

    (
        $(#[$outer:meta])*
        $(@$t:expr;)*
        union $($path:ident)::+ {$(
            $(@$m:expr;)*
            $member_name:ident : $member_schema:tt
        )*}
    ) => (
       $crate::smithy!(@inner
            [$($path)::+],
            [$(@attr[$outer]),*],
            [$crate::schema::Schema::union_builder(
                $crate::smithy!(@id $($path)::+),
                $crate::traits!($($t),*)
            )],
            [$(($member_name, $member_schema, $crate::traits!($($m),*))),*]
        );
    );

    // // === Service Shapes ===
    // // TODO(service shapes): Add Operation, Resource, Service schema macros

    // ============================================================================
    // Actual impl of schema
    //
    // PRIVATE API
    // ============================================================================

    // Strip off namespace recursively to get shape ident then forward
    (
        @inner
        [$head:ident :: $($rest:tt)+],
        [$(@attr[$outer:meta]),*],
        [$builder:expr],
        [$(($member_name:ident, $member_schema:tt, $member_traits:expr)),+ $(,)?]
    ) => {
        $crate::smithy!(
            @inner
            [$($rest)+],
            [$(@attr[$outer]),*],
            [$builder],
            [ $(($member_name, $member_schema, $member_traits)),* ]
        );
    };

    (
        @inner
        [$schema_name:ident],
        [$(@attr[$outer:meta]),*],
        [$builder:expr],
        [$(($member_name:ident, $member_schema:tt, $member_traits:expr)),+ $(,)?]
    ) => {
        $crate::pastey::paste! {
            #[doc(hidden)]
            pub static [<$schema_name:snake:upper _BUILDER>]: $crate::LazyLock<$crate::Ref<$crate::schema::SchemaBuilder>> =
                $crate::LazyLock::new(|| $crate::Ref::new($builder));

            $(#[$outer])*
            pub static [<$schema_name:snake:upper>]: $crate::LazyLock<$crate::schema::Schema> = $crate::LazyLock::new(|| {
                $crate::smithy!(@build_chain (&*[<$schema_name:snake:upper _BUILDER>]), &*[<$schema_name:snake:upper _BUILDER>] $(, ($member_name, $member_schema, $member_traits))*)
            });

            #[allow(dead_code)]
            const [<$schema_name:snake:upper _KEYS>]: &[&str] = &[$(stringify!($member_name)),+];
        }
    };

    // Strip off namespace recursively to get shape ident then forward
    (
        @inner
        [$head:ident :: $($rest:tt)+],
        [$(@attr[$outer:meta]),*],
        [$builder:expr]
    ) => {
        $crate::smithy!(
            @inner
            [$($rest)+],
            [$(@attr[$outer]),*],
            [$builder]
        );
    };

    // No-op (i.e. no members)
    (
        @inner
        [$schema_name:ident],
        [$(@attr[$outer:meta]),*],
        [$builder:expr]
    ) => {
        $crate::pastey::paste! {
            $(#[$outer])*
            pub static [<$schema_name:snake:upper>]: $crate::LazyLock<$crate::schema::Schema> = $crate::LazyLock::new(|| {
                $builder
            });
        }
    };
    //
    // // ============================================================================
    // // Internal helpers to build chain of member `put` statements
    // //
    // // INTERNAL API
    // // ============================================================================

    // Case - @self recursion case (matches (@self) as single tt)
    (@build_chain $builder:expr, $builder_ref:expr, ($member_name:ident, (@ self), $member_traits:expr) $(, $rest:tt)*) => {
        $crate::smithy!(@build_chain $builder.put_member(stringify!($member_name), $builder_ref, $member_traits), $builder_ref $(, $rest)*)
    };
    // Case - members to add to chain.
    (@build_chain $builder:expr, $builder_ref:expr, ($member_name:ident, $member_schema:tt, $member_traits:expr) $(, $rest:tt)*) => {
        $crate::smithy!(@build_chain $builder.put_member(stringify!($member_name), &$member_schema, $member_traits), $builder_ref $(, $rest)*)
    };
    // Case - No more members to process so schema can be built.
    (@build_chain $builder:expr, $builder_ref:expr $(,)?) => {
        $builder.build()
    };

    // ============================================================================
    // Internal helpers for ID/Name parsing
    //
    // INTERNAL API
    // ============================================================================
    (@id $head:ident :: $next:ident :: $($rest:tt)+) => {
        concat!(stringify!($head), ".", $crate::smithy!(@id $next :: $($rest)+))
    };
    (@id $prev:ident :: $last:ident) => {
        concat!(stringify!($prev), "#", stringify!($last))
    };
}

// ============================================================================
// Helper Macros
// ----------------------------------------------------------------------------
// These macros are generally should not be used directly
// ============================================================================

/// Helper macro for deserializing required struct members in generated code.
///
/// This macro simplifies the pattern of checking if a member schema matches
/// and deserializing its value into the builder using a StructReader.
#[doc(hidden)]
#[macro_export]
macro_rules! deserialize_member {
    ($member:expr, $schema:expr, $reader:expr, $builder:expr, $method:ident, $ty:ty) => {
        if $member == *$schema {
            let value: $ty = $reader.read_value($member)?;
            $builder = $builder.$method(value);
            continue;
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
    ($member:expr, $schema:expr, $reader:expr, $builder:expr, $method:ident, $ty:ty) => {
        if $member == *$schema {
            let value: Option<$ty> = $reader.read_value($member)?;
            if let Some(v) = value {
                $builder = $builder.$method(v);
            }
            continue;
        }
    };
}

/// Helper macro that creates a list of traits for use in Schema builders
///
/// <div class ="note">
/// **NOTE**: Unlike the `vec!` macro, the default here creates a _unallocated_ vec
/// so there is no added overhead from always using it in schema macros.
/// </div>
///
/// # Panics
/// Invalid trait definitions will cause the code generated by this macro to panic.
/// Trait definitions here should always be valid, however as they should be checked
/// by the Smithy validation prior to code generation.
#[doc(hidden)]
#[macro_export]
macro_rules! traits {
    () => { Vec::new() };
    ($($x:expr),+ $(,)?) => (
        vec![$($x.try_into().unwrap()),*]
    );
}

/// Adds generated file from the "rust-types" Smithy build plugin.
///
/// If used with no argument then this will import from the default
/// Smithy `source` projection.
///
/// A projection name can be provided to select a specific projection
/// to import.
#[macro_export]
macro_rules! generated_shapes {
    () => {
        generated_shapes!("source");
    };
    ($projection:literal) => {
        include!(concat!(
            env!("SMITHY_OUTPUT_DIR"),
            "/",
            $projection,
            "/rust-types/smithy-generated.rs"
        ));
    };
}

/// Constructs a map document from a set of key-value pairs
///
/// This macro primarily exists to support dynamic traits in codegen, but
/// can also be useful for testing.
#[doc(hidden)]
#[macro_export]
macro_rules! doc_map {
    ($($key:expr => $val:expr),* $(,)?) => {
        $crate::IndexMap::<String, Box<dyn $crate::schema::Document>>::from_iter([$(($key.into(), $val.into()),)*])
    }
}

/// Constructs an ordered string map (`IndexMap<String,_>`) from a set of key-value pairs
///
/// This macro primarily exists to support trait initializers, but
/// can also be useful for testing.
#[macro_export]
macro_rules! string_map {
    ($($key:expr => $val:expr),* $(,)?) => {
        $crate::IndexMap::<String, _>::from_iter([$(($key.into(), $val.into()),)*])
    }
}
