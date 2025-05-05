#![allow(dead_code)]
#![allow(unused_variables)]

use std::fmt::Display;
use crate::errors::JsonSerdeError;
use crate::get_member_name;
use json::JsonValue;
use smithy4rs_core::schema::Schema;
use smithy4rs_core::schema::documents::Document;
use smithy4rs_core::serde::se::{ListItemConsumer, MapEntryConsumer, SerializableStruct, Serializer};
use smithy4rs_core::{BigDecimal, BigInt, ByteBuffer};
use std::time::Instant;

// TODO: This implementation should maybe just write to a Sink
//       to allow writing really large objects without saving the whole state.
//       It's fina
pub enum JsonSerializer<'ser> {
    Root(Option<JsonValue>),
    Nested(&'ser mut JsonValue),
}

impl JsonSerializer<'_> {
    pub fn new<'a>() -> JsonSerializer<'a> {
        JsonSerializer::Root(None)
    }

    pub fn of(parent: &mut JsonValue) -> JsonSerializer {
        JsonSerializer::Nested(parent)
    }

    fn push_value(&mut self, schema: &Schema, value: JsonValue) -> Result<(), JsonSerdeError> {
        match self {
            // If no root then current object is the root object
            JsonSerializer::Root(None) => {
                *self = JsonSerializer::Root(Some(value));
                Ok(())
            },
            // Otherwise, if we are still in root node, just push data to that node.
            JsonSerializer::Root(Some(root)) => Self::push_impl(schema, root, value),
            // If we are in a nested node, push to parent node.
            JsonSerializer::Nested(parent) => Self::push_impl(schema, parent, value),
        }
    }
    fn push_impl(schema: &Schema, into: &mut JsonValue, value: JsonValue) -> Result<(), JsonSerdeError> {
        match into {
            JsonValue::Object(parent) => parent[get_member_name(schema)] = value,
            JsonValue::Array(arr) => arr.push(value),
            _ => {
                return Err(JsonSerdeError::SerializationError(
                    "Cannot append to Type".to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl Display for JsonSerializer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match &self {
            // TODO: Remove this clone?
            JsonSerializer::Root(Some(root)) => json::stringify(root.clone()),
            _ => "".to_string(),
        };
        write!(f, "{}", value)
    }
}

// TODO: Handle JSON Name trait
impl Serializer for JsonSerializer<'_> {
    type Error = JsonSerdeError;

    fn write_struct<T: SerializableStruct>(
        &mut self,
        schema: &Schema,
        structure: &T,
    ) -> Result<(), Self::Error> {
        let mut data = JsonValue::new_object();
        structure.serialize_members(&mut JsonSerializer::of(&mut data))?;
        self.push_value(schema, data)
    }

    fn write_map<K, V, C: MapEntryConsumer<K, V>>(
        &mut self,
        schema: &Schema,
        map_state: impl Iterator<Item = (K, V)> + ExactSizeIterator,
        consumer: C,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_map_entry<K, V, C: MapEntryConsumer<K, V>>(&mut self, schema: &Schema, key: K, value: V, consumer: &C) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_list<I, C: ListItemConsumer<I>>(
        &mut self,
        schema: &Schema,
        list_state: impl Iterator<Item = I> + ExactSizeIterator,
        consumer: C,
    ) -> Result<(), Self::Error> {
        let mut list_data = JsonValue::new_array();
        for item in list_state {
            C::write_item(item, &mut JsonSerializer::of(&mut list_data))?
        }
        self.push_value(schema, list_data)
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), Self::Error> {
        self.push_value(schema, json::from(value))
    }

    fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<(), Self::Error> {
        self.push_value(schema, json::from(value))
    }

    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), Self::Error> {
        self.push_value(schema, json::from(value))
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), Self::Error> {
        self.push_value(schema, json::from(value))
    }

    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), Self::Error> {
        self.push_value(schema, json::from(value))
    }

    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), Self::Error> {
        self.push_value(schema, json::from(value))
    }

    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), Self::Error> {
        self.push_value(schema, json::from(value))
    }

    fn write_big_integer(&mut self, schema: &Schema, value: &BigInt) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_big_decimal(
        &mut self,
        schema: &Schema,
        value: &BigDecimal,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_string(&mut self, schema: &Schema, value: &String) -> Result<(), Self::Error> {
        self.push_value(schema, json::from(value.as_str()))
    }

    fn write_blob(&mut self, schema: &Schema, value: &ByteBuffer) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_timestamp(&mut self, schema: &Schema, value: &Instant) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_document(&mut self, schema: &Schema, value: &Document) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_null(&mut self, schema: &Schema) -> Result<(), Self::Error> {
        self.push_value(schema, JsonValue::Null)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}
