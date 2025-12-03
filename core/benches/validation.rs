//! Benchmarks of Validation

use std::hint::black_box;
use std::sync::LazyLock;
use bigdecimal::BigDecimal;
use criterion::{criterion_group, criterion_main, Criterion};
use indexmap::IndexMap;
use smithy4rs_core::prelude::{LengthTrait, RangeTrait, RequiredTrait, UniqueItemsTrait, INTEGER, STRING};
use smithy4rs_core::schema::{Schema, SchemaRef, ShapeId, StaticSchemaShape};
use smithy4rs_core::serde::builders::{MaybeBuilt, Required};
use smithy4rs_core::serde::correction::{ErrorCorrection, ErrorCorrectionDefault};
use smithy4rs_core::serde::de::{DeserializeWithSchema, Deserializer};
use smithy4rs_core::serde::se::{SerializeWithSchema, Serializer, StructSerializer};
use smithy4rs_core::serde::ShapeBuilder;
use smithy4rs_core::serde::validate::Validator;
use smithy4rs_core::traits;

// ==== Test shapes ====
static VALIDATE_SHAPE_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("test#ValidationStruct"), Vec::new())
        .put_member("string", &STRING, traits![LengthTrait::builder().min(1).max(100).build(), RequiredTrait])
        .put_member("required_int", &INTEGER, traits![RequiredTrait])
        .put_member("integer", &INTEGER, traits![RangeTrait::builder().max(BigDecimal::from(100)).build()])
        .build()
});
static FIELD_STRING: LazyLock<&SchemaRef> = LazyLock::new(|| { VALIDATE_SHAPE_SCHEMA.expect_member("string") });
static FIELD_REQUIRED_INT: LazyLock<&SchemaRef> = LazyLock::new(|| { VALIDATE_SHAPE_SCHEMA.expect_member("required_int") });
static FIELD_INTEGER: LazyLock<&SchemaRef> = LazyLock::new(|| { VALIDATE_SHAPE_SCHEMA.expect_member("integer") });

#[derive(Clone)]
pub struct ValidatedStruct {
    string: String,
    required_int: i32,
    integer: Option<i32>
}
impl StaticSchemaShape for ValidatedStruct {
    fn schema() -> &'static SchemaRef {
        &VALIDATE_SHAPE_SCHEMA
    }
}
impl SerializeWithSchema for ValidatedStruct {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: S
    ) -> Result<S::Ok, S::Error> {
        let mut ser = serializer.write_struct(schema, 2usize)?;
        ser.serialize_member_named("string", &FIELD_STRING, &self.string)?;
        ser.serialize_member_named("required_int", &FIELD_REQUIRED_INT, &self.required_int)?;
        ser.serialize_member_named("integer", &FIELD_INTEGER, &self.integer)?;
        ser.end(schema)
    }
}

#[derive(Clone)]
pub struct ValidatedStructBuilder {
    string: Required<String>,
    required_int: Required<i32>,
    integer: Option<i32>
}
impl ValidatedStructBuilder {
    pub fn string(mut self, value: String) -> Self {
        self.string = Required::Set(value);
        self
    }
    pub fn required_int(mut self, value: i32) -> Self {
        self.required_int = Required::Set(value);
        self
    }
    pub fn integer(mut self, value: i32) -> Self {
        self.integer = Some(value);
        self
    }
}
impl <'de> DeserializeWithSchema<'de> for ValidatedStructBuilder {
    fn deserialize_with_schema<D>(_schema: &SchemaRef, _deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        unimplemented!("We dont need to deserialize to test.")
    }
}
impl ErrorCorrectionDefault for ValidatedStruct {
    fn default() -> Self {
        ValidatedStructBuilder::new().correct()
    }
}
impl SerializeWithSchema for ValidatedStructBuilder {
    fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
        let mut ser = serializer.write_struct(schema, 1usize)?;
        ser.serialize_member_named("string", &FIELD_STRING, &self.string)?;
        ser.serialize_member_named("required_int", &FIELD_REQUIRED_INT, &self.required_int)?;
        ser.serialize_member_named("integer", &FIELD_INTEGER, &self.integer)?;
        ser.end(schema)
    }
}
impl <'de> ShapeBuilder<'de, ValidatedStruct> for ValidatedStructBuilder {
    fn new() -> Self {
        Self {
            string: Required::Unset,
            required_int: Required::Unset,
            integer: None,
        }
    }
}
impl ErrorCorrection for ValidatedStructBuilder {
    type Value = ValidatedStruct;

