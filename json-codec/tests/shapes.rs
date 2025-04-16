use std::sync::LazyLock;
use smithy4rs_core::schema::{prelude, Schema};
use smithy4rs_core::serde::de::{Deserializable, Deserializer, ShapeBuilder};
use smithy4rs_core::serde::se::{Serializable, SerializableStruct, Serializer};
use smithy4rs_core::shapes::ShapeId;

static NESTED: LazyLock<Schema> = LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("com.example#Nested"))
        .put_member("c", &*prelude::STRING)
        .build()
});
static MEMBER_C: LazyLock<&Schema> = LazyLock::new(|| {
    NESTED.expect_member("c")
});

static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("com.example#Shape"))
        .put_member("a", &*prelude::STRING)
        .put_member("b", &*prelude::STRING)
        .put_member("nested", &*NESTED)
        .build()
});
static MEMBER_A: LazyLock<&Schema> = LazyLock::new(|| {
    SCHEMA.expect_member("a")
});
static MEMBER_B: LazyLock<&Schema> = LazyLock::new(|| {
    SCHEMA.expect_member("b")
});
static MEMBER_NESTED: LazyLock<&Schema> = LazyLock::new(|| {
    SCHEMA.expect_member("nested")
});

//#[derive(SerializableStruct)]
//#[schema(SCHEMA)]
pub(crate) struct SerializeMe {
    pub member_a: String,
    pub member_b: String,
    pub nested: Nested
}

impl SerializeMe {
    pub const fn builder() -> SerializeMeBuilder {
        SerializeMeBuilder::new()
    }
}

impl Serializable for SerializeMe {
    fn serialize<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
        SerializableStruct::serialize(self, serializer)
    }
}

impl SerializableStruct for SerializeMe {
    fn schema() -> &'static Schema<'static> {
        &*SCHEMA
    }

    fn serialize_members<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.write_string(&*MEMBER_A, self.member_a)?;
        serializer.write_string(&*MEMBER_B, self.member_b)?;
        serializer.write_struct(&*MEMBER_NESTED, self.nested)?;
        Ok(())
    }
}

pub(crate) struct Nested {
    pub member_c: String,
}
impl Nested {
    pub const fn builder() -> NestedBuilder {
        NestedBuilder::new()
    }
}

impl SerializableStruct for Nested {
    fn schema() -> &'static Schema<'static> {
        &*NESTED
    }

    fn serialize_members<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.write_string(&*MEMBER_C, self.member_c)?;
        Ok(())
    }
}

impl Serializable for Nested {
    fn serialize<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
        SerializableStruct::serialize(self, serializer)
    }
}

pub struct NestedBuilder {
    pub member_c: Option<String>
}

impl NestedBuilder {
    pub const fn new() -> Self {
        NestedBuilder { member_c: None }
    }

    pub fn member_c(&mut self, member_c: &str) -> &mut NestedBuilder {
        self.member_c = Some(member_c.to_string());
        self
    }
}

impl Deserializable for NestedBuilder {
    fn schema() -> &'static Schema<'static> {
        &*NESTED
    }

    fn deserialize_member<D: Deserializer>(&mut self, member_schema: &Schema, member_deserializer: &mut D) -> Result<(), D::Error> {
        match member_schema.member_index {
            // TODO: Should these raise result?
            Some(0) => self.member_c(member_deserializer.read_string(&*MEMBER_C)?),
            // TODO: Throw real error?
            _ => panic!("Expected member index")
        };
        Ok(())
    }
}

impl ShapeBuilder<Nested> for NestedBuilder {
    fn build(self) -> Nested {
        Nested {
            member_c: self.member_c.expect("member_c is set")
        }
    }
}

pub struct SerializeMeBuilder {
    pub member_a: Option<String>,
    pub member_b: Option<String>,
    pub nested: Option<Nested>
}
impl SerializeMeBuilder {
    pub const fn new() -> SerializeMeBuilder {
        SerializeMeBuilder{ member_a: None, member_b: None, nested: None }
    }

    pub fn member_a(&mut self, member_a: &str) -> &mut SerializeMeBuilder {
        self.member_a = Some(member_a.to_string());
        self
    }

    pub fn member_b(&mut self, member_b: &str) -> &mut SerializeMeBuilder {
        self.member_b = Some(member_b.to_string());
        self
    }

    pub fn nested(&mut self, nested: Nested) -> &mut SerializeMeBuilder {
        self.nested = Some(nested);
        self
    }
}

impl Deserializable for SerializeMeBuilder {
    fn schema() -> &'static Schema<'static> {
        &*SCHEMA
    }


    fn deserialize_member<D: Deserializer>(&mut self, member_schema: &Schema, member_deserializer: &mut D) -> Result<(), D::Error> {
        match member_schema.member_index {
            // Should these raise result?
            Some(0) => self.member_a(member_deserializer.read_string(&*MEMBER_A)?),
            Some(1) => self.member_b(member_deserializer.read_string(&*MEMBER_B)?),
            Some(2) => self.nested(Nested::builder().deserialize(member_deserializer)?.build()),
            Some(_) => panic!("Unexpected member: {}", member_schema.id.name),
            // TODO: Throw real error?
            _ => panic!("Expected member index, but none found for member {}", member_schema.id.name)
        };

        Ok(())
    }
}

impl ShapeBuilder<SerializeMe> for SerializeMeBuilder {
    fn build(self) -> SerializeMe {
        SerializeMe {
            member_a: self.member_a.expect("Could not find member_a"),
            member_b: self.member_b.expect("Could not find member_b"),
            nested: self.nested.expect("Could not find nested")
        }
    }
}