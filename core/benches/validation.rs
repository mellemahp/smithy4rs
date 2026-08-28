//! Benchmarks of Validation

use std::hint::black_box;

use bigdecimal::BigDecimal;
use criterion::{Criterion, criterion_group, criterion_main};
use smithy4rs_core::{
    IndexMap,
    derive::SmithyShape,
    prelude::*,
    serde::validation::{DefaultValidator, Validator},
    smithy,
};

// ==== Test shapes ====
smithy!(
    structure test::ValidationStruct {
        @LengthTrait::builder().min(1).max(100).build();
        string: STRING
        @RangeTrait::builder().max(BigDecimal::from(100u64)).build();
        requiredInt: INTEGER
        integer: INTEGER
    }
);
#[derive(SmithyShape, Clone)]
#[schema(schema = VALIDATION_STRUCT)]
pub struct ValidatedStruct {
    pub string: String,
    pub required_int: i32,
    pub integer: Option<i32>,
}

smithy!(
    structure test::UnvalidatedShape {
        string: STRING
        requiredInt: INTEGER
        integer: INTEGER
    }
);
#[derive(SmithyShape, Clone)]
#[schema(schema = UNVALIDATED_SHAPE)]
pub struct UnvalidatedStruct {
    pub string: String,
    pub required_int: i32,
    pub integer: Option<i32>,
}

smithy!(
    list com::example::ListOfNested {
        member: VALIDATION_STRUCT
    }
);
smithy!(
    map com::example::MapOfNested {
        key: STRING
        value: VALIDATION_STRUCT
    }
);

smithy!(
    structure test::StructWithCollections {
        fieldNestedList: LIST_OF_NESTED
        fieldNestedMap: MAP_OF_NESTED
    }
);

#[derive(SmithyShape, Clone)]
#[schema(schema = STRUCT_WITH_COLLECTIONS)]
pub struct StructWithCollections {
    pub field_nested_list: Option<Vec<ValidatedStruct>>,
    pub field_nested_map: Option<IndexMap<String, ValidatedStruct>>,
}

smithy!(
    structure test::StructWithSet {
        @UniqueItemsTrait::builder().build();
        fieldNestedSet: LIST_OF_NESTED
    }
);

#[derive(SmithyShape, Clone)]
#[schema(schema = STRUCT_WITH_SET)]
pub struct StructWithSet {
    pub field_nested_set: Option<Vec<ValidatedStruct>>,
}

smithy!(
    structure test::StructWithList {
        fieldNestedList: LIST_OF_NESTED
    }
);

// Mostly just for comparison against set implementation.
#[derive(SmithyShape, Clone)]
#[schema(schema = STRUCT_WITH_LIST)]
pub struct StructWithList {
    pub field_nested_list: Option<Vec<ValidatedStruct>>,
}

// ==== Benchmarks ====
pub fn validate_builder(c: &mut Criterion) {
    let builder = ValidatedStructBuilder::new()
        .string("string".to_string())
        .required_int(1);
    c.bench_function("Validate Shape Builder", |b| {
        b.iter(|| {
            let _ = black_box(DefaultValidator::new().validate(&VALIDATION_STRUCT, &builder));
        })
    });
}

pub fn validate_shape(c: &mut Criterion) {
    let built_shape = ValidatedStructBuilder::new()
        .string("string")
        .required_int(1)
        .build()
        .expect("Shape should build");
    c.bench_function("Validate built shape", |b| {
        b.iter(|| {
            let _ = black_box(DefaultValidator::new().validate(&VALIDATION_STRUCT, &built_shape));
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
            let _ =
                black_box(DefaultValidator::new().validate(&UNVALIDATED_SHAPE, &unvalidated_shape));
        })
    });
}

pub fn builder_with_collections(c: &mut Criterion) {
    let builder = ValidatedStructBuilder::new()
        .string("string")
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
            let _ = black_box(DefaultValidator::new().validate(&UNVALIDATED_SHAPE, &collection));
        })
    });
}

pub fn built_shape_with_collections(c: &mut Criterion) {
    let built = ValidatedStructBuilder::new()
        .string("string")
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
        .string("string")
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
        .string("string")
        .required_int(2)
        .build()
        .expect("Shape should build");
    let built2 = ValidatedStructBuilder::new()
        .string("string")
        .required_int(2)
        .build()
        .expect("Shape should build");
    let built3 = ValidatedStructBuilder::new()
        .string("string")
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
