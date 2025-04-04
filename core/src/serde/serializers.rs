use std::time::Instant;
use crate::schema::Schema;
use crate::BigDecimal;
use crate::BigInt;
use crate::ByteBuffer;
use crate::documents::Document;

trait Serializable {
    /// Serialize the state of the shape into the given serializer.
    fn serialize<T: Serializer>(&self, serializer: &mut T);
}

trait SerializableStruct: Serializable {
    fn schema() -> &'static Schema;
    fn serialize_members<T: Serializer>(&self, serializer: &mut T);
    // TODO: get member value
}

// TODO: docs
// TODO: Should this implement `Write`?
trait Serializer {
    fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: &T);
    fn write_map<T, M: MapSerializer>(&mut self, schema: &Schema, map_state:T, size: usize, consumer: fn(T, M));
    fn write_list<T, L: Serializer>(&mut self, schema: &Schema, list_state: T, size: usize, consumer: fn(T, L));
    fn write_boolean(&mut self, schema: &Schema, value: bool);
    fn write_byte(&mut self, schema: &Schema, value: u8);
    fn write_short(&mut self, schema: &Schema, value: i16);
    fn write_integer(&mut self, schema: &Schema, value: i32);
    fn write_long(&mut self,schema: &Schema, value: i64);
    fn write_float(&mut self, schema: &Schema, value: f32);
    fn write_double(&mut self, schema: &Schema, value: f64);
    fn write_big_integer(&mut self, schema: &Schema, value: BigInt);
    fn write_big_decimal(&mut self, schema: &Schema, value: BigDecimal);
    fn write_string(&mut self, schema: &Schema, value: &str);
    fn write_blob(&mut self, schema: &Schema, value: ByteBuffer);
    // TODO: datastream?
    // TODO: event stream?
    fn write_timestamp(&mut self, schema: &Schema, value: Instant);
    fn write_document(&mut self, schema: &Schema, value: Document);
    fn write_null(&mut self, schema: &Schema);
    fn flush();
}

trait MapSerializer {
    fn write_entry<T, S: Serializer>(key_schema: &Schema, key: &str, state: T, value_serializer: fn(T, S));
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;
    use super::*;
    use crate::*;
    use crate::shapes::{ShapeId};

    static TARGET: LazyLock<Schema> = LazyLock::new(|| {
        Schema::create_string(ShapeId::from("com.example#Shape$memberA"))
    });

    static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("com.example#Shape"))
            .put_member("a", &*TARGET)
            .build()
    });
    static MEMBER_A: LazyLock<&Schema> = LazyLock::new(|| {
         SCHEMA.expect_member("a")
    });

    struct SerializeMe {
        pub member_a: String
    }

    impl Serializable for SerializeMe {
        fn serialize<T: Serializer>(&self, serializer: &mut T) {
            serializer.write_struct(&*SCHEMA, self)
        }
    }

    impl SerializableStruct for SerializeMe {
        fn schema() -> &'static Schema {
            &*SCHEMA
        }

        fn serialize_members<T: Serializer>(&self, serializer: &mut T) {
            serializer.write_string(&*MEMBER_A, &self.member_a);
           //serializer::write_string(&*MEMBER_A, &self.member_a)
        }
    }
    struct TestSerializer {}

    impl Serializer for TestSerializer {
        fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: &T) {
            print!("{}=<", schema.id.name);
            structure.serialize_members(self);
            println!(">")
        }

        fn write_map<T, M: MapSerializer>(&mut self, schema: &Schema, map_state: T, size: usize, consumer: fn(T, M)) {
            todo!()
        }

        fn write_list<T, L: Serializer>(&mut self, schema: &Schema, list_state: T, size: usize, consumer: fn(T, L)) {
            todo!()
        }

        fn write_boolean(&mut self, schema: &Schema, value: bool) {
            todo!()
        }

        fn write_byte(&mut self, schema: &Schema, value: u8) {
            todo!()
        }

        fn write_short(&mut self, schema: &Schema, value: i16) {
            todo!()
        }

        fn write_integer(&mut self, schema: &Schema, value: i32) {
            todo!()
        }

        fn write_long(&mut self, schema: &Schema, value: i64) {
            todo!()
        }

        fn write_float(&mut self, schema: &Schema, value: f32) {
            todo!()
        }

        fn write_double(&mut self, schema: &Schema, value: f64) {
            todo!()
        }

        fn write_big_integer(&mut self, schema: &Schema, value: BigInt) {
            todo!()
        }

        fn write_big_decimal(&mut self, schema: &Schema, value: BigDecimal) {
            todo!()
        }

        fn write_string(&mut self, schema: &Schema, value: &str) {
            print!("{} : {}", schema.id.member.as_ref().unwrap(), value);
        }

        fn write_blob(&mut self, schema: &Schema, value: ByteBuffer) {
            todo!()
        }

        fn write_timestamp(&mut self, schema: &Schema, value: Instant) {
            todo!()
        }

        fn write_document(&mut self, schema: &Schema, value: Document) {
            todo!()
        }

        fn write_null(&mut self, schema: &Schema) {
            todo!()
        }

        fn flush() {
            todo!()
        }
    }

    #[test]
    fn test_serde() {
        let mut serializer = TestSerializer {};
        let structure = SerializeMe { member_a: "Hello".to_string() };
        structure.serialize(&mut serializer);
    }
}