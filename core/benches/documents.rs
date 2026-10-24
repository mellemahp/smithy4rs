use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use smithy4rs_core::{
    IndexMap,
    derive::SmithyShape,
    features::documents::{Document, TryFromDocument},
    prelude::{INTEGER, STRING},
    smithy,
};

smithy!(
    map com::example::Map {
        key: STRING
        value: STRING
    }
);
smithy!(
    list com::example::List {
        member: STRING
    }
);
smithy!(
    structure com::example::Shape {
        memberA: STRING
        memberB: INTEGER
        memberOptional: STRING
        memberList: LIST
        memberMap: MAP
    }
);

#[derive(SmithyShape, Clone, PartialEq)]
#[schema(schema = SHAPE)]
pub struct SerializeMe {
    pub member_a: String,
    pub member_b: i32,
    pub member_optional: Option<String>,
    pub member_list: Vec<String>,
    pub member_map: IndexMap<String, String>,
}

fn convert_into(c: &mut Criterion) {
    let mut map = IndexMap::new();
    map.insert(String::from("a"), String::from("b"));

    let struct_to_convert = SerializeMeBuilder::new()
        .member_a(String::from("a"))
        .member_b(1)
        .member_list(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        .member_map(map)
        .build()
        .expect("Should build shape");

    c.bench_function("Convert Shape into document", |b| {
        b.iter_batched(
            || struct_to_convert.clone(),
            |shape| {
                let doc: Box<dyn Document> = shape.into();
                doc
            },
            BatchSize::SmallInput,
        );
    });
}

fn convert_from(c: &mut Criterion) {
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

    // TODO: use .into()?
    c.bench_function("Document to shape", |b| {
        b.iter_batched(
            || document.clone(),
            |mut shape| {
                <SerializeMeBuilder as TryFromDocument>::try_from(&mut *shape)
                    .expect("Should convert to document")
                    .build()
                    .expect("Should build document")
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(into, convert_into);
criterion_group!(from, convert_from);
criterion_main!(into, from);
