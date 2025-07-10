// use crate::schema::SchemaRef;
// use crate::serde::de::{Deserialize, Deserializer};
// use crate::serde::SerializeShape;
//
// pub trait ShapeBuilder<'a, T>: Deserialize<'a> {
//     fn new() -> Self;
//
//     // TODO: Should this set the validator?
//     // TODO: Should probably return a build-validation error.
//     fn build(self) -> Result<T, String>;
//
//     fn deserialize_member<D: Deserializer<'a>>(
//         &mut self,
//         member_schema: &SchemaRef,
//         member_deserializer: D,
//     ) -> Result<(), D::Error>;
// }