    fn correct(self) -> Self::Value {
        ValidatedStruct {
            string: self.string.get(),
            required_int: self.required_int.get(),
            integer: self.integer,
        }
    }
}

static UNVALIDATE_SHAPE_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("test#UnvalidatedShape"), Vec::new())
        .put_member("string", &STRING, traits![RequiredTrait])
        .put_member("required_int", &INTEGER, traits![RequiredTrait])
        .put_member("integer", &INTEGER, traits![])
        .build()
});
static FIELD_UNCHECKED_STRING: LazyLock<&SchemaRef> = LazyLock::new(|| { VALIDATE_SHAPE_SCHEMA.expect_member("string") });
static FIELD_UNCHECKED_REQUIRED_INT: LazyLock<&SchemaRef> = LazyLock::new(|| { VALIDATE_SHAPE_SCHEMA.expect_member("required_int") });
static FIELD_UNCHECKED_INT: LazyLock<&SchemaRef> = LazyLock::new(|| { VALIDATE_SHAPE_SCHEMA.expect_member("integer") });

pub struct UnvalidatedStruct {
    string: String,
    required_int: i32,
    integer: Option<i32>
}
impl StaticSchemaShape for UnvalidatedStruct {
    fn schema() -> &'static SchemaRef {
        &UNVALIDATE_SHAPE_SCHEMA
    }
}
impl SerializeWithSchema for UnvalidatedStruct {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: S
    ) -> Result<S::Ok, S::Error> {
        let mut ser = serializer.write_struct(schema, 2usize)?;
        ser.serialize_member_named("string", &FIELD_UNCHECKED_STRING, &self.string)?;
        ser.serialize_member_named("required_int", &FIELD_UNCHECKED_REQUIRED_INT, &self.required_int)?;
        ser.serialize_member_named("integer", &FIELD_UNCHECKED_INT, &self.integer)?;
        ser.end(schema)
    }
}

static LIST_OF_NESTED_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::list_builder(ShapeId::from("com.example#ListOfNested"), traits![])
        .put_member("member", &VALIDATE_SHAPE_SCHEMA, traits![])
        .build()
});
static MAP_OF_NESTED_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::map_builder(ShapeId::from("com.example#MapOfNested"), traits![])
        .put_member("key", &STRING, traits![])
        .put_member("value", &VALIDATE_SHAPE_SCHEMA, traits![])
        .build()
});
static STRUCT_WITH_COLLECTIONS: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("test#StructWithNestedList"), Vec::new())
        .put_member("field_nested_list", &LIST_OF_NESTED_SCHEMA, traits![])
        .put_member("field_nested_map", &MAP_OF_NESTED_SCHEMA, traits![])
        .build()
});
static FIELD_NESTED_LIST: LazyLock<&SchemaRef> = LazyLock::new(|| STRUCT_WITH_COLLECTIONS.expect_member("field_nested_list"));
static FIELD_NESTED_MAP: LazyLock<&SchemaRef> = LazyLock::new(|| STRUCT_WITH_COLLECTIONS.expect_member("field_nested_map"));

struct StructWithCollections {
    field_nested_list: Option<Vec<ValidatedStruct>>,
    field_nested_map: Option<IndexMap<String, ValidatedStruct>>,
}

