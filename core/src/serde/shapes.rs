use crate::schema::Schema;
use crate::serde::se::{Serialize, Serializer, SerializerResult};

pub trait SerializeShape: Serialize {
    fn schema(&self) -> &Schema;

    fn serialize_shape<'a, S: Serializer<'a>>(&self, serializer: &mut S) -> SerializerResult<S::Error> {
        self.serialize(self.schema(), serializer)
    }

    fn get_member<T>(&self) -> Option<&T> {
        todo!()
    }
}

pub trait BuildShape: SerializeShape {
    //fn builder<B: Builder<Self>>() -> B;
}

pub trait Builder<T: SerializeShape> {

}