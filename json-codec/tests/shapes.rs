use smithy4rs_core::schema::{prelude, Schema};
use smithy4rs_core::schema::shapes::ShapeId;
use smithy4rs_core::serde::de::{Deserializable, Deserializer, ShapeBuilder};
use smithy4rs_core::serde::se::{Serialize, Serializer, StructSerializer};
use smithy4rs_core::{lazy_member_schema, traits};
use std::sync::LazyLock;
use indexmap::IndexMap;

static NESTED: LazyLock<Schema> = LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("com.example#Nested"))
        .put_member("c", &prelude::STRING, traits![])
        .build()
});
lazy_member_schema!(MEMBER_C, NESTED, "c");

static MAP: LazyLock<Schema> = LazyLock::new(|| {
    Schema::map_builder(ShapeId::from("com.example#Map"))
        .put_member("key", &prelude::STRING, traits![])
        .put_member("value", &NESTED, traits![])
        .build()
});
lazy_member_schema!(MAP_KEY, MAP, "key");
lazy_member_schema!(MAP_VALUE, MAP, "value");

pub static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("com.example#Shape"))
        .put_member("a", &prelude::STRING, traits![])
        .put_member("b", &prelude::STRING, traits![])
        .put_member("nested", &NESTED, traits![])
        .put_member("map", &MAP, traits![])
        .build()
});
lazy_member_schema!(MEMBER_A, SCHEMA, "a");
lazy_member_schema!(MEMBER_B, SCHEMA, "b");
lazy_member_schema!(MEMBER_NESTED, SCHEMA, "nested");
lazy_member_schema!(MEMBER_MAP_NESTED, SCHEMA, "map");

//#[derive(SerializableStruct)]
//#[schema(SCHEMA)]
pub(crate) struct SerializeMe {
    pub member_a: String,
    pub member_b: String,
    pub nested: Nested,
    pub map_nested: IndexMap<String, Nested>
}

impl SerializeMe {
    #[allow(dead_code)]
    pub const fn builder() -> SerializeMeBuilder {
        SerializeMeBuilder::new()
    }
}

impl Serialize for SerializeMe {
    fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        let mut struct_writer = serializer.write_struct(&SCHEMA, 4)?;
        struct_writer.serialize_member(&MEMBER_A, &self.member_a)?;
        struct_writer.serialize_member(&MEMBER_B, &self.member_b)?;
        struct_writer.serialize_member(&MEMBER_NESTED, &self.nested)?;
       // struct_writer.serialize_member(&MEMBER_MAP_NESTED, &self.map_nested)?;
        struct_writer.end(schema)
    }
}

#[derive(Clone)]
pub struct Nested {
    pub member_c: String,
}
impl Nested {
    pub const fn builder() -> NestedBuilder {
        NestedBuilder::new()
    }
}

impl Serialize for Nested {
    fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        let mut struct_writer = serializer.write_struct(&NESTED, 1)?;
        struct_writer.serialize_member(&MEMBER_C, &self.member_c)?;
        struct_writer.end(schema)
    }
}

pub struct NestedBuilder {
    pub member_c: Option<String>,
}

impl NestedBuilder {
    pub const fn new() -> Self {
        NestedBuilder { member_c: None }
    }

    pub fn member_c(mut self, member_c: &str) -> NestedBuilder {
        self.set_member_c(member_c);
        self
    }

    fn set_member_c(&mut self, member_c: &str) {
        self.member_c = Some(member_c.to_string());
    }
}

impl Deserializable for NestedBuilder {
    fn schema() -> &'static Schema<'static> {
        &NESTED
    }

    fn deserialize_member<D: Deserializer>(
        &mut self,
        member_schema: &Schema,
        member_deserializer: &mut D,
    ) -> Result<(), D::Error> {
        match member_schema.member_index {
            // TODO: Should these raise result?
            Some(0) => self.set_member_c(member_deserializer.read_string(&MEMBER_C)?),
            // TODO: Throw real error?
            _ => panic!("Expected member index"),
        };
        Ok(())
    }
}

impl ShapeBuilder<Nested> for NestedBuilder {
    fn build(self) -> Nested {
        Nested {
            member_c: self.member_c.expect("member_c is set"),
        }
    }
}

#[derive(Clone)]
pub struct SerializeMeBuilder {
    pub member_a: Option<String>,
    pub member_b: Option<String>,
    pub nested: Option<Nested>,
    pub map_nested: Option<IndexMap<String, Nested>>
}
impl SerializeMeBuilder {
    pub const fn new() -> SerializeMeBuilder {
        SerializeMeBuilder {
            member_a: None,
            member_b: None,
            nested: None,
            map_nested: None
        }
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

    pub fn nested(mut self, nested: Nested) -> SerializeMeBuilder {
        self.set_nested(nested);
        self
    }

    fn set_nested(&mut self, nested: Nested) {
        self.nested = Some(nested);
    }

    pub fn map_nested(mut self, value: IndexMap<String, Nested>) -> SerializeMeBuilder {
        self.set_map_nested(value);
        self
    }

    fn set_map_nested(&mut self, value: IndexMap<String, Nested>) {
        self.map_nested = Some(value);
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
            // Should these raise result?
            Some(0) => self.set_member_a(member_deserializer.read_string(&MEMBER_A)?),
            Some(1) => self.set_member_b(member_deserializer.read_string(&MEMBER_B)?),
            Some(2) => self.set_nested(Nested::builder().deserialize(member_deserializer)?.build()),
            Some(_) => panic!("Unexpected member: {}", member_schema.id.name),
            // TODO: Throw real error?
            _ => panic!(
                "Expected member index, but none found for member {}",
                member_schema.id.name
            ),
        };

        Ok(())
    }
}

impl ShapeBuilder<SerializeMe> for SerializeMeBuilder {
    fn build(self) -> SerializeMe {
        SerializeMe {
            member_a: self.member_a.expect("Could not find member_a"),
            member_b: self.member_b.expect("Could not find member_b"),
            nested: self.nested.expect("Could not find nested"),
            map_nested: self.map_nested.expect("Could not find map_nested")
        }
    }
}
