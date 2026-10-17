use criterion::{Criterion, criterion_group, criterion_main, BatchSize};
use smithy4rs_core::{
    IndexMap,
    derive::SmithyShape,
    prelude::{INTEGER, STRING},
    schema::Document,
    smithy,
};
use smithy4rs_core::schema::{DocumentValue, MapCursor, ShapeId};

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
        memberList: LIST
        memberMap: MAP
    }
);

#[derive(SmithyShape, Clone, PartialEq)]
#[schema(schema = SHAPE)]
pub struct SerializeMe {
    pub member_a: String,
    pub member_b: i32,
    pub member_list: Vec<String>,
    pub member_map: IndexMap<String, String>,
}

impl Document for SerializeMe {
    fn discriminator(&self) -> Option<&ShapeId> {
        Some(SHAPE.id())
    }

    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::Map(self)
    }

    fn len(&self) -> usize {
        4
    }
}

impl MapCursor for SerializeMe {
    #[inline]
    fn get_key(&self, idx: usize) -> Option<&str> {
        match idx {
            0 => Some("memberA"),
            1 => Some("memberB"),
            2 => Some("memberOptional"),
            3 => Some("memberList"),
            4 => Some("memberMap"),
            _ => None,
        }
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get(&self, key: &str) -> Option<&dyn Document> {
        match key {
            "memberA" => Some(&self.member_a),
            "memberB" => Some(&self.member_b),
            "memberList" => Some(&self.member_list),
            "memberMap" => Some(&self.member_map),
            _ => None
        }
    }

    fn get_entry(&self, idx: usize) -> Option<(&str, &dyn Document)> {
        match idx {
            0 => Some(("memberA", &self.member_a)),
            1 => Some(("memberB", &self.member_b)),
            3 => Some(("memberList", &self.member_list)),
            4 => Some(("memberMap", &self.member_map)),
            _ => None,
        }
    }

    fn len(&self) -> usize {
        4
    }
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
            BatchSize::SmallInput
        );
    });
}

fn convert_from(c: &mut Criterion) {
    // let mut map = IndexMap::new();
    // map.insert(String::from("a"), String::from("b"));
    // let struct_to_convert = SerializeMeBuilder::new()
    //     .member_a(String::from("a"))
    //     .member_b(1)
    //    // .member_list(vec!["a".to_string(), "b".to_string()])
    //    // .member_map(map)
    //     .build()
    //     .expect("Should build document");
    // let document: Box<dyn Document> = struct_to_convert.into();
    //
    // c.bench_function("Document to shape", |b| {
    //     b.iter(|| {
    //         let _ = black_box(
    //             <SerializeMeBuilder as TryFromDocument>::try_from(document.clone())
    //                 .expect("Should convert to document")
    //                 .build()
    //                 .expect("Should build document"),
    //         );
    //     });
    // });
}

criterion_group!(into, convert_into);
// criterion_group!(from, convert_from);
criterion_main!(into); //, from
