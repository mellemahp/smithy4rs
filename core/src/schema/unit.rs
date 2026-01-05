use crate::{
    prelude::UNIT,
    schema::{SchemaRef, StaticSchemaShape},
};

/// # Unit type
///
/// This structure represents a member without a meaningful value. It is
/// used by Operations to represent an empty input/output and by and Unions
/// to represent a variant that contains no value.
///
/// <div class="note">
/// **NOTE**: Units are always serialized and deserialized as empty structs.
/// So for example in a JSON protocol the Unit would be represented as `{}`.
pub struct Unit;

impl StaticSchemaShape for Unit {
    fn schema() -> &'static SchemaRef {
        &UNIT
    }
}
