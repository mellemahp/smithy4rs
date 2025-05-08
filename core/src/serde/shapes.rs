use crate::schema::Schema;
use crate::serde::se::{Serialize, Serializer};

pub trait SerializeShape: Serialize {
    fn schema(&self) -> &Schema;

    fn serialize_shape<S: Serializer>(&self, serializer: &mut S) -> Result<S::Ok, S::Error> {
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