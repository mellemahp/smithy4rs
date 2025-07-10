// #![allow(dead_code)]
//
// use indexmap::IndexMap;
// use smithy4rs_core::schema::shapes::ShapeId;
// use smithy4rs_core::schema::{Schema, prelude};
// use smithy4rs_core::serde::de::{Deserializable, Deserializer, ShapeBuilder};
// use smithy4rs_core::serde::se::{Serialize, Serializer, StructSerializer};
// use smithy4rs_core::serde::{FmtSerializer, SerializeShape};
// use smithy4rs_core::{lazy_member_schema, traits};
// use std::fmt::Display;
// use std::sync::LazyLock;
//
// pub static LIST_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
//     Schema::list_builder(ShapeId::from("com.example#MyList"))
//         .put_member("member", &prelude::STRING, traits![])
//         .build()
// });
//
// pub static MAP_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
//     Schema::map_builder(ShapeId::from("com.example#MyMap"))
//         .put_member("key", &prelude::STRING, traits![])
//         .put_member("value", &prelude::STRING, traits![])
//         .build()
// });
//
// pub static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
//     Schema::structure_builder(ShapeId::from("com.example#Shape"))
//         .put_member("a", &prelude::STRING, traits![])
//         .put_member("b", &prelude::STRING, traits![])
//         .put_member("list", &LIST_SCHEMA, traits![])
//         .put_member("map", &MAP_SCHEMA, traits![])
//         .build()
// });
// lazy_member_schema!(MEMBER_A, SCHEMA, "a");
// lazy_member_schema!(MEMBER_B, SCHEMA, "b");
// lazy_member_schema!(MEMBER_LIST, SCHEMA, "list");
// lazy_member_schema!(MEMBER_MAP, SCHEMA, "map");
//
// //#[derive(SerializableStruct)]
// //#[schema(SCHEMA)]
// pub(crate) struct SerializeMe {
//     pub member_a: String,
//     pub member_b: String,
//     pub list_member: Vec<String>,
//     pub map_member: IndexMap<String, String>,
// }
//
// impl SerializeMe {
//     pub const fn builder() -> SerializeMeBuilder {
//         SerializeMeBuilder::new()
//     }
// }
//
// impl SerializeShape for SerializeMe {
//     fn schema(&self) -> &'static Schema<'static> {
//         &SCHEMA
//     }
// }
// impl Serialize for SerializeMe {
//     fn serialize<S: Serializer>(
//         &self,
//         schema: &Schema,
//         serializer: &mut S,
//     ) -> Result<S::Ok, S::Error> {
//         let mut struct_ser = serializer.write_struct(schema, 5)?;
//         struct_ser.serialize_member(&MEMBER_A, &self.member_a)?;
//         struct_ser.serialize_member(&MEMBER_B, &self.member_b)?;
//         struct_ser.serialize_member(&MEMBER_LIST, &self.list_member)?;
//         struct_ser.serialize_member(&MEMBER_MAP, &self.map_member)?;
//         struct_ser.end(schema)
//     }
// }
//
// impl Display for SerializeMe {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut output = FmtSerializer::default();
//         match self.serialize_shape(&mut output) {
//             Ok(_) => write!(f, "{}", output.flush()),
//             Err(_e) => Err(std::fmt::Error {}),
//         }
//     }
// }
//
// pub struct SerializeMeBuilder {
//     pub member_a: Option<String>,
//     pub member_b: Option<String>,
//     pub list_member: Option<Vec<String>>,
//     pub map_member: Option<IndexMap<String, String>>,
// }
//
// // TODO: Add builder derive macro?
// impl SerializeMeBuilder {
//     pub const fn new() -> SerializeMeBuilder {
//         SerializeMeBuilder {
//             member_a: None,
//             member_b: None,
//             list_member: None,
//             map_member: None,
//         }
//     }
//
//     pub fn member_a(mut self, member_a: &str) -> Self {
//         self.set_member_a(member_a);
//         self
//     }
//
//     fn set_member_a(&mut self, member_a: &str) {
//         self.member_a = Some(member_a.to_string());
//     }
//
//     pub fn member_b(mut self, member_b: &str) -> Self {
//         self.set_member_b(member_b);
//         self
//     }
//
//     fn set_member_b(&mut self, member_b: &str) {
//         self.member_b = Some(member_b.to_string());
//     }
//
//     pub fn list_member(mut self, list_member: Vec<String>) -> Self {
//         self.set_list_member(list_member);
//         self
//     }
//
//     fn set_list_member(&mut self, list_member: Vec<String>) {
//         self.list_member = Some(list_member);
//     }
//
//     pub fn map_member(mut self, map: IndexMap<String, String>) -> Self {
//         self.set_map_member(map);
//         self
//     }
//
//     fn set_map_member(&mut self, map_member: IndexMap<String, String>) {
//         self.map_member = Some(map_member);
//     }
// }
//
// impl Deserializable for SerializeMeBuilder {
//     fn schema() -> &'static Schema<'static> {
//         &SCHEMA
//     }
//
//     fn deserialize_member<D: Deserializer>(
//         &mut self,
//         member_schema: &Schema,
//         member_deserializer: &mut D,
//     ) -> Result<(), D::Error> {
//         match member_schema.member_index {
//             Some(0) => self.set_member_a(member_deserializer.read_string(&MEMBER_A)?),
//             Some(1) => self.set_member_b(member_deserializer.read_string(&MEMBER_B)?),
//             _ => panic!("EEK!"),
//         };
//         Ok(())
//     }
// }
//
// impl ShapeBuilder<SerializeMe> for SerializeMeBuilder {
//     fn build(self) -> SerializeMe {
//         SerializeMe {
//             member_a: self.member_a.expect("Could not find member_a"),
//             member_b: self.member_b.expect("Could not find member_b"),
//             list_member: self.list_member.expect("Could not find list_member"),
//             map_member: self.map_member.expect("Could not find map_member"),
//         }
//     }
// }
