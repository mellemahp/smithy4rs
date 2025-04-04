// use std::collections::HashMap;
// use std::io::empty;
// use crate::schema::Schema;
// use crate::shapes::{ShapeId, ShapeType};

pub struct Document {}

// static DOCUMENT_SCHEMA: Schema = Schema {
//     id: ShapeId::from("smithy.api#Document"),
//     shape_type: ShapeType::DOCUMENT,
//     traits: None,
//     members: HashMap::new(),
//     member_target: None,
//     member_index: None
// };
//
// enum DocumentValue {
//     Document(Document),
//     Map(HashMap<String, DocumentValue>),
//     List(Vec<String>),
//     String(String),
//     Integer(i32),
//     Float(f32),
//     Double(f64),
//     Boolean(bool),
//     Null(None),
//     Bytes(Vec<u8>),
//     TimeStamp(u64),
// }
//
// /// Wrapped protocol-agnostic open content
// struct Document<'a> {
//     value: DocumentValue,
//     shape_type: ShapeType,
//     schema: &'a Schema,
// }
//
// impl Document {
//     fn new(value: DocumentValue, schema: Option<&Schema>) -> Document {
//         let schema =  schema.unwrap_or(&DOCUMENT_SCHEMA);
//         match value {
//             DocumentValue::Map(_) => {}
//             DocumentValue::List(_) => {}
//             _ => {}
//         }
//         Document {
//             value,
//             shape_type,
//             schema
//         }
//     }
//
//     fn _is_raw_map(&self, value: DocumentValue::Map) -> bool {
//        // len(value) != 0 and not isinstance(next(iter(value.values())), Document)
//
//     }
//
// }

