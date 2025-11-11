#![allow(unused_extern_crates)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use indexmap::IndexMap;
use smithy4rs_core::{
    lazy_schema,
    prelude::*,
    schema::{Schema, ShapeId},
    serde::serializers::SerializeWithSchema,
    traits,
};
use smithy4rs_core_derive::SerializableStruct;
use smithy4rs_json_codec::JsonSerializer;

// Define structs that work with both serde and smithy4rs

// Address struct for smithy4rs
lazy_schema!(
    ADDRESS_SCHEMA,
    Schema::structure_builder(ShapeId::from("bench#Address"), traits![]),
    (STREET, "street", STRING, traits![]),
    (CITY, "city", STRING, traits![]),
    (ZIP_CODE, "zip_code", STRING, traits![])
);

#[derive(SerializableStruct, Clone, serde::Serialize)]
#[smithy_schema(ADDRESS_SCHEMA)]
struct Address {
    #[smithy_schema(STREET)]
    street: String,
    #[smithy_schema(CITY)]
    city: String,
    #[smithy_schema(ZIP_CODE)]
    zip_code: String,
}

// Simple struct
lazy_schema!(
    PERSON_SCHEMA,
    Schema::structure_builder(ShapeId::from("bench#Person"), traits![]),
    (NAME, "name", STRING, traits![]),
    (AGE, "age", INTEGER, traits![]),
    (EMAIL, "email", STRING, traits![])
);

#[derive(SerializableStruct, Clone, serde::Serialize)]
#[smithy_schema(PERSON_SCHEMA)]
struct Person {
    #[smithy_schema(NAME)]
    name: String,
    #[smithy_schema(AGE)]
    age: i32,
    #[smithy_schema(EMAIL)]
    email: String,
}

fn create_person() -> Person {
    Person {
        name: "John Doe".to_string(),
        age: 35,
        email: "john.doe@example.com".to_string(),
    }
}

fn create_address(num: usize) -> Address {
    Address {
        street: format!("{} Main Street", num),
        city: "Springfield".to_string(),
        zip_code: format!("{:05}", num),
    }
}

fn bench_simple_struct(c: &mut Criterion) {
    let person = create_person();
    let mut group = c.benchmark_group("simple_struct");

    group.bench_function("smithy4rs", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            let serializer = JsonSerializer::new(&mut buf);
            person
                .serialize_with_schema(black_box(&PERSON_SCHEMA), serializer)
                .unwrap();
            black_box(&buf);
        });
    });

    group.bench_function("serde_json", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            buf.clear();
            serde_json::to_writer(&mut buf, black_box(&person)).unwrap();
            black_box(&buf);
        });
    });

    group.finish();
}

fn bench_vec_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_100_items");

    let addresses: Vec<Address> = (0..100).map(create_address).collect();

    // Smithy4rs
    let address_list_schema = Schema::list_builder(ShapeId::from("bench#AddressList"), traits![])
        .put_member("member", &ADDRESS_SCHEMA, traits![])
        .build();

    // Measure JSON size for throughput
    let mut buf = Vec::new();
    serde_json::to_writer(&mut buf, &addresses).unwrap();
    let json_size = buf.len();
    group.throughput(Throughput::Bytes(json_size as u64));

    group.bench_function("smithy4rs", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            let serializer = JsonSerializer::new(&mut buf);
            addresses
                .serialize_with_schema(black_box(&address_list_schema), serializer)
                .unwrap();
            black_box(&buf);
        });
    });

    group.bench_function("serde_json", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            buf.clear();
            serde_json::to_writer(&mut buf, black_box(&addresses)).unwrap();
            black_box(&buf);
        });
    });

    group.finish();
}

fn bench_string_heavy(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_heavy");

    // Create a struct with many string fields
    #[derive(serde::Serialize)]
    struct StringHeavy {
        field1: String,
        field2: String,
        field3: String,
        field4: String,
        field5: String,
        field6: String,
        field7: String,
        field8: String,
        field9: String,
        field10: String,
    }

    let data = StringHeavy {
        field1: "The quick brown fox jumps over the lazy dog".to_string(),
        field2: "Lorem ipsum dolor sit amet, consectetur adipiscing elit".to_string(),
        field3: "Pack my box with five dozen liquor jugs".to_string(),
        field4: "How vexingly quick daft zebras jump!".to_string(),
        field5: "Sphinx of black quartz, judge my vow".to_string(),
        field6: "The five boxing wizards jump quickly".to_string(),
        field7: "Jackdaws love my big sphinx of quartz".to_string(),
        field8: "Mr Jock, TV quiz PhD, bags few lynx".to_string(),
        field9: "Waltz, bad nymph, for quick jigs vex".to_string(),
        field10: "Glib jocks quiz nymph to vex dwarf".to_string(),
    };

    // Measure JSON size
    let mut buf = Vec::new();
    serde_json::to_writer(&mut buf, &data).unwrap();
    let json_size = buf.len();
    group.throughput(Throughput::Bytes(json_size as u64));

    group.bench_function("serde_json", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            buf.clear();
            serde_json::to_writer(&mut buf, black_box(&data)).unwrap();
            black_box(&buf);
        });
    });

    group.finish();
}

fn bench_numbers(c: &mut Criterion) {
    let mut group = c.benchmark_group("numbers");

    #[derive(serde::Serialize)]
    struct Numbers {
        int1: i32,
        int2: i32,
        int3: i32,
        int4: i32,
        int5: i32,
        float1: f64,
        float2: f64,
        float3: f64,
        float4: f64,
        float5: f64,
    }

    let data = Numbers {
        int1: 42,
        int2: -1234567,
        int3: 999999999,
        int4: 0,
        int5: 12345,
        float1: 3.14159265,
        float2: -2.71828,
        float3: 1.414213,
        float4: 0.0,
        float5: 299792458.0,
    };

    // Measure JSON size
    let mut buf = Vec::new();
    serde_json::to_writer(&mut buf, &data).unwrap();
    let json_size = buf.len();
    group.throughput(Throughput::Bytes(json_size as u64));

    group.bench_function("serde_json", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            buf.clear();
            serde_json::to_writer(&mut buf, black_box(&data)).unwrap();
            black_box(&buf);
        });
    });

    group.finish();
}

fn bench_map_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_100_entries");

    let mut map = IndexMap::new();
    for i in 0..100 {
        map.insert(format!("key{}", i), create_address(i));
    }

    // Smithy4rs
    let address_map_schema = Schema::map_builder(ShapeId::from("bench#AddressMap"), traits![])
        .put_member("key", &STRING, traits![])
        .put_member("value", &ADDRESS_SCHEMA, traits![])
        .build();

    // Measure JSON size
    let mut buf = Vec::new();
    serde_json::to_writer(&mut buf, &map).unwrap();
    let json_size = buf.len();
    group.throughput(Throughput::Bytes(json_size as u64));

    group.bench_function("smithy4rs", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            let serializer = JsonSerializer::new(&mut buf);
            map.serialize_with_schema(black_box(&address_map_schema), serializer)
                .unwrap();
            black_box(&buf);
        });
    });

    group.bench_function("serde_json", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            buf.clear();
            serde_json::to_writer(&mut buf, black_box(&map)).unwrap();
            black_box(&buf);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_struct,
    bench_vec_serialization,
    bench_map_serialization,
    bench_string_heavy,
    bench_numbers,
);
criterion_main!(benches);
