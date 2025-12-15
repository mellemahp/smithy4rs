use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use smithy4rs_core::{
    prelude::{HTTPChecksumRequiredTrait, HTTPQueryParamsTrait, HTTPQueryTrait},
    schema::StaticTraitId,
    smithy,
};

smithy!("com.example#Example": {
    @HTTPQueryTrait::new("foo");
    @HTTPQueryParamsTrait;
    @HTTPChecksumRequiredTrait;
    string TRAIT_SCHEMA
});

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
