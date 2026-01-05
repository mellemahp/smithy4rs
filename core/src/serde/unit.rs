use crate::{
    schema::{SchemaRef, Unit},
    serde::{
        de::{DeserializeWithSchema, Deserializer, Error},
        se::{SerializeWithSchema, Serializer, StructSerializer},
    },
};

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
