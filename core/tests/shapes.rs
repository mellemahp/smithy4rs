#![allow(dead_code)]

use std::collections::HashMap;
use smithy4rs_core::schema::shapes::ShapeId;
use smithy4rs_core::schema::{Schema, prelude};
use smithy4rs_core::serde::de::{Deserializable, Deserializer, ShapeBuilder};
use smithy4rs_core::serde::se::{FmtSerializer, Serializable, SerializableStruct, Serializer};
use smithy4rs_core::serde::serializers::{ListItemConsumer, MapEntryConsumer};
use smithy4rs_core::{lazy_member_schema, traits};
use std::fmt::Display;
use std::sync::LazyLock;
use indexmap::IndexMap;

pub static LIST_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::list_builder(ShapeId::from("com.example#MyList"))
        .put_member("member", &prelude::STRING, traits![])
        .build()
});

pub static MAP_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::map_builder(ShapeId::from("com.example#MyMap"))
        .put_member("key", &prelude::STRING, traits![])
        .put_member("value", &prelude::STRING, traits![])
        .build()
});

pub static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("com.example#Shape"))
        .put_member("a", &prelude::STRING, traits![])
        .put_member("b", &prelude::STRING, traits![])
        .put_member("list", &LIST_SCHEMA, traits![])
        .put_member("map", &MAP_SCHEMA, traits![])
        .build()
});
lazy_member_schema!(MEMBER_A, SCHEMA, "a");
lazy_member_schema!(MEMBER_B, SCHEMA, "b");
lazy_member_schema!(MEMBER_LIST, SCHEMA, "list");
lazy_member_schema!(MEMBER_MAP, SCHEMA, "map");

//#[derive(SerializableStruct)]
//#[schema(SCHEMA)]
pub(crate) struct SerializeMe {
    pub member_a: String,
    pub member_b: String,
    pub list_member: Vec<String>,
    pub map_member: IndexMap<String, String>,
}

impl SerializeMe {
    pub const fn builder() -> SerializeMeBuilder {
        SerializeMeBuilder::new()
    }
}

impl SerializableStruct for SerializeMe {
    fn schema(&self) -> &'static Schema<'static> {
        &SCHEMA
    }

    fn serialize_members<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.write_string(&MEMBER_A, &self.member_a)?;
        serializer.write_string(&MEMBER_B, &self.member_b)?;
        serializer.write_list(
            &MEMBER_LIST,
            &mut self.list_member.iter(),
            ListMemberSerializer,
        )?;
        serializer.write_map(
            &MEMBER_MAP,
            &mut self.map_member.iter(),
            MapMemberSerializer
        )?;
        Ok(())
    }
}

impl Display for SerializeMe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = FmtSerializer::new();
        match self.serialize(&mut output) {
            Ok(_) => write!(f, "{}", output.string),
            Err(_e) => Err(std::fmt::Error {}),
        }
    }
}

struct ListMemberSerializer;
impl ListItemConsumer<&String> for ListMemberSerializer {
    fn write_item<S: Serializer>(item: &String, serializer: &mut S) -> Result<(), S::Error> {
        serializer.write_string(&prelude::STRING, item)
    }
}

struct MapMemberSerializer;
impl MapEntryConsumer<&String, &String> for MapMemberSerializer {
    fn write_key<S: Serializer>(key: &String, serializer: &mut S) -> Result<(), S::Error> {
        serializer.write_string(&prelude::STRING, &key)
    }

    fn write_value<S: Serializer>(value: &String, serializer: &mut S) -> Result<(), S::Error> {
        serializer.write_string(&prelude::STRING, &value)
    }
}

impl Serializable for SerializeMe {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.write_struct(&SCHEMA, self)
    }
}

pub struct SerializeMeBuilder {
    pub member_a: Option<String>,
    pub member_b: Option<String>,
    pub list_member: Option<Vec<String>>,
    pub map_member: Option<IndexMap<String, String>>,
}

// TODO: Add builder derive macro?
impl SerializeMeBuilder {
    pub const fn new() -> SerializeMeBuilder {
        SerializeMeBuilder {
            member_a: None,
            member_b: None,
            list_member: None,
            map_member: None,
        }
    }

    pub fn member_a(mut self, member_a: &str) -> Self {
        self.set_member_a(member_a);
        self
    }

    fn set_member_a(&mut self, member_a: &str) {
        self.member_a = Some(member_a.to_string());
    }

    pub fn member_b(mut self, member_b: &str) -> Self {
        self.set_member_b(member_b);
        self
    }

    fn set_member_b(&mut self, member_b: &str) {
        self.member_b = Some(member_b.to_string());
    }

    pub fn list_member(mut self, list_member: Vec<String>) -> Self {
        self.set_list_member(list_member);
        self
    }

    fn set_list_member(&mut self, list_member: Vec<String>) {
        self.list_member = Some(list_member);
    }

    pub fn map_member(mut self, map: IndexMap<String, String>) -> Self {
        self.set_map_member(map);
        self
    }

    fn set_map_member(&mut self, map_member: IndexMap<String, String>) {
        self.map_member = Some(map_member);
    }
}

impl Deserializable for SerializeMeBuilder {
    fn schema() -> &'static Schema<'static> {
        &SCHEMA
    }

    fn deserialize_member<D: Deserializer>(
        &mut self,
        member_schema: &Schema,
        member_deserializer: &mut D,
    ) -> Result<(), D::Error> {
        match member_schema.member_index {
            Some(0) => self.set_member_a(member_deserializer.read_string(&MEMBER_A)?),
            Some(1) => self.set_member_b(member_deserializer.read_string(&MEMBER_B)?),
            _ => panic!("EEK!"),
        };
        Ok(())
    }
}

impl ShapeBuilder<SerializeMe> for SerializeMeBuilder {
    fn build(self) -> SerializeMe {
        SerializeMe {
            member_a: self.member_a.expect("Could not find member_a"),
            member_b: self.member_b.expect("Could not find member_b"),
            list_member: self.list_member.expect("Could not find list_member"),
            map_member: self.map_member.expect("Could not find map_member"),
        }
    }
}
