use std::sync::LazyLock;
use smithy4rs_core::schema::{prelude, Schema};
use smithy4rs_core::serde::{Deserializer, Serializable, SerializableStruct, Serializer, ShapeBuilder, StructMemberConsumer};
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

impl SerializableStruct for SerializeMe {
    fn schema() -> &'static Schema<'static> {
        &*SCHEMA
    }

    fn serialize_members<S: Serializer>(&self, serializer: &mut S) {
        serializer.write_string(&*MEMBER_A, &self.member_a);
        serializer.write_string(&*MEMBER_B, &self.member_b);
        serializer.write_struct(&*MEMBER_NESTED, &self.nested);
    }
}

impl Serializable for SerializeMe {
    fn serialize<S: Serializer>(&self, serializer: &mut S) {
        serializer.write_struct(&*SCHEMA, self)
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

    fn serialize_members<S: Serializer>(&self, serializer: &mut S) {
        serializer.write_string(&*MEMBER_C, &self.member_c);
    }
}

impl Serializable for Nested {
    fn serialize<S: Serializer>(&self, serializer: &mut S) {
        serializer.write_struct(&*NESTED, self)
    }
}

pub struct NestedBuilder {
    pub member_c: Option<String>
}

impl NestedBuilder {
    pub(super) const fn new() -> Self {
        NestedBuilder { member_c: None }
    }

    pub fn member_c(&mut self, member_c: String) -> &mut NestedBuilder {
        self.member_c = Some(member_c);
        self
    }
}

impl ShapeBuilder<Nested> for NestedBuilder {
    fn schema() -> &'static Schema<'static> {
        &*NESTED
    }

    fn build(self) -> Nested {
        Nested {
            member_c: self.member_c.expect("member_c is set")
        }
    }

    fn deserialize<D: Deserializer>(&mut self, decoder: &mut D) -> &mut Self {
        decoder.read_struct(&*NESTED, self, NestedMemberConsumer {});
        self
    }
}

struct NestedMemberConsumer;
impl <D: Deserializer> StructMemberConsumer<NestedBuilder, D> for NestedMemberConsumer {
    fn accept(&self, state: &mut NestedBuilder, member_schema: &Schema, member_deserializer: &mut D) {
        match member_schema.member_index{
            // TODO: Should these raise result?
            Some(0) => state.member_c(member_deserializer.read_string(&*MEMBER_C).unwrap().to_string()).ignore(),
            // TODO: Throw real error?
            _ => panic!("Expected member index")
        }
    }
}

pub struct SerializeMeBuilder {
    pub member_a: Option<String>,
    pub member_b: Option<String>,
    pub nested: Option<Nested>
}
impl SerializeMeBuilder {
    pub(super) const fn new() -> SerializeMeBuilder {
        SerializeMeBuilder{ member_a: None, member_b: None, nested: None }
    }

    pub fn member_a(&mut self, member_a: String) -> &mut SerializeMeBuilder {
        self.member_a = Some(member_a);
        self
    }

    pub fn member_b(&mut self, member_b: String) -> &mut SerializeMeBuilder {
        self.member_b = Some(member_b);
        self
    }

    pub fn nested(&mut self, nested: Nested) -> &mut SerializeMeBuilder {
        self.nested = Some(nested);
        self
    }
}

impl ShapeBuilder<SerializeMe> for SerializeMeBuilder {
    fn schema() -> &'static Schema<'static> {
        &*SCHEMA
    }

    fn build(self) -> SerializeMe {
        SerializeMe {
            member_a: self.member_a.expect("Could not find member_a"),
            member_b: self.member_b.expect("Could not find member_b"),
            nested: self.nested.expect("Could not find nested")
        }
    }

    fn deserialize<D: Deserializer>(&mut self, decoder: &mut D) -> &mut Self {
        decoder.read_struct(&*SCHEMA, self, MemberConsumer {});
        self
    }
}

struct MemberConsumer;
impl <D: Deserializer> StructMemberConsumer<SerializeMeBuilder, D> for MemberConsumer {
    fn accept(&self, state: &mut SerializeMeBuilder, member_schema: &Schema, member_deserializer: &mut D) {
        println!("FOR MEMBER: {}", member_schema.id.id);
        match member_schema.member_index{
            // Should these raise result?
            Some(0) => state.member_a(member_deserializer.read_string(&*MEMBER_A).unwrap().to_string()).ignore(),
            Some(1) => state.member_b(member_deserializer.read_string(&*MEMBER_B).unwrap().to_string()).ignore(),
            Some(2) => {
                let mut builder = Nested::builder();
                builder.deserialize(member_deserializer);
                state.nested(builder.build()).ignore()
            }
            // TODO: Throw real error?
            _ => panic!("Expected member index")
        }
    }
}
