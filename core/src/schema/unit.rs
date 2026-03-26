use crate::{
    prelude::UnitTypeTrait,
    schema::{Schema, StaticSchemaShape},
    smithy,
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
/// </div>
#[derive(PartialEq, Clone)]
pub struct Unit;

// ============================================================================
// Unit Type
// ---------
// Unit types is used by unions and operations to indicate input/output
// or variants that have no meaningful value
// ============================================================================
smithy!("smithy.api#Unit": {
    /// Empty type representation used in Unions and Operations
    @UnitTypeTrait::builder().build();
    structure UNIT {}
});

impl StaticSchemaShape for Unit {
    #[inline]
    fn schema() -> &'static Schema {
        &UNIT
    }
}