impl StaticSchemaShape for StructWithCollections {
    fn schema() -> &'static SchemaRef {
        &STRUCT_WITH_COLLECTIONS
    }
}
impl SerializeWithSchema for StructWithCollections {
    fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
        let mut ser = serializer.write_struct(schema, 2usize)?;
        ser.serialize_optional_member_named("field_nested_list", &FIELD_NESTED_LIST, &self.field_nested_list)?;
        ser.serialize_optional_member_named("field_nested_map", &FIELD_NESTED_MAP, &self.field_nested_map)?;
        ser.end(schema)
    }
}
struct StructWithCollectionsBuilder {
    field_nested_list: Option<MaybeBuilt<Vec<ValidatedStruct>, Vec<ValidatedStructBuilder>>>,
    field_nested_map: Option<MaybeBuilt<IndexMap<String, ValidatedStruct>, IndexMap<String, ValidatedStructBuilder>>>,
}
impl <'de> DeserializeWithSchema<'de> for StructWithCollectionsBuilder {
    fn deserialize_with_schema<D>(_schema: &SchemaRef, _deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        unimplemented!("We dont need to deserialize for testing.")
    }
}
impl SerializeWithSchema for StructWithCollectionsBuilder {
    fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
        let mut ser = serializer.write_struct(schema, 2usize)?;
        ser.serialize_optional_member_named("field_nested_list", &FIELD_NESTED_LIST, &self.field_nested_list)?;
        ser.serialize_optional_member_named("field_nested_map", &FIELD_NESTED_MAP, &self.field_nested_map)?;
        ser.end(schema)
    }
}
impl ErrorCorrection for StructWithCollectionsBuilder {
    type Value = StructWithCollections;

    fn correct(self) -> Self::Value {
        StructWithCollections {
            field_nested_map: self.field_nested_map.correct(),
            field_nested_list: self.field_nested_list.correct(),
        }
    }
}
impl <'de> ShapeBuilder<'de, StructWithCollections> for StructWithCollectionsBuilder {
    fn new() -> Self {
        StructWithCollectionsBuilder {
            field_nested_map: None,
            field_nested_list: None,
        }
    }
}
impl StructWithCollectionsBuilder {
    pub fn field_nested_map(mut self, values: IndexMap<String, ValidatedStruct>) -> Self {
        self.field_nested_map = Some(MaybeBuilt::Struct(values));
        self
    }

    pub fn field_nested_map_builder(mut self, values: IndexMap<String, ValidatedStructBuilder>) -> Self {
        self.field_nested_map = Some(MaybeBuilt::Builder(values));
        self
    }

    pub fn field_nested_list(mut self, values: Vec<ValidatedStruct>) -> Self {
        self.field_nested_list = Some(MaybeBuilt::Struct(values));
        self
    }

    pub fn field_nested_list_builder(mut self, values: Vec<ValidatedStructBuilder>) -> Self {
        self.field_nested_list = Some(MaybeBuilt::Builder(values));
        self
    }
}

static STRUCT_WITH_SET: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("test#StructWithNestedSet"), Vec::new())
        .put_member("field_nested_set", &LIST_OF_NESTED_SCHEMA, traits![UniqueItemsTrait])
        .build()
});
static FIELD_NESTED_SET: LazyLock<&SchemaRef> = LazyLock::new(|| STRUCT_WITH_SET.expect_member("field_nested_set"));

struct StructWithSet {
    field_nested_set: Option<Vec<ValidatedStruct>>,
}
impl StaticSchemaShape for StructWithSet {
    fn schema() -> &'static SchemaRef {
        &STRUCT_WITH_SET
    }
}
impl SerializeWithSchema for StructWithSet {
    fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
        let mut ser = serializer.write_struct(schema, 1usize)?;
        ser.serialize_optional_member_named("field_nested_set", &FIELD_NESTED_SET, &self.field_nested_set)?;
        ser.end(schema)
    }
}

static STRUCT_WITH_LIST: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("test#StructWithNestedList"), Vec::new())
        .put_member("field_nested_list", &LIST_OF_NESTED_SCHEMA, traits![])
        .build()
});
static FIELD_WITH_NESTED_LIST: LazyLock<&SchemaRef> = LazyLock::new(|| STRUCT_WITH_LIST.expect_member("field_nested_list"));

