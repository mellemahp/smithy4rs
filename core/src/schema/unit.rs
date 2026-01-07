use crate::{
    annotation_trait,
    schema::{SchemaRef, StaticSchemaShape},
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
pub struct Unit;

// ============================================================================
// Unit Type
// ---------
// Unit types is used by unions and operations to indicate input/output
// or variants that have no meaningful value
// ============================================================================
annotation_trait!(UnitTypeTrait, "smithy.api#UnitTypeTrait");

smithy!("smithy.api#Unit": {
    @UnitTypeTrait;
    structure UNIT {}
});

impl StaticSchemaShape for Unit {
    fn schema() -> &'static SchemaRef {
        &UNIT
    }
}
