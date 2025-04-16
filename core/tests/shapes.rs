#![allow(dead_code)]

use std::sync::LazyLock;
use smithy4rs_core::schema::{prelude, Schema};
use smithy4rs_core::serde::de::{Deserializable, Deserializer, ShapeBuilder};
use smithy4rs_core::serde::se::{Serializable, SerializableStruct, Serializer};
use smithy4rs_core::serde::serializers::ListConsumer;
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
pub(crate) static MEMBER_A: LazyLock<&Schema> = LazyLock::new(|| {
    SCHEMA.expect_member("a")
});
pub(crate) static MEMBER_B: LazyLock<&Schema> = LazyLock::new(|| {
    SCHEMA.expect_member("b")
});
pub(crate) static MEMBER_LIST: LazyLock<&Schema> = LazyLock::new(|| {
    SCHEMA.expect_member("list")
});

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

// TODO:
struct ListMemberSerializer {}
impl ListConsumer<Vec<String>> for ListMemberSerializer {
    fn accept<S: Serializer>(self, state: Vec<String>, serializer: &mut S) -> Result<(), S::Error> {
        for item in state.into_iter() {
            serializer.write_string(&*prelude::STRING, item)?;
        }
        Ok(())
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
    pub list_member: Vec<String>
}

// TODO: Add builder derive macro?
impl SerializeMeBuilder {
    pub const fn new() -> SerializeMeBuilder {
        SerializeMeBuilder{ member_a: None, member_b: None, list_member: Vec::new() }
    }

    pub fn member_a(&mut self, member_a: &str) -> &mut SerializeMeBuilder {
        self.member_a = Some(member_a.to_string());
        self
    }

    pub fn member_b(&mut self, member_b: &str) -> &mut SerializeMeBuilder {
        self.member_b = Some(member_b.to_string());
        self
    }

    pub fn add_list_item(&mut self, item: &str) -> &mut SerializeMeBuilder {
        self.list_member.push(item.to_string());
        self
    }
}

impl Deserializable for SerializeMeBuilder {
    fn schema() -> &'static Schema<'static> {
        &*SCHEMA
    }

    fn deserialize_member<D: Deserializer>(&mut self, member_schema: &Schema, member_deserializer: &mut D) -> Result<(), D::Error> {
        match member_schema.member_index{
            Some(0) => self.member_a(member_deserializer.read_string(&*MEMBER_A)?),
            Some(1) => self.member_b(member_deserializer.read_string(&*MEMBER_B)?),
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
            list_member: self.list_member,
        }
    }
}