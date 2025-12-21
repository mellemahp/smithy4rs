//! Benchmarks of Validation

use std::hint::black_box;

use bigdecimal::BigDecimal;
use criterion::{Criterion, criterion_group, criterion_main};
use indexmap::IndexMap;
use smithy4rs_core::{
    prelude::*,
    serde::{
        ShapeBuilder,
        validate::{DefaultValidator, Validator},
    },
    smithy,
};
use smithy4rs_core_derive::SmithyShape;

// ==== Test shapes ====
smithy!("test#ValidationStruct": {
    structure VALIDATE_SHAPE_SCHEMA {
        @LengthTrait::builder().min(1).max(100).build();
        STRING: STRING = "string"
        @RangeTrait::builder().max(BigDecimal::from(100)).build();
        REQUIRED_INT: INTEGER = "required_int"
        INTEGER: INTEGER = "integer"
    }
});
#[derive(SmithyShape, Clone)]
#[smithy_schema(VALIDATE_SHAPE_SCHEMA)]
pub struct ValidatedStruct {
    #[smithy_schema(STRING)]
    string: String,
    #[smithy_schema(REQUIRED_INT)]
    required_int: i32,
    #[smithy_schema(INTEGER)]
    integer: Option<i32>,
}

smithy!("test#UnvalidatedShape": {
    structure UNVALIDATED_SHAPE_SCHEMA {
        STRING: STRING = "string"
        REQUIRED_INT: INTEGER = "required_int"
        INT: INTEGER = "integer"
    }
});
#[derive(SmithyShape, Clone)]
#[smithy_schema(UNVALIDATED_SHAPE_SCHEMA)]
pub struct UnvalidatedStruct {
    #[smithy_schema(STRING)]
    string: String,
    #[smithy_schema(REQUIRED_INT)]
    required_int: i32,
    #[smithy_schema(INT)]
    integer: Option<i32>,
}

smithy!("com.example#ListOfNested": {
    list LIST_OF_NESTED_SCHEMA {
        member: VALIDATE_SHAPE_SCHEMA
    }
});
smithy!("com.example#MapOfNested": {
    map MAP_OF_NESTED_SCHEMA {
        key: STRING
        value: VALIDATE_SHAPE_SCHEMA
    }
});

smithy!("test#StructWithNestedList": {
    structure STRUCT_WITH_COLLECTIONS {
        LIST: LIST_OF_NESTED_SCHEMA = "field_nested_list"
        MAP: MAP_OF_NESTED_SCHEMA = "field_nested_map"
    }
});

#[derive(SmithyShape, Clone)]
#[smithy_schema(STRUCT_WITH_COLLECTIONS)]
pub struct StructWithCollections {
    #[smithy_schema(LIST)]
    field_nested_list: Option<Vec<ValidatedStruct>>,
    #[smithy_schema(MAP)]
    field_nested_map: Option<IndexMap<String, ValidatedStruct>>,
}

smithy!("test#StructWithNestedSet": {
    structure STRUCT_WITH_SET {
        @UniqueItemsTrait;
        NESTED_SET: LIST_OF_NESTED_SCHEMA = "field_nested_set"
    }
});

#[derive(SmithyShape, Clone)]
#[smithy_schema(STRUCT_WITH_SET)]
pub struct StructWithSet {
    #[smithy_schema(NESTED_SET)]
    field_nested_set: Option<Vec<ValidatedStruct>>,
}

smithy!("test#StructWithNestedList": {
    structure STRUCT_WITH_LIST {
        NESTED_LIST: LIST_OF_NESTED_SCHEMA = "field_nested_list"
    }
});

// Mostly just for comparison against set implementation.
#[derive(SmithyShape, Clone)]
#[smithy_schema(STRUCT_WITH_LIST)]
pub struct StructWithList {
    #[smithy_schema(NESTED_LIST)]
    field_nested_list: Option<Vec<ValidatedStruct>>,
}

// ==== Benchmarks ====
pub fn validate_builder(c: &mut Criterion) {
    let builder = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(1);
    c.bench_function("Validate Shape Builder", |b| {
        b.iter(|| {
            let _ = black_box(DefaultValidator::new().validate(&VALIDATE_SHAPE_SCHEMA, &builder));
        })
    });
}

pub fn validate_shape(c: &mut Criterion) {
    let built_shape = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(1)
        .build()
        .expect("Shape should build");
    c.bench_function("Validate built shape", |b| {
        b.iter(|| {
            let _ =
                black_box(DefaultValidator::new().validate(&VALIDATE_SHAPE_SCHEMA, &built_shape));
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
            let _ = black_box(
                DefaultValidator::new().validate(&UNVALIDATED_SHAPE_SCHEMA, &unvalidated_shape),
            );
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
    map.insert("c".to_string(), builder);
    let collection = StructWithCollectionsBuilder::new()
        .field_nested_map_builder(map)
        .field_nested_list_builder(list);
    c.bench_function("Collections of Builders", |b| {
        b.iter(|| {
            let _ =
                black_box(DefaultValidator::new().validate(&UNVALIDATED_SHAPE_SCHEMA, &collection));
        })
    });
}

pub fn built_shape_with_collections(c: &mut Criterion) {
    let built = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(1)
        .build()
        .expect("Shape should build");
    let list = vec![built.clone(), built.clone(), built.clone()];
    let mut map = IndexMap::new();
    map.insert("a".to_string(), built.clone());
    map.insert("b".to_string(), built.clone());
    map.insert("c".to_string(), built);
    let collection = StructWithCollectionsBuilder::new()
        .field_nested_map(map)
        .field_nested_list(list)
        .build()
        .expect("Shape should build");
    c.bench_function("Collections of Built", |b| {
        b.iter(|| {
            let _ =
                black_box(DefaultValidator::new().validate(&STRUCT_WITH_COLLECTIONS, &collection));
        })
    });
}

// Primarily for comparison against set implementation.
pub fn built_shape_with_list(c: &mut Criterion) {
    let built = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(1)
        .build()
        .expect("Shape should build");
    let collection = StructWithList {
        field_nested_list: Some(vec![built.clone(), built.clone(), built]),
    };
    c.bench_function("List of Built", |b| {
        b.iter(|| {
            let _ = black_box(DefaultValidator::new().validate(&STRUCT_WITH_LIST, &collection));
        })
    });
}

pub fn built_shape_with_set(c: &mut Criterion) {
    let built1 = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(2)
        .build()
        .expect("Shape should build");
    let built2 = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(2)
        .build()
        .expect("Shape should build");
    let built3 = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(3)
        .build()
        .expect("Shape should build");
    let collection = StructWithSet {
        field_nested_set: Some(vec![built1, built2, built3]),
    };
    c.bench_function("Set of Built", |b| {
        b.iter(|| {
            let _ = black_box(DefaultValidator::new().validate(&STRUCT_WITH_SET, &collection));
        })
    });
}

criterion_group!(basic, validate_builder, validate_shape, unvalidated_shape);
criterion_group!(
    collections,
    builder_with_collections,
    built_shape_with_collections,
    built_shape_with_list,
    built_shape_with_set
);
criterion_main!(basic, collections);
