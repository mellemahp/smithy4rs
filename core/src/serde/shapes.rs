// TODO: SerializableShapes types should implement `Into<Document>` for conversion.

use crate::schema::SchemaRef;

/// Returns the schema for a shape
///
/// This schema is typically statically defined in generate code.
pub trait SchemaShape {
    /// Schema of this shape.
    fn schema(&self) -> &SchemaRef;
}

// pub trait SerializeShape: Serialize {
//     // Should schema be moved to a Trait that can be shared with deserializeShape?
//     fn schema(&self) -> &SchemaRef;
//
//     fn serialize_shape<S: Serializer>(&self, serializer: &mut S) -> SerializerResult<S::Error> {
//         self.serialize(self.schema(), serializer)
//     }
//
//     fn get_member<T>(&self) -> Option<&T> {
//         todo!()
//     }
// }
