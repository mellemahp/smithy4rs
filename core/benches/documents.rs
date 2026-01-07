use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use indexmap::IndexMap;
use smithy4rs_core::prelude::{INTEGER, STRING};
use smithy4rs_core::schema::Document;
use smithy4rs_core::smithy;
use smithy4rs_core_derive::SmithyShape;
use smithy4rs_core::serde::builders::ShapeBuilder;

smithy!("com.example#Map": {
        map MAP_SCHEMA {
            key: STRING
            value: STRING
        }
    });
smithy!("com.example#List": {
        list LIST_SCHEMA {
            member: STRING
        }
    });
smithy!("com.example#Shape": {
        structure SCHEMA {
            A: STRING = "a"
            B: INTEGER = "b"
            LIST: LIST_SCHEMA = "list"
            MAP: MAP_SCHEMA = "map"
        }
    });

#[derive(SmithyShape, Clone, PartialEq)]
#[smithy_schema(SCHEMA)]
pub struct SerializeMe {
    #[smithy_schema(A)]
    pub member_a: String,
    #[smithy_schema(B)]
    pub member_b: i32,
    #[smithy_schema(LIST)]
    pub member_list: Vec<String>,
    #[smithy_schema(MAP)]
    pub member_map: IndexMap<String, String>,
}

pub fn convert_into(c: &mut Criterion) {
    let mut map = IndexMap::new();
    map.insert(String::from("a"), String::from("b"));
    let struct_to_convert = SerializeMeBuilder::new()
        .member_a(String::from("a"))
        .member_b(1)
        .member_list(vec!["a".to_string(), "b".to_string()])
        .member_map(map)
        .build()
        .expect("Should build document");
    c.bench_function("Convert Shape into document", |b| {
        b.iter(|| {
            let _: Box<dyn Document> = black_box(struct_to_convert.clone().into());
        })
    });
}

pub fn convert_from(c: &mut Criterion) {
    let mut map = IndexMap::new();
    map.insert(String::from("a"), String::from("b"));
    let struct_to_convert = SerializeMeBuilder::new()
        .member_a(String::from("a"))
        .member_b(1)
        .member_list(vec!["a".to_string(), "b".to_string()])
        .member_map(map)
        .build()
        .expect("Should build document");
    let document: Box<dyn Document> = struct_to_convert.into();

    c.bench_function("Document to shape", |b| {
        b.iter(|| {
            let _ = black_box(SerializeMeBuilder::from_document(document.clone())
                .expect("Should convert to document")
                .build()
                .expect("Should build document"));
        })
    });
}

criterion_group!(into, convert_into);
criterion_group!(from, convert_from);
criterion_main!(into, from);
