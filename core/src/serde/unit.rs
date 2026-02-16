use crate::{
    schema::{Schema, Unit},
    serde::{
        de::{DeserializeWithSchema, Deserializer, Error, StructReader},
        se::{SerializeWithSchema, Serializer, StructSerializer},
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
    fn deserialize_with_schema<D>(_schema: &Schema, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut reader = deserializer.read_struct()?;

        // Unit types should have no members
        if let Some(ref field_name) = reader.read_name()? {
            return Err(D::Error::custom(format!(
                "Attempted to read member `{field_name}` on Unit type"            )));
        }

        Ok(Unit)
    }
}
