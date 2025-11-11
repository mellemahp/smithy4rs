use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use indexmap::IndexMap;
use smithy4rs_core::{
    lazy_schema,
    prelude::*,
    schema::{Schema, SchemaBuilder, SchemaRef, ShapeId},
    serde::serializers::SerializeWithSchema,
    traits,
};
use smithy4rs_core_derive::SerializableStruct;
use smithy4rs_json_codec::JsonSerializer;
use std::sync::{Arc, LazyLock};

// Simple struct for basic benchmarks
lazy_schema!(
    SIMPLE_SCHEMA,
    Schema::structure_builder(ShapeId::from("bench#Simple"), traits![]),
    (NAME, "name", STRING, traits![]),
    (AGE, "age", INTEGER, traits![]),
    (SCORE, "score", DOUBLE, traits![]),
    (ACTIVE, "active", BOOLEAN, traits![])
);

#[derive(SerializableStruct)]
#[smithy_schema(SIMPLE_SCHEMA)]
struct SimpleStruct {
    #[smithy_schema(NAME)]
    name: String,
    #[smithy_schema(AGE)]
    age: i32,
    #[smithy_schema(SCORE)]
    score: f64,
    #[smithy_schema(ACTIVE)]
    active: bool,
}

// Address for nested benchmarks
lazy_schema!(
    ADDRESS_SCHEMA,
    Schema::structure_builder(ShapeId::from("bench#Address"), traits![]),
    (STREET, "street", STRING, traits![]),
    (CITY, "city", STRING, traits![]),
    (ZIP_CODE, "zip_code", STRING, traits![])
);

#[derive(SerializableStruct, Clone)]
#[smithy_schema(ADDRESS_SCHEMA)]
struct Address {
    #[smithy_schema(STREET)]
    street: String,
    #[smithy_schema(CITY)]
    city: String,
    #[smithy_schema(ZIP_CODE)]
    zip_code: String,
}

// List schema
pub static ADDRESS_LIST_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::list_builder(ShapeId::from("bench#AddressList"), traits![])
        .put_member("member", &ADDRESS_SCHEMA, traits![])
        .build()
});

// Map schema
pub static ADDRESS_MAP_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::map_builder(ShapeId::from("bench#AddressMap"), traits![])
        .put_member("key", &STRING, traits![])
        .put_member("value", &ADDRESS_SCHEMA, traits![])
        .build()
});

// Person with nested fields
lazy_schema!(
    PERSON_SCHEMA,
    Schema::structure_builder(ShapeId::from("bench#Person"), traits![]),
    (PERSON_NAME, "name", STRING, traits![]),
    (PERSON_AGE, "age", INTEGER, traits![]),
    (PRIMARY_ADDRESS, "primary_address", ADDRESS_SCHEMA, traits![]),
    (ADDRESSES, "addresses", ADDRESS_LIST_SCHEMA, traits![])
);

#[derive(SerializableStruct)]
#[smithy_schema(PERSON_SCHEMA)]
struct Person {
    #[smithy_schema(PERSON_NAME)]
    name: String,
    #[smithy_schema(PERSON_AGE)]
    age: i32,
    #[smithy_schema(PRIMARY_ADDRESS)]
    primary_address: Address,
    #[smithy_schema(ADDRESSES)]
    addresses: Vec<Address>,
}

// Recursive organization
pub static ORG_BUILDER: LazyLock<Arc<SchemaBuilder>> = LazyLock::new(|| {
    Arc::new(Schema::structure_builder(ShapeId::from("bench#Organization"), traits![]))
});

pub static ORG_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    ORG_BUILDER
        .put_member("name", &STRING, traits![])
        .put_member("parent", &*ORG_BUILDER, traits![])
        .build()
});

static ORG_NAME: LazyLock<&SchemaRef> = LazyLock::new(|| ORG_SCHEMA.expect_member("name"));
static ORG_PARENT: LazyLock<&SchemaRef> = LazyLock::new(|| ORG_SCHEMA.expect_member("parent"));

#[derive(SerializableStruct)]
#[smithy_schema(ORG_SCHEMA)]
struct Organization {
    #[smithy_schema(ORG_NAME)]
    name: String,
    #[smithy_schema(ORG_PARENT)]
    parent: Option<Box<Organization>>,
}

fn create_simple_struct() -> SimpleStruct {
    SimpleStruct {
        name: "John Doe".to_string(),
        age: 35,
        score: 98.5,
        active: true,
    }
}

fn create_address(num: usize) -> Address {
    Address {
        street: format!("{} Main St", num),
        city: "Springfield".to_string(),
        zip_code: format!("{:05}", num),
    }
}

fn create_person_with_addresses(count: usize) -> Person {
    let addresses = (0..count).map(create_address).collect();
    Person {
        name: "Jane Smith".to_string(),
        age: 42,
        primary_address: create_address(0),
        addresses,
    }
}

fn create_recursive_org(depth: usize) -> Organization {
    if depth == 0 {
        Organization {
            name: "Root Org".to_string(),
            parent: None,
        }
    } else {
        Organization {
            name: format!("Org Level {}", depth),
            parent: Some(Box::new(create_recursive_org(depth - 1))),
        }
    }
}

