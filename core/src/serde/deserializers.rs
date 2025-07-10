// #![allow(dead_code)]
//
// use crate::schema::documents::Document;
// use crate::schema::{Schema, SchemaRef};
// use bigdecimal::BigDecimal;
// use bytebuffer::ByteBuffer;
// use num_bigint::BigInt;
// use std::error::Error;
// use std::hash::Hash;
// use std::io::Read;
// use std::time::Instant;
// use indexmap::IndexMap;
// use crate::serde::builders::ShapeBuilder;
// use crate::serde::se::{Serializer};
//
// pub trait Deserialize<'de>: Sized {
//     fn deserialize<'s, D>(schema: &'s SchemaRef, deserializer: D) -> Result<Self, D::Error>
//         where
//             D: Deserializer<'de>;
// }
//
//
// // TODO: Deserialize Shape
//
// // pub trait Deserializable: Sized {
// //     fn schema<'a>() -> &'a SchemaRef;
// //
// //     fn deserialize<D: Deserializer<'de>>(mut self, decoder: &mut D) -> Result<Self, D::Error> {
// //         //decoder.read_struct(Self::schema(), &mut self, Self::deserialize_member)?;
// //         Ok(self)
// //     }
// //
// //     fn deserialize_member<D: Deserializer<'de>>(
// //         &mut self,
// //         member_schema: &Schema,
// //         member_deserializer: &mut D,
// //     ) -> Result<(), D::Error>;
// //
// //     fn error_correction(&mut self) {
// //         todo!()
// //     }
// // }
//
// pub trait MapReader<'de> {
//     /// Must match the `Error` type of our [`Deserializer`] and be able to handle unknown errors.
//     type Error: Error + From<Box<dyn Error>>;
//     fn size_hint(&self) -> Option<usize>;
//     fn read_entry<'k, 'v, K, V>(&mut self, schema: &SchemaRef) -> Result<Option<(K, V)>, Self::Error>
//     where
//         K: Deserialize<'k> + Hash + Eq,
//         V: Deserialize<'v>;
// }
//
// pub trait ListReader<'de> {
//     /// Must match the `Error` type of our [`Deserializer`] and be able to handle unknown errors.
//     type Error: Error + From<Box<dyn Error>>;
//
//     fn size_hint(&self) -> Option<usize>;
//     fn read_item<'i, I>(&mut self, schema: &SchemaRef) -> Result<Option<I>, Self::Error>
//     where I: Deserialize<'i>;
// }
//
// // TODO: datastream?
// // TODO: event stream?
// pub trait Deserializer<'de>: Sized {
//     type Error: Error + From<Box<dyn Error>>;
//
//     type MapReader<'m>: MapReader<'m, Error=Self::Error>;
//     type ListReader<'l>: ListReader<'l, Error=Self::Error>;
//
//     fn read_struct<'a, T>(
//         self,
//         schema: &SchemaRef,
//         builder: impl ShapeBuilder<'a, T>,
//     ) -> Result<(), Self::Error>;
//
//     fn read_list(self, schema: &SchemaRef) -> Result<Self::ListReader<'_>, Self::Error>;
//     fn read_map(self, schema: &SchemaRef) -> Result<Self::MapReader<'_>, Self::Error>;
//
//     fn read_boolean(self, schema: &SchemaRef) -> Result<bool, Self::Error>;
//
//     fn read_blob(self, schema: &SchemaRef) -> Result<ByteBuffer, Self::Error>;
//
//     fn read_byte(self, schema: &SchemaRef) -> Result<i8, Self::Error>;
//
//     fn read_short(self, schema: &SchemaRef) -> Result<i16, Self::Error>;
//
//     fn read_integer(self, schema: &SchemaRef) -> Result<i32, Self::Error>;
//
//     fn read_long(self, schema: &SchemaRef) -> Result<i64, Self::Error>;
//
//     fn read_float(self, schema: &SchemaRef) -> Result<f32, Self::Error>;
//
//     fn read_double(self, schema: &SchemaRef) -> Result<f64, Self::Error>;
//
//     fn read_big_integer(self, schema: &SchemaRef) -> Result<BigInt, Self::Error>;
//
//     fn read_big_decimal(self, schema: &SchemaRef) -> Result<BigDecimal, Self::Error>;
//
//     fn read_string(self, schema: &SchemaRef) -> Result<String, Self::Error>;
//
//     fn read_timestamp(self, schema: &SchemaRef) -> Result<Instant, Self::Error>;
//
//     fn read_document(self, schema: &SchemaRef) -> Result<Document, Self::Error>;
//
//     /// Peek at next value to determine if it is null without consuming
//     fn is_null(&self, schema: &SchemaRef) -> bool;
//
//     ///  Read (skip) the null value. Only makes sense after is_null().
//     fn read_null<T>(&mut self) -> Result<(), Self::Error>;
//
//     // Finish reading all remaining data
//     fn finish(&mut self) -> Result<(), Self::Error>;
// }
// // INTERCEPTING DESERIALIZER?
//
//
// // === Exported Implementations for basic types ===
// impl <'de> Deserialize<'de> for String {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         deserializer.read_string(schema)
//     }
// }
//
// impl <'de> Deserialize<'de> for bool {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         deserializer.read_boolean(schema)
//     }
// }
//
// impl <'de> Deserialize<'de> for i8 {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         deserializer.read_byte(schema)
//     }
// }
//
// impl <'de> Deserialize<'de> for i16 {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         deserializer.read_short(schema)
//     }
// }
//
// impl <'de> Deserialize<'de> for i32 {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         deserializer.read_integer(schema)
//     }
// }
//
// impl <'de> Deserialize<'de> for i64 {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         deserializer.read_long(schema)
//     }
// }
//
// impl <'de> Deserialize<'de> for f32 {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         deserializer.read_float(schema)
//     }
// }
//
// impl <'de> Deserialize<'de> for f64 {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         deserializer.read_double(schema)
//     }
// }
//
// impl <'de> Deserialize<'de> for BigInt {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         deserializer.read_big_integer(schema)
//     }
// }
//
// impl <'de> Deserialize<'de> for BigDecimal {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         deserializer.read_big_decimal(schema)
//     }
// }
//
// impl <'de> Deserialize<'de> for ByteBuffer {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         deserializer.read_blob(schema)
//     }
// }
//
// // TODO: How to handle optional types inside sparse list/map?
// impl <'de, 'k, 'v, 's, K, V> Deserialize<'de> for IndexMap<K, V>
// where
//     K: Deserialize<'k> + Eq + Hash,
//     V: Deserialize<'v>,
//     'k: 'de,
//     'v: 'k + 'de
// {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         let mut map_reader = deserializer.read_map(schema)?;
//         let mut map = IndexMap::with_capacity(map_reader.size_hint().unwrap_or(0));
//         while let Some((key, value)) = map_reader.read_entry(schema)? {
//             map.insert(key, value);
//         }
//         Ok(map)
//     }
// }
//
// impl <'de, I: Deserialize<'de>> Deserialize<'de> for Vec<I> {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         let mut reader = deserializer.read_list(schema)?;
//         let mut vec = Vec::with_capacity(reader.size_hint().unwrap_or(0));
//         while let Some(value) = reader.read_item(schema)? {
//             vec.push(value);
//         }
//         Ok(vec)
//     }
// }
//
// // Struct Ideation
// #[derive(Debug)]
// struct SerializeMe {
//     member_a: String
// }
// struct SerializeMeBuilder {
//     member_a: Option<String>
// }
// impl SerializeMeBuilder {
//     pub fn member_a(mut self, member_a: &str) -> Self {
//         self.set_member_a(member_a.to_string());
//         self
//     }
//
//     fn set_member_a(&mut self, member_a: String) {
//         self.member_a = Some(member_a);
//     }
// }
// impl <'de> ShapeBuilder<'de, SerializeMe> for SerializeMeBuilder {
//     fn new() -> Self {
//         SerializeMeBuilder { member_a: None }
//     }
//
//     // Build, not deserialization is where validation errors are thrown, seperating the two out
//     // Also allows for shape to be constructed from multiple deser sources.
//     fn build(self) -> Result<SerializeMe, String> {
//         Ok(SerializeMe { member_a: self.member_a.unwrap() })
//     }
//
//     fn deserialize_member<D: Deserializer<'de>>(&mut self, member_schema: &SchemaRef, deserializer: D) -> Result<(), D::Error> {
//         match member_schema.as_member().unwrap().index {
//             0 => self.set_member_a(String::deserialize(member_schema, deserializer)?),
//             _ => panic!("EEK!"),
//         };
//         Ok(())
//     }
// }
//
// impl <'de> Deserialize<'de> for SerializeMeBuilder {
//     fn deserialize<D: Deserializer<'de>>(schema: &SchemaRef, deserializer: D) -> Result<Self, D::Error> {
//         // let mut builder = Self::new();
//         // for member in schema.members().values() {
//         //     builder.deserialize_member(member, deserializer)?;
//         // }
//         // Ok(builder)
//         todo!()
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use std::sync::LazyLock;
//     use crate::schema::{prelude, ShapeId};
//     use crate::serde::FmtError;
//     use crate::{lazy_member_schema, traits};
//     use super::*;
//
//     struct DummyMapReader {}
//     impl MapReader<'_> for DummyMapReader {
//         type Error = FmtError;
//
//         fn size_hint(&self) -> Option<usize> {
//             todo!()
//         }
//
//         fn read_entry<'a, K, V>(&mut self, schema: &SchemaRef) -> Result<Option<(K, V)>, Self::Error>
//         where
//             K: Deserialize<'a> + Hash + Eq,
//             V: Deserialize<'a>
//         {
//             todo!()
//         }
//     }
//     struct DummyListReader{}
//     impl ListReader<'_> for DummyListReader {
//         type Error = FmtError;
//         fn size_hint(&self) -> Option<usize> {
//             todo!()
//         }
//
//         fn read_item<'a, I: Deserialize<'a>, D: Deserializer<'a>>(&mut self, schema: &SchemaRef) -> Result<Option<I>, Self::Error> {
//             todo!()
//         }
//     }
//     struct DeserTest {}
//     impl Deserializer<'_> for DeserTest {
//         type Error = FmtError;
//         type MapReader<'m> = DummyMapReader
//         where
//             Self: 'm;
//         type ListReader<'l> = DummyListReader
//         where
//             Self: 'l;
//
//         fn read_struct<T>(&mut self, schema: &SchemaRef, builder: impl ShapeBuilder<T>) -> Result<(), Self::Error> {
//             todo!()
//         }
//
//         fn read_list(&mut self, schema: &SchemaRef) -> Result<Self::ListReader<'_>, Self::Error> {
//             todo!()
//         }
//
//         fn read_map(&mut self, schema: &SchemaRef) -> Result<Self::MapReader<'_>, Self::Error> {
//             todo!()
//         }
//
//         fn read_boolean(&mut self, schema: &SchemaRef) -> Result<bool, Self::Error> {
//             todo!()
//         }
//
//         fn read_blob(&mut self, schema: &SchemaRef) -> Result<ByteBuffer, Self::Error> {
//             todo!()
//         }
//
//         fn read_byte(&mut self, schema: &SchemaRef) -> Result<i8, Self::Error> {
//             todo!()
//         }
//
//         fn read_short(&mut self, schema: &SchemaRef) -> Result<i16, Self::Error> {
//             todo!()
//         }
//
//         fn read_integer(&mut self, schema: &SchemaRef) -> Result<i32, Self::Error> {
//             todo!()
//         }
//
//         fn read_long(&mut self, schema: &SchemaRef) -> Result<i64, Self::Error> {
//             todo!()
//         }
//
//         fn read_float(&mut self, schema: &SchemaRef) -> Result<f32, Self::Error> {
//             todo!()
//         }
//
//         fn read_double(&mut self, schema: &SchemaRef) -> Result<f64, Self::Error> {
//             todo!()
//         }
//
//         fn read_big_integer(&mut self, schema: &SchemaRef) -> Result<BigInt, Self::Error> {
//             todo!()
//         }
//
//         fn read_big_decimal(&mut self, schema: &SchemaRef) -> Result<BigDecimal, Self::Error> {
//             todo!()
//         }
//
//         fn read_string(&mut self, schema: &SchemaRef) -> Result<String, Self::Error> {
//             Ok("testing".to_string())
//         }
//
//         fn read_timestamp(&mut self, schema: &SchemaRef) -> Result<Instant, Self::Error> {
//             todo!()
//         }
//
//         fn read_document(&mut self, schema: &SchemaRef) -> Result<Document, Self::Error> {
//             todo!()
//         }
//
//         fn is_null(&self, schema: &SchemaRef) -> bool {
//             todo!()
//         }
//
//         fn read_null<T>(&mut self) -> Result<(), Self::Error> {
//             todo!()
//         }
//
//         fn finish(&mut self) -> Result<(), Self::Error> {
//             todo!()
//         }
//     }
//
//     pub static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
//         Schema::structure_builder(ShapeId::from("com.example#Shape"))
//             .put_member("a", &prelude::STRING, traits![])
//             .build()
//     });
//     lazy_member_schema!(MEMBER_A, SCHEMA, "a");
//
//     #[test]
//     fn test_builder() {
//         let mut deserializer = DeserTest{};
//         let builder = SerializeMeBuilder::deserialize(&SCHEMA, &mut deserializer).expect("deserialization failed");
//         let output = builder.build().expect("building failed");
//         println!("{:?}", output);
//     }
// }