// Mostly just for comparison against set implementation.
struct StructWithList {
    field_nested_list: Option<Vec<ValidatedStruct>>,
}
impl StaticSchemaShape for StructWithList {
    fn schema() -> &'static SchemaRef {
        &STRUCT_WITH_LIST
    }
}
impl SerializeWithSchema for StructWithList {
    fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
        let mut ser = serializer.write_struct(schema, 1usize)?;
        ser.serialize_optional_member_named("field_nested_list", &FIELD_WITH_NESTED_LIST, &self.field_nested_list)?;
        ser.end(schema)
    }
}

// ==== Benchmarks ====
pub fn validate_builder(c: &mut Criterion) {
    let builder = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(1);
    c.bench_function("Validate Shape Builder", |b| {
        b.iter(|| {
            let _ = black_box(Validator::new().validate(&VALIDATE_SHAPE_SCHEMA, &builder));
        })
    });
}

pub fn validate_shape(c: &mut Criterion) {
    let built_shape = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(1)
        .build().expect("Shape should build");
    c.bench_function("Validate built shape", |b| {
        b.iter(|| {
            let _ = black_box(Validator::new().validate(&VALIDATE_SHAPE_SCHEMA, &built_shape));
        })
    });
}

pub fn unvalidated_shape(c: &mut Criterion) {
    let unvalidated_shape = UnvalidatedStruct {
        string: "string".to_string(),
        required_int: 1,
        integer: None,
    };
    c.bench_function("Shape with no constraints", |b| {
        b.iter(|| {
            let _ = black_box(Validator::new().validate(&UNVALIDATE_SHAPE_SCHEMA, &unvalidated_shape));
        })
    });
}

pub fn builder_with_collections(c: &mut Criterion) {
    let builder = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(1);
    let list = vec![builder.clone(), builder.clone(), builder.clone()];
    let mut map = IndexMap::new();
    map.insert("a".to_string(), builder.clone());
    map.insert("b".to_string(), builder.clone());
    map.insert("c".to_string(), builder.clone());
    let collection = StructWithCollectionsBuilder::new()
        .field_nested_map_builder(map)
        .field_nested_list_builder(list);
    c.bench_function("Collections of Builders", |b| {
        b.iter(|| {
            let _ = black_box(Validator::new().validate(&UNVALIDATE_SHAPE_SCHEMA, &collection));
        })
    });
}

pub fn built_shape_with_collections(c: &mut Criterion) {
    let built = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(1).build()
        .expect("Shape should build");
    let list = vec![built.clone(), built.clone(), built.clone()];
    let mut map = IndexMap::new();
    map.insert("a".to_string(), built.clone());
    map.insert("b".to_string(), built.clone());
    map.insert("c".to_string(), built.clone());
    let collection = StructWithCollectionsBuilder::new()
        .field_nested_map(map)
        .field_nested_list(list)
        .build().expect("Shape should build");
    c.bench_function("Collections of Built", |b| {
        b.iter(|| {
            let _ = black_box(Validator::new().validate(&STRUCT_WITH_COLLECTIONS, &collection));
        })
    });
}

// Primarily for comparison against set implementation.
pub fn built_shape_with_list(c: &mut Criterion) {
    let built = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(1).build()
        .expect("Shape should build");
    let collection = StructWithList {
        field_nested_list: Some(vec![built.clone(), built.clone(), built.clone()]),
    };
    c.bench_function("List of Built", |b| {
        b.iter(|| {
            let _ = black_box(Validator::new().validate(&STRUCT_WITH_LIST, &collection));
        })
    });
}

pub fn built_shape_with_set(c: &mut Criterion) {
    let built1 = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(2).build()
        .expect("Shape should build");
    let built2 = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(2).build()
        .expect("Shape should build");
    let built3 = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(3).build()
        .expect("Shape should build");
    let collection = StructWithSet {
        field_nested_set: Some(vec![built1, built2, built3]),
    };
    c.bench_function("Set of Built", |b| {
        b.iter(|| {
            let _ = black_box(Validator::new().validate(&STRUCT_WITH_SET, &collection));
        })
    });
}

criterion_group!(basic, validate_builder, validate_shape, unvalidated_shape);
criterion_group!(collections, builder_with_collections, built_shape_with_collections, built_shape_with_list, built_shape_with_set);
criterion_main!(basic, collections);