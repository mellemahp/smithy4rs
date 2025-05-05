mod deserialization;
mod errors;
mod serialization;

pub use deserialization::*;
pub use serialization::*;

extern crate smithy4rs_core;

use smithy4rs_core::schema::Schema;

fn get_member_name<'s>(schema: &'s Schema) -> &'s str {
    schema
        .member_name
        .as_ref()
        .expect("Should have a member name")
}
