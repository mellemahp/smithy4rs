// TODO: SerializableShapes types should implement `Into<Document>` for conversion.

use crate::serde::se::Serialize;

/// Marker Trait used to differentiate between generated shapes and Documents for
/// some blanket impelementations.
///
/// NOTE: In general you should not need to implement this yourself
pub trait SerializableShape: Serialize {}

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
