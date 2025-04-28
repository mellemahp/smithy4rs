#![allow(dead_code)]

use std::convert::Into;
use std::sync::LazyLock;
use crate::schema::Schema;
use crate::shapes::ShapeType;

pub static BLOB: LazyLock<Schema> = LazyLock::new(|| Schema::create_blob("smithy.api#Blob".into()));
pub static BOOLEAN: LazyLock<Schema> = LazyLock::new(|| Schema::create_boolean("smithy.api#Boolean".into()));
pub static STRING: LazyLock<Schema> = LazyLock::new(|| Schema::create_string("smithy.api#String".into()));
pub static TIMESTAMP: LazyLock<Schema> = LazyLock::new(|| Schema::create_timestamp("smithy.api#Timestamp".into()));
pub static BYTE: LazyLock<Schema> = LazyLock::new(|| Schema::create_byte("smithy.api#Byte".into()));
pub static SHORT: LazyLock<Schema> = LazyLock::new(|| Schema::create_short("smithy.api#Short".into()));
pub static  INTEGER: LazyLock<Schema> = LazyLock::new(|| Schema::create_integer("smithy.api#Integer".into()));
pub static LONG: LazyLock<Schema> = LazyLock::new(|| Schema::create_long("smithy.api#Long".into()));
pub static FLOAT: LazyLock<Schema> = LazyLock::new(|| Schema::create_float("smithy.api#Float".into()));
pub static DOUBLE: LazyLock<Schema> = LazyLock::new(|| Schema::create_double("smithy.api#Double".into()));
pub static BIG_INTEGER: LazyLock<Schema> = LazyLock::new(|| Schema::create_big_integer("smithy.api#BigInteger".into()));
pub static BIG_DECIMAL: LazyLock<Schema> = LazyLock::new(|| Schema::create_big_decimal("smithy.api#BigDecimal".into()));
pub static DOCUMENT: LazyLock<Schema> = LazyLock::new(|| Schema::create_document("smithy.api#Document".into()));

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