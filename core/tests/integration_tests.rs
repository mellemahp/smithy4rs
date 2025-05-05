mod shapes;

use std::collections::HashMap;
use indexmap::IndexMap;
use crate::shapes::SerializeMe;

#[test]
fn test_fmt_serializer() {
    let mut map = IndexMap::new();
    map.insert("a".to_string(), "b".to_string());
    map.insert("c".to_string(), "d".to_string());
    map.insert("e".to_string(), "f".to_string());
    let structure = SerializeMe {
        member_a: "Hello".to_string(),
        member_b: "World".to_string(),
        list_member: vec!["s".to_string(), "e".to_string()],
        map_member: map
    };
    assert_eq!(
        format!("{}", structure),
        "Shape[a=Hello, b=World, list=[s, e], map={a:b, c:d, e:f}]"
    );
}

//
// struct DummyDeserializer<'de> {
//     arr: Vec<&'de str>,
//     field_name: Option<&'de str>,
//     value: Option<&'de str>,
// }
// impl<'de> DummyDeserializer<'de> {
//     fn next_field(&mut self) -> Option<&'de str> {
//         self.arr.pop()
//             .map(|val| val.split_once('=').expect("could not parse"))
//             .map(| (field, value) | {
//                 self.value = Some(value);
//                 self.field_name = Some(field);
//                 field
//             }).or_else(|| {
//                 self.value = None;
//                 self.field_name = None;
//                 None
//             })
//     }
// }

// impl Deserializer for DummyDeserializer<'_> {
//     type Error = ();
//
//     fn read_struct<T>(&mut self, schema: &Schema, state: &mut T, consumer: StructConsumer<T, Self>) -> Result<(), Self::Error> {
//         while self.next_field().is_some() {
//             if let Some(field) = self.field_name {
//                 let member = schema.get_member(field).expect("could not find member");
//                 // TODO: Handle unknown members?
//                 consumer.accept(state, member, self);
//             }
//         }
//         Ok(())
//     }
//
//     fn read_list<T>(&mut self, schema: &Schema, state: T, consumer: ListConsumer<T, Self>) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn read_string_map<T>(schema: &Schema, state: T, consumer: StringMapConsumer<T, Self>) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//
//     fn read_boolean(&mut self, schema: &Schema) -> bool {
//         todo!()
//     }
//
//     fn read_blob(&mut self, schema: &Schema) -> ByteBuffer {
//         todo!()
//     }
//
//     fn read_byte(&mut self, schema: &Schema) -> u8 {
//         todo!()
//     }
//
//     fn read_short(&mut self, schema: &Schema) -> i16 {
//         todo!()
//     }
//
//     fn read_integer(&mut self, schema: &Schema) -> i32 {
//         todo!()
//     }
//
//     fn read_long(&mut self, schema: &Schema) -> i64 {
//         todo!()
//     }
//
//     fn read_float(&mut self, schema: &Schema) -> f32 {
//         todo!()
//     }
//
//     fn read_double(&mut self, schema: &Schema) -> f64 {
//         todo!()
//     }
//
//     fn read_big_integer(&mut self, schema: &Schema) -> BigInt {
//         todo!()
//     }
//
//     fn read_big_decimal(&mut self, schema: &Schema) -> BigDecimal {
//         todo!()
//     }
//
//     fn read_string(&mut self, schema: &Schema) -> &str {
//         self.value.expect("String value expected")
//     }
//
//     fn read_timestamp(&mut self, schema: &Schema) -> Instant {
//         todo!()
//     }
//
//     fn read_document(&mut self, schema: &Schema) -> Document {
//         todo!()
//     }
//
//     fn is_null() -> bool {
//         todo!()
//     }
//
//     fn read_null<T>() {
//         todo!()
//     }
//
//     fn finish(&mut self) -> Result<(), Self::Error> {
//         todo!()
//     }
// }

#[test]
fn test_struct_deserialization() {
    // // Deserialize
    // let input = "a=Hello,b=World".split(",").collect::<Vec<&str>>();
    // let mut deserializer = DummyDeserializer { arr: input, field_name: None, value: None };
    // let mut shapebuilder = SerializeMe::builder();
    // let shape = shapebuilder.deserialize(&mut deserializer).expect("deserialization failed").build();
    // println!("A: {}", shape.member_a);
    // println!("B: {}", shape.member_b);
    //
    // // Serialize again
    // let mut output = FmtSerializer::new();
    // shape.serialize(&mut output).expect("Serialization failed");
    // assert_eq!(output.string, "Shape[a=Hello, b=World]");
    // println!("OUTPUT: {}", output.string);
}
