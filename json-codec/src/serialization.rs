#![allow(dead_code)]
#![allow(unused_variables)]

use crate::errors::JsonSerdeError;
use crate::get_member_name;
use json::JsonValue;
use smithy4rs_core::schema::Schema;
use smithy4rs_core::schema::documents::Document;
use smithy4rs_core::serde::se::{MapSerializer, Serialize, Serializer};
use smithy4rs_core::serde::serializers::{ListSerializer, StructSerializer};
use smithy4rs_core::{BigDecimal, BigInt, ByteBuffer};
use std::fmt::Display;
use std::time::Instant;

// TODO: This implementation should maybe just write to a Sink
//       to allow writing really large objects without saving the whole state.
//       It's fina
// TODO: ADD Settings
pub struct JsonSerializer {
    pub value: Option<JsonValue>,
}

// TODO: Document discriminators
impl JsonSerializer {
    pub fn new() -> JsonSerializer {
        JsonSerializer { value: None }
    }

    pub fn of(value: JsonValue) -> JsonSerializer {
        JsonSerializer { value: Some(value) }
    }

    fn push_value(&mut self, schema: &Schema, value: JsonValue) -> Result<(), JsonSerdeError> {
        let Some(root) = &mut self.value else {
            self.value = Some(value);
            return Ok(());
        };
        match root {
            JsonValue::Object(obj) => obj[get_member_name(schema)] = value,
            JsonValue::Array(arr) => arr.push(value),
            _ => {
                return Err(JsonSerdeError::SerializationError(
                    "Cannot push to non-aggregate type".to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl Display for JsonSerializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let value = match &self {
        //     // TODO: Remove this clone?
        //     JsonSerializer::Root(Some(root)) => json::stringify(root.clone()),
        //     _ => "".to_string(),
        // };
        // write!(f, "{}", value)
        todo!()
    }
}

// TODO: Handle JSON Name trait
impl Serializer for JsonSerializer {
    type Error = JsonSerdeError;
    type Ok = ();
    type SerializeList<'l>
        = JsonAggregateTypeSerializer<'l>
    where
        Self: 'l;

    type SerializeMap<'m>
        = JsonAggregateTypeSerializer<'m>
    where
        Self: 'm;

    type SerializeStruct<'s>
        = JsonAggregateTypeSerializer<'s>
    where
        Self: 's;

    fn write_struct<'a>(
        &'a mut self,
        schema: &'a Schema<'a>,
        size: usize,
    ) -> Result<Self::SerializeStruct<'_>, Self::Error> {
        Ok(JsonAggregateTypeSerializer::new_object(self))
    }

    fn write_map<'a>(
        &'a mut self,
        schema: &'a Schema<'a>,
        len: usize,
    ) -> Result<Self::SerializeMap<'_>, Self::Error> {
        Ok(JsonAggregateTypeSerializer::new_object(self))
    }

    fn write_list<'a>(
        &'a mut self,
        schema: &'a Schema<'a>,
        len: usize,
    ) -> Result<Self::SerializeList<'_>, Self::Error> {
        Ok(JsonAggregateTypeSerializer::new_arr(self))
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
        self.push_value(schema, JsonValue::String(value.clone()))
    }

    fn write_blob(&mut self, schema: &Schema, value: &ByteBuffer) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_timestamp(&mut self, schema: &Schema, value: &Instant) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_document(&mut self, schema: &Schema, value: &Document) -> Result<(), Self::Error> {
        // match &value.schema.shape_type {
        //     ShapeType::Structure | ShapeType::Union => {
        //         let mut data = JsonValue::new_object();
        //         // TODO: make this a setting? Maybe take in a mapper?
        //         data[Self::TYPE_PREFIX] = JsonValue::String(value.schema.id.id.clone());
        //         value.serialize_members(&mut JsonSerializer::of(&mut data))?;
        //         self.push_value(schema, data)
        //     },
        //     _ => todo!() //value.serialize_contents(self)
        todo!()
    }

    fn write_null(&mut self, schema: &Schema) -> Result<(), Self::Error> {
        self.push_value(schema, JsonValue::Null)
    }

    fn skip(&mut self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

pub struct JsonAggregateTypeSerializer<'se> {
    parent: &'se mut JsonSerializer,
    inner: JsonSerializer,
}
impl<'se> JsonAggregateTypeSerializer<'se> {
    fn new_arr(parent: &'se mut JsonSerializer) -> Self {
        Self {
            parent,
            inner: JsonSerializer::of(JsonValue::new_array()),
        }
    }

    fn new_object(parent: &'se mut JsonSerializer) -> Self {
        Self {
            parent,
            inner: JsonSerializer::of(JsonValue::new_object()),
        }
    }

    fn flush(mut self, schema: &Schema) -> Result<(), JsonSerdeError> {
        let Some(root) = self.inner.value else {
            unreachable!("should never be null");
        };
        self.parent.push_value(schema, root)
    }
}
impl ListSerializer for JsonAggregateTypeSerializer<'_> {
    type Error = JsonSerdeError;
    type Ok = ();

    fn serialize_element<T>(
        &mut self,
        element_schema: &Schema,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(element_schema, &mut self.inner)
    }

    fn end(mut self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        self.flush(schema)
    }
}
impl MapSerializer for JsonAggregateTypeSerializer<'_> {
    type Ok = ();
    type Error = JsonSerdeError;

    fn serialize_entry<K, V>(
        &mut self,
        key_schema: &Schema,
        value_schema: &Schema,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        value.serialize(key_schema, &mut self.inner)
    }

    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        self.flush(schema)
    }
}
impl StructSerializer for JsonAggregateTypeSerializer<'_> {
    type Ok = ();
    type Error = JsonSerdeError;

    fn serialize_member<T>(&mut self, member_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(member_schema, &mut self.inner)
    }

    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        self.flush(schema)
    }
}
