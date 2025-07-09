mod deserialization;
mod errors;
mod serialization;

pub use deserialization::*;
pub use serialization::*;

extern crate smithy4rs_core;

use smithy4rs_core::schema::{Schema, SchemaRef};

fn get_member_name(schema: &SchemaRef) -> &str {
    schema.as_member()
        .expect("Should be member schema")
        .name
        .as_ref()
}
