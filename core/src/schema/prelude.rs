#![allow(dead_code)]

use crate::schema::{Schema, SchemaRef};
use crate::traits;
use std::sync::LazyLock;

pub static BLOB: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_blob("smithy.api#Blob", traits![]));
pub static BOOLEAN: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_boolean("smithy.api#Boolean", traits![]));
pub static STRING: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_string("smithy.api#String", traits![]));
pub static TIMESTAMP: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_timestamp("smithy.api#Timestamp", traits![]));
pub static BYTE: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_byte("smithy.api#Byte", traits![]));
pub static SHORT: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_short("smithy.api#Short", traits![]));
pub static INTEGER: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_integer("smithy.api#Integer", traits![]));
pub static LONG: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_long("smithy.api#Long", traits![]));
pub static FLOAT: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_float("smithy.api#Float", traits![]));
pub static DOUBLE: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_double("smithy.api#Double", traits![]));
pub static BIG_INTEGER: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_big_integer("smithy.api#BigInteger", traits![]));
pub static BIG_DECIMAL: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_big_decimal("smithy.api#BigDecimal", traits![]));
pub static DOCUMENT: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_document("smithy.api#Document", traits![]));

// TODO:
// - Primitive types
