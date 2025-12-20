#![allow(dead_code)]

use crate::{
    prelude::UNIT,
    schema::{SchemaRef, StaticSchemaShape},
    serde::{
        de::{DeserializeWithSchema, Deserializer, Error},
        se::{SerializeWithSchema, Serializer, StructSerializer},
    },
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

impl SerializeWithSchema for Unit {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // Writes an empty structure
        serializer.write_struct(schema, 0usize)?.end(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for Unit {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_struct(schema, Unit, |_, member, _| {
            // Consumer should NEVER be called on unit schemas as that
            // would imply that the unit has members
            Err(D::Error::custom(format!(
                "Attempted to read member `{:?}` on Unit type",
                member.id().member()
            )))
        })
    }
}
