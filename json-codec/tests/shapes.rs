use std::sync::LazyLock;
use smithy4rs_core::schema::{prelude, Schema};
use smithy4rs_core::serde::{Serializable, SerializableStruct, Serializer};
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
        .put_member("c", &*NESTED)
        .build()
});
static MEMBER_A: LazyLock<&Schema> = LazyLock::new(|| {
    SCHEMA.expect_member("a")
});
static MEMBER_B: LazyLock<&Schema> = LazyLock::new(|| {
    SCHEMA.expect_member("b")
});
static MEMBER_NESTED: LazyLock<&Schema> = LazyLock::new(|| {
    SCHEMA.expect_member("c")
});

//#[derive(SerializableStruct)]
//#[schema(SCHEMA)]
pub(crate) struct SerializeMe {
    pub member_a: String,
    pub member_b: String,
    pub nested: Nested
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
