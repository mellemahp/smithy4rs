// // #![allow(dead_code)]
// //
// // use crate::schema::documents::Document;
// // use crate::schema::{Schema, SchemaRef};
// // use bigdecimal::BigDecimal;
// // use bytebuffer::ByteBuffer;
// // use num_bigint::BigInt;
// // use std::error::Error;
// // use std::hash::Hash;
// // use std::io::Read;
// // use std::time::Instant;
// // use indexmap::IndexMap;
// // use crate::serde::builders::ShapeBuilder;
// // use crate::serde::se::{Serializer};
//
//
//
// /// A shape that can be deserialized with a schema.
// pub(crate) trait DeserializeWithSchema {
//     fn deserialize_with_schema<D: Deserializer>(schema: &SchemaRef, deserializer: D) -> Self;
// }
//
//
// use std::fmt;
// use bigdecimal::BigDecimal;
// use num_bigint::BigInt;
// use serde::de::{EnumAccess, Error, MapAccess, SeqAccess, Unexpected};
// // TODO: Get member by schema?
// use crate::schema::{SchemaRef, SchemaShape};
// use crate::serde::se::Serialize;
//
// /// Smithy Deserializer
// ///
// /// NOTE: Based on `serde::Deserializer` to try to allow for compatibility.
// pub trait Deserializer<'de>: Sized {
//     /// The error type that can be returned if some error occurs during
//     /// deserialization.
//     type Error: Error;
//
//     /// Hint that the `Deserialize` type is expecting a `bool` value.
//     fn deserialize_bool<V: Visitor<'de>>(self, schema: &SchemaRef, visitor: V) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a `byte` value.
//     fn deserialize_byte<V: Visitor<'de>>(self, schema: &SchemaRef, visitor: V) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a `short` value.
//     fn deserialize_short<V: Visitor<'de>>(self, schema: &SchemaRef, visitor: V) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a `integer` value.
//     fn deserialize_integer<V: Visitor<'de>>(self, schema: &SchemaRef, visitor: V) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a `long` value.
//     fn deserialize_long<V: Visitor<'de>>(self, schema: &SchemaRef, visitor: V) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a `float` value.
//     fn deserialize_float<V: Visitor<'de>>(self, schema: &SchemaRef, visitor: V) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a `double` value.
//     fn deserialize_double<V: Visitor<'de>>(self, schema: &SchemaRef, visitor: V) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a `BigInt` value.
//     fn deserialize_big_integer<V: Visitor<'de>>(self, schema: &SchemaRef, value: &BigInt) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a `BigDecimal` value.
//     fn deserialize_big_decimal<V: Visitor<'de>>(
//         self,
//         schema: &SchemaRef,
//         value: &BigDecimal,
//     ) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a `String` value.
//     fn deserialize_string<V: Visitor<'de>>(self, schema: &SchemaRef, visitor: V) -> Result<V::Value, Self::Error>;
//
//     // TODO: How to handle data streams?
//     /// Hint that the `Deserialize` type is expecting a `blob` value.
//     fn deserialize_blob<V: Visitor<'de>>(self, schema: &SchemaRef, visitor: V) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a `time` value.
//     fn deserialize_timestamp<V: Visitor<'de>>(self, schema: &SchemaRef, visitor: V) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a `document` value.
//     fn deserialize_document<V: Visitor<'de>>(self, schema: &SchemaRef, visitor: V) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a map of key-value pairs.
//     fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
//
//     /// Hint that the `Deserialize` type is expecting a struct with a particular
//     /// name and fields.
//     fn deserialize_struct<V: Visitor<'de>>(
//         self,
//         schema: &SchemaRef,
//         visitor: V
//     ) -> Result<V::Value, Self::Error>;
// }
//
// pub trait Visitor<'de>: Sized {
//     /// The value produced by this visitor.
//     type Value;
//
//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result;
//
//     fn visit_member<D: Deserializer>(
//         self,
//         member_schema: &SchemaRef,
//         deserializer: D,
//     ) -> Result<(), D::Error> {
//         Err(Error::invalid_type(Unexpected::StructVariant, &format!("{:?}", member_schema)))
//     }
//
//     /// The input contains a boolean.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_boolean<E>(self, v: bool) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         Err(Error::invalid_type(Unexpected::Bool(v), &self))
//     }
//
//     /// The input contains an `i8`.
//     ///
//     /// The default implementation forwards to [`visit_i64`].
//     ///
//     /// [`visit_i64`]: #method.visit_i64
//     fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_i64(v as i64)
//     }
//
//     /// The input contains an `i16`.
//     ///
//     /// The default implementation forwards to [`visit_i64`].
//     ///
//     /// [`visit_i64`]: #method.visit_i64
//     fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_i64(v as i64)
//     }
//
//     /// The input contains an `i32`.
//     ///
//     /// The default implementation forwards to [`visit_i64`].
//     ///
//     /// [`visit_i64`]: #method.visit_i64
//     fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_i64(v as i64)
//     }
//
//     /// The input contains an `i64`.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         Err(Error::invalid_type(Unexpected::Signed(v), &self))
//     }
//
//     /// The input contains a `i128`.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         let mut buf = [0u8; 58];
//         let mut writer = crate::format::Buf::new(&mut buf);
//         fmt::Write::write_fmt(&mut writer, format_args!("integer `{}` as i128", v)).unwrap();
//         Err(Error::invalid_type(
//             Unexpected::Other(writer.as_str()),
//             &self,
//         ))
//     }
//
//     /// The input contains a `u8`.
//     ///
//     /// The default implementation forwards to [`visit_u64`].
//     ///
//     /// [`visit_u64`]: #method.visit_u64
//     fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_u64(v as u64)
//     }
//
//     /// The input contains a `u16`.
//     ///
//     /// The default implementation forwards to [`visit_u64`].
//     ///
//     /// [`visit_u64`]: #method.visit_u64
//     fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_u64(v as u64)
//     }
//
//     /// The input contains a `u32`.
//     ///
//     /// The default implementation forwards to [`visit_u64`].
//     ///
//     /// [`visit_u64`]: #method.visit_u64
//     fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_u64(v as u64)
//     }
//
//     /// The input contains a `u64`.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         Err(Error::invalid_type(Unexpected::Unsigned(v), &self))
//     }
//
//     /// The input contains a `u128`.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         let mut buf = [0u8; 57];
//         let mut writer = crate::Buf::new(&mut buf);
//         fmt::Write::write_fmt(&mut writer, format_args!("integer `{}` as u128", v)).unwrap();
//         Err(Error::invalid_type(
//             Unexpected::Other(writer.as_str()),
//             &self,
//         ))
//     }
//
//     /// The input contains an `f32`.
//     ///
//     /// The default implementation forwards to [`visit_f64`].
//     ///
//     /// [`visit_f64`]: #method.visit_f64
//     fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_f64(v as f64)
//     }
//
//     /// The input contains an `f64`.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         Err(Error::invalid_type(Unexpected::Float(v), &self))
//     }
//
//     /// The input contains a `char`.
//     ///
//     /// The default implementation forwards to [`visit_str`] as a one-character
//     /// string.
//     ///
//     /// [`visit_str`]: #method.visit_str
//     #[inline]
//     fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_str(v.encode_utf8(&mut [0u8; 4]))
//     }
//
//     /// The input contains a string. The lifetime of the string is ephemeral and
//     /// it may be destroyed after this method returns.
//     ///
//     /// This method allows the `Deserializer` to avoid a copy by retaining
//     /// ownership of any buffered data. `Deserialize` implementations that do
//     /// not benefit from taking ownership of `String` data should indicate that
//     /// to the deserializer by using `Deserializer::deserialize_str` rather than
//     /// `Deserializer::deserialize_string`.
//     ///
//     /// It is never correct to implement `visit_string` without implementing
//     /// `visit_str`. Implement neither, both, or just `visit_str`.
//     fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         Err(Error::invalid_type(Unexpected::Str(v), &self))
//     }
//
//     /// The input contains a string that lives at least as long as the
//     /// `Deserializer`.
//     ///
//     /// This enables zero-copy deserialization of strings in some formats. For
//     /// example JSON input containing the JSON string `"borrowed"` can be
//     /// deserialized with zero copying into a `&'a str` as long as the input
//     /// data outlives `'a`.
//     ///
//     /// The default implementation forwards to `visit_str`.
//     #[inline]
//     fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_str(v)
//     }
//
//     /// The input contains a string and ownership of the string is being given
//     /// to the `Visitor`.
//     ///
//     /// This method allows the `Visitor` to avoid a copy by taking ownership of
//     /// a string created by the `Deserializer`. `Deserialize` implementations
//     /// that benefit from taking ownership of `String` data should indicate that
//     /// to the deserializer by using `Deserializer::deserialize_string` rather
//     /// than `Deserializer::deserialize_str`, although not every deserializer
//     /// will honor such a request.
//     ///
//     /// It is never correct to implement `visit_string` without implementing
//     /// `visit_str`. Implement neither, both, or just `visit_str`.
//     ///
//     /// The default implementation forwards to `visit_str` and then drops the
//     /// `String`.
//     #[inline]
//     #[cfg(any(feature = "std", feature = "alloc"))]
//     #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
//     fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_str(&v)
//     }
//
//     /// The input contains a byte array. The lifetime of the byte array is
//     /// ephemeral and it may be destroyed after this method returns.
//     ///
//     /// This method allows the `Deserializer` to avoid a copy by retaining
//     /// ownership of any buffered data. `Deserialize` implementations that do
//     /// not benefit from taking ownership of `Vec<u8>` data should indicate that
//     /// to the deserializer by using `Deserializer::deserialize_bytes` rather
//     /// than `Deserializer::deserialize_byte_buf`.
//     ///
//     /// It is never correct to implement `visit_byte_buf` without implementing
//     /// `visit_bytes`. Implement neither, both, or just `visit_bytes`.
//     fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         Err(Error::invalid_type(Unexpected::Bytes(v), &self))
//     }
//
//     /// The input contains a byte array that lives at least as long as the
//     /// `Deserializer`.
//     ///
//     /// This enables zero-copy deserialization of bytes in some formats. For
//     /// example Postcard data containing bytes can be deserialized with zero
//     /// copying into a `&'a [u8]` as long as the input data outlives `'a`.
//     ///
//     /// The default implementation forwards to `visit_bytes`.
//     #[inline]
//     fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_bytes(v)
//     }
//
//     /// The input contains a byte array and ownership of the byte array is being
//     /// given to the `Visitor`.
//     ///
//     /// This method allows the `Visitor` to avoid a copy by taking ownership of
//     /// a byte buffer created by the `Deserializer`. `Deserialize`
//     /// implementations that benefit from taking ownership of `Vec<u8>` data
//     /// should indicate that to the deserializer by using
//     /// `Deserializer::deserialize_byte_buf` rather than
//     /// `Deserializer::deserialize_bytes`, although not every deserializer will
//     /// honor such a request.
//     ///
//     /// It is never correct to implement `visit_byte_buf` without implementing
//     /// `visit_bytes`. Implement neither, both, or just `visit_bytes`.
//     ///
//     /// The default implementation forwards to `visit_bytes` and then drops the
//     /// `Vec<u8>`.
//     #[cfg(any(feature = "std", feature = "alloc"))]
//     #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
//     fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         self.visit_bytes(&v)
//     }
//
//     /// The input contains an optional that is absent.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_none<E>(self) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         Err(Error::invalid_type(Unexpected::Option, &self))
//     }
//
//     /// The input contains an optional that is present.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let _ = deserializer;
//         Err(Error::invalid_type(Unexpected::Option, &self))
//     }
//
//     /// The input contains a unit `()`.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_unit<E>(self) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         Err(Error::invalid_type(Unexpected::Unit, &self))
//     }
//
//     /// The input contains a newtype struct.
//     ///
//     /// The content of the newtype struct may be read from the given
//     /// `Deserializer`.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let _ = deserializer;
//         Err(Error::invalid_type(Unexpected::NewtypeStruct, &self))
//     }
//
//     /// The input contains a sequence of elements.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
//     where
//         A: SeqAccess<'de>,
//     {
//         let _ = seq;
//         Err(Error::invalid_type(Unexpected::Seq, &self))
//     }
//
//     /// The input contains a key-value map.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
//     where
//         A: MapAccess<'de>,
//     {
//         let _ = map;
//         Err(Error::invalid_type(Unexpected::Map, &self))
//     }
//
//     /// The input contains an enum.
//     ///
//     /// The default implementation fails with a type error.
//     fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
//     where
//         A: EnumAccess<'de>,
//     {
//         let _ = data;
//         Err(Error::invalid_type(Unexpected::Enum, &self))
//     }
//
//     // Used when deserializing a flattened Option field. Not public API.
//     #[doc(hidden)]
//     fn __private_visit_untagged_option<D>(self, _: D) -> Result<Self::Value, ()>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         Err(())
//     }
// }
