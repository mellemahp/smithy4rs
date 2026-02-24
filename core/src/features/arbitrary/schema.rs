use arbitrary::{Arbitrary, MaxRecursionReached};
use num_bigint::BigInt;

use crate::{
    prelude::RequiredTrait,
    schema::{ScalarSchema, Schema, SchemaValue, ShapeType},
};

/// Get a size hint for how many bytes out of an [`Unstructured`](arbitrary::Unstructured)
/// the corresponding field represented by this schema needs to construct itself.
///
/// This trait is used to support `Arbitrary` implementation on generated shapes.
pub trait TrySizeHint {
    /// Get a size hint for how many bytes out of an [`Unstructured`](arbitrary::Unstructured)
    /// the corresponding field represented by this schema needs to construct itself.
    ///
    /// # Errors
    /// Returns a `MaxRecursionReached` error if a
    /// maximum recursion depth (20) is reached while trying to resolve
    /// the size hint.
    fn try_size_hint(&self, depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached>;
}

impl TrySizeHint for Schema {
    fn try_size_hint(&self, depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached> {
        arbitrary::size_hint::try_recursion_guard(depth, |depth| {
            if self.contains_type::<RequiredTrait>() {
                size_of_value(self, depth)
            } else {
                Ok(arbitrary::size_hint::and(
                    // Optional schemas require an additional `bool` check
                    // to determine if they are to be treated as present or not
                    <bool as Arbitrary>::size_hint(depth),
                    // Either empty or present
                    arbitrary::size_hint::or((0, Some(0)), size_of_value(self, depth)?),
                ))
            }
        })
    }
}

fn size_of_value(
    value: &SchemaValue,
    depth: usize,
) -> Result<(usize, Option<usize>), MaxRecursionReached> {
    match value {
        SchemaValue::Scalar(ScalarSchema { shape_type, .. }) => {
            match shape_type {
                ShapeType::Blob => Vec::<u8>::try_size_hint(depth),
                ShapeType::Boolean => <bool as Arbitrary>::try_size_hint(depth),
                ShapeType::String => <String as Arbitrary>::try_size_hint(depth),
                ShapeType::Timestamp => <i64 as Arbitrary>::try_size_hint(depth),
                ShapeType::Byte => <i8 as Arbitrary>::try_size_hint(depth),
                ShapeType::Short => <i16 as Arbitrary>::try_size_hint(depth),
                ShapeType::Integer => <i32 as Arbitrary>::try_size_hint(depth),
                ShapeType::Long => <i64 as Arbitrary>::try_size_hint(depth),
                ShapeType::Float => <f32 as Arbitrary>::try_size_hint(depth),
                ShapeType::Double => <f64 as Arbitrary>::try_size_hint(depth),
                ShapeType::BigInteger => <BigInt as Arbitrary>::try_size_hint(depth),
                ShapeType::BigDecimal => Ok(arbitrary::size_hint::and_all(&[
                    // Scale
                    <i64 as Arbitrary>::try_size_hint(depth)?,
                    // Starting big-int
                    <BigInt as Arbitrary>::try_size_hint(depth)?,
                    // Divisor
                    <f32 as Arbitrary>::try_size_hint(depth)?,
                ])),
                _ => unreachable!("Scalar schemas cannot be created for non-scalar types"),
            }
        }
        SchemaValue::Struct(structure) => {
            let sizes = structure
                .members
                .values()
                .map(|v| v.try_size_hint(depth))
                .collect::<Result<Vec<_>, _>>()?;
            if structure.shape_type == ShapeType::Union {
                // Unions need only one member set, but also need a usize
                // to pick which member to generate.
                Ok(arbitrary::size_hint::and(
                    <usize as Arbitrary>::size_hint(depth),
                    arbitrary::size_hint::or_all(&sizes),
                ))
            } else {
                Ok(arbitrary::size_hint::and_all(&sizes))
            }
        }
        SchemaValue::Enum(_) => <usize as Arbitrary>::try_size_hint(depth),
        SchemaValue::IntEnum(_) => <usize as Arbitrary>::try_size_hint(depth),
        SchemaValue::List(_) => {
            Ok(arbitrary::size_hint::and(
                // usize used to determine how many entries to generate
                <usize as Arbitrary>::try_size_hint(depth)?,
                // 0 or more entries(unbounded max)
                (0, None),
            ))
        }
        SchemaValue::Map(_) => {
            Ok(arbitrary::size_hint::and_all(&[
                // usize used to determine how many entries to generate
                <usize as Arbitrary>::try_size_hint(depth)?,
                // 0 or more entries(unbounded max)
                (0, None),
            ]))
        }
        SchemaValue::Member(member) => (*member.target).try_size_hint(depth),
        SchemaValue::Operation(operation) => Ok(arbitrary::size_hint::and_all(&[
            operation.input.try_size_hint(depth)?,
            operation.output.try_size_hint(depth)?,
        ])),
    }
}
