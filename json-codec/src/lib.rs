mod serialization;
mod deserialization;
mod errors;

pub use serialization::*;
pub use deserialization::*;

extern crate smithy4rs_core;

use smithy4rs_core::schema::Schema;


fn get_member_name<'s>(schema: &'s Schema) -> &'s str {
    schema.member_name.as_ref().expect("Should have a member name")
}