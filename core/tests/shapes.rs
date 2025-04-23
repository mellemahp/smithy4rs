#![allow(dead_code)]

use std::sync::LazyLock;
use smithy4rs_core::lazy_member_schema;
use smithy4rs_core::schema::{prelude, Schema};
use smithy4rs_core::serde::de::{Deserializable, Deserializer, ShapeBuilder};
use smithy4rs_core::serde::se::{Serializable, SerializableStruct, Serializer};
use smithy4rs_core::serde::serializers::ListItemConsumer;
use smithy4rs_core::shapes::ShapeId;

pub(crate) static LIST_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::list_builder(ShapeId::from("com.example#MyList"))
        .put_member("member", &*prelude::STRING)
        .build()
});

pub(crate) static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("com.example#Shape"))
        .put_member("a", &*prelude::STRING)
        .put_member("b", &*prelude::STRING)
        .put_member("list", &*LIST_SCHEMA)
        .build()
});
pub(crate) lazy_member_schema!(MEMBER_A, SCHEMA, "a");
pub(crate) lazy_member_schema!(MEMBER_B, SCHEMA, "b");
pub(crate) lazy_member_schema!(MEMBER_LIST, SCHEMA, "list");

//#[derive(SerializableStruct)]
//#[schema(SCHEMA)]
pub(crate) struct SerializeMe {
    pub member_a: String,
    pub member_b: String,
    pub list_member: Vec<String>,
}

impl SerializeMe {
    pub const fn builder() -> SerializeMeBuilder {
        SerializeMeBuilder::new()
    }
}

impl SerializableStruct for SerializeMe {
    fn schema() -> &'static Schema<'static> {
        &*SCHEMA
    }

    fn serialize_members<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.write_string(&*MEMBER_A, self.member_a)?;
        serializer.write_string(&*MEMBER_B, self.member_b)?;
        serializer.write_list(&*MEMBER_LIST, self.list_member.len(), self.list_member, ListMemberSerializer {})
    }
}

struct ListMemberSerializer {}
impl ListItemConsumer<Vec<String>> for ListMemberSerializer {
    fn consume<S: Serializer>(item: String, serializer: &mut S) -> Result<(), S::Error> {
        serializer.write_string(&*prelude::STRING, item)
    }
}

impl Serializable for SerializeMe {
    fn serialize<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.write_struct(&*SCHEMA, self)
    }
}

pub struct SerializeMeBuilder {
    pub member_a: Option<String>,
    pub member_b: Option<String>,
    pub list_member: Option<Vec<String>>
}

// TODO: Add builder derive macro?
impl SerializeMeBuilder {
    pub const fn new() -> SerializeMeBuilder {
        SerializeMeBuilder{ member_a: None, member_b: None, list_member: None }
    }

    pub fn member_a(mut self, member_a: &str) -> SerializeMeBuilder {
        self.set_member_a(member_a);
        self
    }

    fn set_member_a(&mut self, member_a: &str) {
        self.member_a = Some(member_a.to_string());
    }

    pub fn member_b(mut self, member_b: &str) -> SerializeMeBuilder {
        self.set_member_b(member_b);
        self
    }

    fn set_member_b(&mut self, member_b: &str) {
        self.member_b = Some(member_b.to_string());
    }

    pub fn list_member(mut self, list_member: Vec<String>) -> SerializeMeBuilder {
        self.set_list_member(list_member);
        self
    }

    fn set_list_member(&mut self, list_member: Vec<String>) {
        self.list_member = Some(list_member);
    }
}

impl Deserializable for SerializeMeBuilder {
    fn schema() -> &'static Schema<'static> {
        &*SCHEMA
    }

    fn deserialize_member<D: Deserializer>(&mut self, member_schema: &Schema, member_deserializer: &mut D) -> Result<(), D::Error> {
        match member_schema.member_index{
            Some(0) => self.set_member_a(member_deserializer.read_string(&*MEMBER_A)?),
            Some(1) => self.set_member_b(member_deserializer.read_string(&*MEMBER_B)?),
            _ => panic!("EEK!")
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
        }
    }
}