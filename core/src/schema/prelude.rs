#![allow(dead_code)]

use crate::schema::Schema;
use crate::traits;
use std::sync::LazyLock;

pub static BLOB: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_blob("smithy.api#Blob", traits![]));
pub static BOOLEAN: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_boolean("smithy.api#Boolean", traits![]));
pub static STRING: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_string("smithy.api#String", traits![]));
pub static TIMESTAMP: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_timestamp("smithy.api#Timestamp", traits![]));
pub static BYTE: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_byte("smithy.api#Byte", traits![]));
pub static SHORT: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_short("smithy.api#Short", traits![]));
pub static INTEGER: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_integer("smithy.api#Integer", traits![]));
pub static LONG: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_long("smithy.api#Long", traits![]));
pub static FLOAT: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_float("smithy.api#Float", traits![]));
pub static DOUBLE: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_double("smithy.api#Double", traits![]));
pub static BIG_INTEGER: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_big_integer("smithy.api#BigInteger", traits![]));
pub static BIG_DECIMAL: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_big_decimal("smithy.api#BigDecimal", traits![]));
pub static DOCUMENT: LazyLock<Schema> =
    LazyLock::new(|| Schema::create_document("smithy.api#Document", traits![]));

// TODO:
// - Primitive types
