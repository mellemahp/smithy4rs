use std::{hint::black_box, sync::LazyLock};

use criterion::{Criterion, criterion_group, criterion_main};
use smithy4rs_core::{
    lazy_schema,
    prelude::{HTTPChecksumRequiredTrait, HTTPQueryParamsTrait, HTTPQueryTrait},
    schema::{Schema, SchemaRef, StaticTraitId},
    traits,
};

lazy_schema!(
    TRAIT_SCHEMA,
    Schema::create_string(
        "com.example#Example",
        traits![
            HTTPQueryTrait::new("foo"),
            HTTPQueryParamsTrait,
            HTTPChecksumRequiredTrait
        ]
    )
);

pub fn trait_access_id(c: &mut Criterion) {
    c.bench_function("Trait Access By ID", |b| {
        b.iter(|| {
            let _ = black_box(&TRAIT_SCHEMA.get_trait(HTTPQueryParamsTrait::trait_id()));
        })
    });
}

pub fn trait_access_type(c: &mut Criterion) {
    c.bench_function("Trait Access By Type", |b| {
        b.iter(|| {
            let _ = black_box(&TRAIT_SCHEMA.get_trait_as::<HTTPQueryParamsTrait>());
        })
    });
}
criterion_group!(traits, trait_access_id, trait_access_type);

criterion_main!(traits);
