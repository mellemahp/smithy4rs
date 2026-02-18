use crate::{
    schema::{Schema, Unit},
    serde::{
        de::{DeserializeWithSchema, Deserializer, Error, StructReader},
        se::{SerializeWithSchema, Serializer, StructWriter},
    },
};

impl SerializeWithSchema for Unit {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // Writes an empty structure
        serializer.write_struct(schema, 0usize)?.end(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for Unit {
    #[cold]
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut reader = deserializer.read_struct(schema)?;

        // Unit types should have no members
        if let Some(member_schema) = reader.read_member(schema)? {
            return Err(D::Error::custom(format!(
                "Attempted to read member `{:?}` on Unit type",
                member_schema.id()
            )));
        }

        Ok(Unit)
    }
}
