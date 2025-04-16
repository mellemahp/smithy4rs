#![allow(dead_code)]
#![allow(unused_variables)]

use std::time::Instant;
use json::JsonValue;
use smithy4rs_core::{BigDecimal, BigInt, ByteBuffer};
use smithy4rs_core::documents::Document;
use smithy4rs_core::schema::Schema;
use smithy4rs_core::serde::se::{ListConsumer, MapConsumer, SerializableStruct, Serializer};
use crate::errors::JsonSerdeError;
use crate::get_member_name;

pub struct JsonSerializer {
    pub string: String,
}

impl JsonSerializer {
    pub const fn new() -> Self {
        JsonSerializer { string: String::new() }
    }
}

// TODO: Handle JSON Name trait
impl Serializer for JsonSerializer {
    type Error = JsonSerdeError;

    fn write_struct<T: SerializableStruct>(&mut self, _: &Schema, structure: T) -> Result<(), Self::Error> {
        let mut data = JsonValue::new_object();
        structure.serialize_members(&mut StructSerializer::new(&mut data))?;
        self.string.push_str(data.dump().as_str());
        Ok(())
    }

    fn write_map<T, C: MapConsumer<T>>(&mut self, schema: &Schema, size: usize, map_state: T, consumer: C) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_list<T, C: ListConsumer<T>>(&mut self, schema: &Schema, size: usize, list_state: T, consumer: C) -> Result<(), Self::Error> {
        todo!()
    }


    fn write_boolean(&mut self, _: &Schema, value: bool) -> Result<(), Self::Error> {
        self.string.push_str(json::stringify(value).as_str());
        Ok(())
    }

    fn write_byte(&mut self, _: &Schema, value: u8) -> Result<(), Self::Error> {
        self.string.push_str(json::stringify(value).as_str());
        Ok(())
    }

    fn write_short(&mut self, _: &Schema, value: i16) -> Result<(), Self::Error> {
        self.string.push_str(json::stringify(value).as_str());
        Ok(())
    }

    fn write_integer(&mut self, _: &Schema, value: i32) -> Result<(), Self::Error> {
        self.string.push_str(json::stringify(value).as_str());
        Ok(())
    }

    fn write_long(&mut self, _: &Schema, value: i64) -> Result<(), Self::Error> {
        self.string.push_str(json::stringify(value).as_str());
        Ok(())
    }

    fn write_float(&mut self, _: &Schema, value: f32) -> Result<(), Self::Error> {
        self.string.push_str(json::stringify(value).as_str());
        Ok(())
    }

    fn write_double(&mut self, _: &Schema, value: f64) -> Result<(), Self::Error> {
        self.string.push_str(json::stringify(value).as_str());
        Ok(())
    }

    fn write_big_integer(&mut self, schema: &Schema, value: BigInt) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_big_decimal(&mut self, schema: &Schema, value: BigDecimal) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_string(&mut self, _: &Schema, value: String) -> Result<(), Self::Error> {
        self.string.push_str(json::stringify(value).as_str());
        Ok(())
    }

    fn write_blob(&mut self, schema: &Schema, value: ByteBuffer) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_timestamp(&mut self, schema: &Schema, value: Instant) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_document(&mut self, schema: &Schema, value: Document) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_null(&mut self, _: &Schema) -> Result<(), Self::Error> {
        self.string.push_str(JsonValue::Null.dump().as_str());
        Ok(())
    }

    fn flush(&self) -> Result<(), Self::Error> {
        // Do nothing
        Ok(())
    }
}

struct StructSerializer<'s> {
    parent: &'s mut JsonValue,
}

impl <'a> StructSerializer<'a> {
    const fn new(parent: &'a mut JsonValue) -> Self {
        StructSerializer { parent }
    }
}

impl Serializer for StructSerializer<'_> {
    type Error = JsonSerdeError;

    fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: T) -> Result<(), Self::Error> {
        let mut data = JsonValue::new_object();
        structure.serialize_members(&mut StructSerializer::new(&mut data))?;
        self.parent[get_member_name(schema)] = data;
        Ok(())
    }

    fn write_map<T, C: MapConsumer<T>>(&mut self, schema: &Schema, size: usize, map_state: T, consumer: C) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_list<T, C: ListConsumer<T>>(&mut self, schema: &Schema, size: usize, list_state: T, consumer: C) -> Result<(), Self::Error> {
        todo!()
    }


    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), Self::Error> {
        self.parent[get_member_name(schema)] = json::from(value);
        Ok(())
    }

    fn write_byte(&mut self, schema: &Schema, value: u8) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), Self::Error> {
        self.parent[get_member_name(schema)] = json::from(value);
        Ok(())
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), Self::Error> {
        self.parent[get_member_name(schema)] = json::from(value);
        Ok(())
    }

    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), Self::Error> {
        self.parent[get_member_name(schema)] = json::from(value);
        Ok(())
    }

    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), Self::Error> {
        self.parent[get_member_name(schema)] = json::from(value);
        Ok(())
    }

    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), Self::Error> {
        self.parent[get_member_name(schema)] = json::from(value);
        Ok(())
    }

    fn write_big_integer(&mut self, schema: &Schema, value: BigInt) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_big_decimal(&mut self, schema: &Schema, value: BigDecimal) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_string(&mut self, schema: &Schema, value: String) -> Result<(), Self::Error> {
        self.parent[get_member_name(schema)] = json::from(value);
        Ok(())
    }

    fn write_blob(&mut self, schema: &Schema, value: ByteBuffer) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_timestamp(&mut self, schema: &Schema, value: Instant) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_document(&mut self, schema: &Schema, value: Document) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_null(&mut self, schema: &Schema) -> Result<(), Self::Error> {
        self.parent[get_member_name(schema)] = JsonValue::Null;
        Ok(())
    }

    fn flush(&self) -> Result<(), Self::Error> {
        /* Do nothing on flush */
        Ok(())
    }
}