fn bench_simple_struct(c: &mut Criterion) {
    let simple = create_simple_struct();
    let mut buf = Vec::new();

    c.bench_function("serialize_simple_struct", |b| {
        b.iter(|| {
            let serializer = JsonSerializer::new(&mut buf);
            simple
                .serialize_with_schema(black_box(&SIMPLE_SCHEMA), serializer)
                .unwrap();
            black_box(&buf);
        });
    });
}

fn bench_nested_struct(c: &mut Criterion) {
    let person = create_person_with_addresses(5);
    let mut buf = Vec::new();

    let mut group = c.benchmark_group("serialize_nested");

    // Estimate throughput based on final JSON size
    buf.clear();
    let serializer = JsonSerializer::new(&mut buf);
    person.serialize_with_schema(&PERSON_SCHEMA, serializer).unwrap();
    let json_size = buf.len();

    group.throughput(Throughput::Bytes(json_size as u64));
    group.bench_function("person_with_5_addresses", |b| {
        b.iter(|| {
            let serializer = JsonSerializer::new(&mut buf);
            person
                .serialize_with_schema(black_box(&PERSON_SCHEMA), serializer)
                .unwrap();
            black_box(&buf);
        });
    });

    group.finish();
}

fn bench_list_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_lists");

    for size in [10, 100, 1000].iter() {
        let addresses: Vec<Address> = (0..*size).map(create_address).collect();
        let mut buf = Vec::new();

        // Measure throughput
        buf.clear();
        let serializer = JsonSerializer::new(&mut buf);
        addresses.serialize_with_schema(&ADDRESS_LIST_SCHEMA, serializer).unwrap();
        let json_size = buf.len();

        group.throughput(Throughput::Bytes(json_size as u64));
        group.bench_function(format!("{}_addresses", size), |b| {
            b.iter(|| {
                let serializer = JsonSerializer::new(&mut buf);
                addresses
                    .serialize_with_schema(black_box(&ADDRESS_LIST_SCHEMA), serializer)
                    .unwrap();
                black_box(&buf);
            });
        });
    }

    group.finish();
}

fn bench_map_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_maps");

    for size in [10, 100, 1000].iter() {
        let mut map = IndexMap::new();
        for i in 0..*size {
            map.insert(format!("key{}", i), create_address(i));
        }
        let mut buf = Vec::new();

        // Measure throughput
        buf.clear();
        let serializer = JsonSerializer::new(&mut buf);
        map.serialize_with_schema(&ADDRESS_MAP_SCHEMA, serializer).unwrap();
        let json_size = buf.len();

        group.throughput(Throughput::Bytes(json_size as u64));
        group.bench_function(format!("{}_entries", size), |b| {
            b.iter(|| {
                let serializer = JsonSerializer::new(&mut buf);
                map.serialize_with_schema(black_box(&ADDRESS_MAP_SCHEMA), serializer)
                    .unwrap();
                black_box(&buf);
            });
        });
    }

    group.finish();
}

fn bench_recursive_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_recursive");

    for depth in [5, 10, 20].iter() {
        let org = create_recursive_org(*depth);
        let mut buf = Vec::new();

        // Measure throughput
        buf.clear();
        let serializer = JsonSerializer::new(&mut buf);
        org.serialize_with_schema(&ORG_SCHEMA, serializer).unwrap();
        let json_size = buf.len();

        group.throughput(Throughput::Bytes(json_size as u64));
        group.bench_function(format!("depth_{}", depth), |b| {
            b.iter(|| {
                let serializer = JsonSerializer::new(&mut buf);
                org.serialize_with_schema(black_box(&ORG_SCHEMA), serializer)
                    .unwrap();
                black_box(&buf);
            });
        });
    }

    group.finish();
}

fn bench_string_escaping(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_escaping");

    // Simple ASCII string (fast path)
    let simple = SimpleStruct {
        name: "Simple ASCII String With No Special Characters".to_string(),
        age: 42,
        score: 3.14,
        active: true,
    };

    // String with escapes
    let escaped = SimpleStruct {
        name: "String with \"quotes\" and \n newlines \t tabs".to_string(),
        age: 42,
        score: 3.14,
        active: true,
    };

    let mut buf = Vec::new();

    group.bench_function("no_escapes", |b| {
        b.iter(|| {
            let serializer = JsonSerializer::new(&mut buf);
            simple
                .serialize_with_schema(black_box(&SIMPLE_SCHEMA), serializer)
                .unwrap();
            black_box(&buf);
        });
    });

    group.bench_function("with_escapes", |b| {
        b.iter(|| {
            let serializer = JsonSerializer::new(&mut buf);
            escaped
                .serialize_with_schema(black_box(&SIMPLE_SCHEMA), serializer)
                .unwrap();
            black_box(&buf);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_struct,
    bench_nested_struct,
    bench_list_serialization,
    bench_map_serialization,
    bench_recursive_serialization,
    bench_string_escaping
);
criterion_main!(benches);
