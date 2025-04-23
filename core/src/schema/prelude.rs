#![allow(dead_code)]

use std::sync::LazyLock;
use crate::schema::Schema;
use crate::shapes::{ShapeId, ShapeType};

pub static BLOB: LazyLock<Schema> = LazyLock::new(|| Schema::create_blob(ShapeId::from("smithy.api#Blob")));
pub static BOOLEAN: LazyLock<Schema> = LazyLock::new(|| Schema::create_boolean(ShapeId::from("smithy.api#Boolean")));
pub static STRING: LazyLock<Schema> = LazyLock::new(|| Schema::create_string(ShapeId::from("smithy.api#String")));
pub static TIMESTAMP: LazyLock<Schema> = LazyLock::new(|| Schema::create_timestamp(ShapeId::from("smithy.api#Timestamp")));
pub static BYTE: LazyLock<Schema> = LazyLock::new(|| Schema::create_byte(ShapeId::from("smithy.api#Byte")));
pub static SHORT: LazyLock<Schema> = LazyLock::new(|| Schema::create_short(ShapeId::from("smithy.api#Short")));
pub static  INTEGER: LazyLock<Schema> = LazyLock::new(|| Schema::create_integer(ShapeId::from("smithy.api#Integer")));
pub static LONG: LazyLock<Schema> = LazyLock::new(|| Schema::create_long(ShapeId::from("smithy.api#Long")));
pub static FLOAT: LazyLock<Schema> = LazyLock::new(|| Schema::create_float(ShapeId::from("smithy.api#Float")));
pub static DOUBLE: LazyLock<Schema> = LazyLock::new(|| Schema::create_double(ShapeId::from("smithy.api#Double")));
pub static BIG_INTEGER: LazyLock<Schema> = LazyLock::new(|| Schema::create_big_integer(ShapeId::from("smithy.api#BigInteger")));
pub static BIG_DECIMAL: LazyLock<Schema> = LazyLock::new(|| Schema::create_big_decimal(ShapeId::from("smithy.api#BigDecimal")));
pub static DOCUMENT: LazyLock<Schema> = LazyLock::new(|| Schema::create_document(ShapeId::from("smithy.api#Document")));

// TODO:
// - Primitive types

///  Returns the most appropriate prelude schema reference based on the given type.
///
/// Types with no corresponding prelude schema (e.g., LIST, STRUCTURE, UNION) are returned
/// as document schemas.
///
/// *NOTE*: Primitive numbers and boolean types return the nullable value type.

pub fn get_schema_for_type(shape_type: ShapeType) -> &'static Schema<'static> {
    match shape_type {
        ShapeType::Blob => &BLOB,
        ShapeType::Byte => &*BYTE,
        ShapeType::Boolean => &*BOOLEAN,
        ShapeType::String | ShapeType::Enum => &*STRING,
        ShapeType::Timestamp => &*TIMESTAMP,
        ShapeType::Short => &*SHORT,
        ShapeType::Integer | ShapeType::IntEnum => &*INTEGER,
        ShapeType::Long => &*LONG,
        ShapeType::Float => &*FLOAT,
        ShapeType::Double => &*DOUBLE,
        ShapeType::BigInteger => &*BIG_INTEGER,
        ShapeType::BigDecimal => &*BIG_DECIMAL,
        _ => &*DOCUMENT
    }
}