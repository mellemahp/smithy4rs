//! Operation shape support.
//!
//! Operations are the core unit of work in Smithy services. Each operation
//! has an input shape, output shape, and optionally a set of error shapes.

use crate::{
    schema::{Schema, SchemaShape, StaticSchemaShape},
    serde::{BuildableShape, se::SerializeWithSchema},
};

/// Declares the associated input and output types for a Smithy operation.
///
/// This is the trait that operation types implement directly. It only requires
/// declaring the `Input` and `Output` associated types — the schema accessor
/// methods are provided automatically by the blanket [`Operation`] impl.
///
/// For operations with no meaningful input or output, use
/// [`Unit`](crate::schema::Unit).
// TODO(errors): Add error support via a type registry. Operations can return
// multiple error types, and the correct one must be selected at runtime based
// on error codes or discriminators.
// TODO(resources): Once we get far enough, we'll want to tie in the enclosing resource shape
// TODO(services): Once we get far enough, we'll want to tie in the enclosing service shape
pub trait OperationShape: SchemaShape {
    /// The input type for this operation.
    type Input: SerializeWithSchema + BuildableShape;

    /// The output type for this operation.
    type Output: SerializeWithSchema + BuildableShape;
}

/// Provides runtime access to an operation's input and output schemas.
///
/// This trait is blanket-implemented for all types that implement both
/// [`OperationShape`] and [`StaticSchemaShape`]. The `input_schema` and
/// `output_schema` methods are derived from the
/// [`OperationSchema`](crate::schema::OperationSchema) embedded in the
/// operation's static schema.
pub trait Operation: OperationShape {
    /// Get the input shape's schema.
    fn input_schema(&self) -> &Schema;

    /// Get the output shape's schema.
    fn output_schema(&self) -> &Schema;
}

impl<T: OperationShape + StaticSchemaShape> Operation for T {
    fn input_schema(&self) -> &Schema {
        &<T as StaticSchemaShape>::schema()
            .as_operation()
            .expect("OperationShape schema must be an OperationSchema")
            .input
    }

    fn output_schema(&self) -> &Schema {
        &<T as StaticSchemaShape>::schema()
            .as_operation()
            .expect("OperationShape schema must be an OperationSchema")
            .output
    }
}
