use crate::schema::SchemaRef;
use crate::serde::se::{Serialize, Serializer, SerializerResult};

pub trait SerializeShape: Serialize {
    // Should schema be moved to a Trait that can be shared with deserializeShape?
    fn schema(&self) -> &SchemaRef;

    fn serialize_shape<S: Serializer>(&self, serializer: &mut S) -> SerializerResult<S::Error> {
        self.serialize(self.schema(), serializer)
    }

    fn get_member<T>(&self) -> Option<&T> {
        todo!()
    }
}

