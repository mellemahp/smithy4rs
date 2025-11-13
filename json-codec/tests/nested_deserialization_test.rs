use indexmap::IndexMap;
use smithy4rs_core::{
    lazy_schema,
    prelude::*,
    schema::{Schema, SchemaBuilder, SchemaRef, ShapeId},
    serde::deserializers::Deserialize,
    traits,
};
use smithy4rs_core_derive::DeserializableStruct;
use smithy4rs_json_codec::JsonDeserializer;
use std::sync::{Arc, LazyLock};

// Inner struct schema
lazy_schema!(
    ADDRESS_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#Address"), traits![]),
    (STREET, "street", STRING, traits![]),
    (CITY, "city", STRING, traits![]),
    (ZIP_CODE, "zip_code", STRING, traits![])
);

#[derive(Debug, PartialEq, DeserializableStruct)]
#[smithy_schema(ADDRESS_SCHEMA)]
struct Address {
    #[smithy_schema(STREET)]
    street: String,
    #[smithy_schema(CITY)]
    city: String,
    #[smithy_schema(ZIP_CODE)]
    zip_code: String,
}

// List of addresses
pub static ADDRESS_LIST_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::list_builder(ShapeId::from("test#AddressList"), traits![])
        .put_member("member", &ADDRESS_SCHEMA, traits![])
        .build()
});

// Map with address values
pub static ADDRESS_MAP_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::map_builder(ShapeId::from("test#AddressMap"), traits![])
        .put_member("key", &STRING, traits![])
        .put_member("value", &ADDRESS_SCHEMA, traits![])
        .build()
});

// Person struct with nested fields
lazy_schema!(
    PERSON_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#Person"), traits![]),
    (NAME, "name", STRING, traits![]),
    (AGE, "age", INTEGER, traits![]),
    (
        PRIMARY_ADDRESS,
        "primary_address",
        ADDRESS_SCHEMA,
        traits![]
    ),
    (
        SECONDARY_ADDRESS,
        "secondary_address",
        ADDRESS_SCHEMA,
        traits![]
    ),
    (
        ALL_ADDRESSES,
        "all_addresses",
        ADDRESS_LIST_SCHEMA,
        traits![]
    ),
    (
        NAMED_ADDRESSES,
        "named_addresses",
        ADDRESS_MAP_SCHEMA,
        traits![]
    )
);

#[derive(Debug, PartialEq, DeserializableStruct)]
#[smithy_schema(PERSON_SCHEMA)]
struct Person {
    #[smithy_schema(NAME)]
    name: String,
    #[smithy_schema(AGE)]
    age: i32,
    #[smithy_schema(PRIMARY_ADDRESS)]
    primary_address: Address,
    #[smithy_schema(SECONDARY_ADDRESS)]
    secondary_address: Option<Address>,
    #[smithy_schema(ALL_ADDRESSES)]
    all_addresses: Vec<Address>,
    #[smithy_schema(NAMED_ADDRESSES)]
    named_addresses: IndexMap<String, Address>,
}

// Recursive organization structure
pub static ORG_BUILDER: LazyLock<Arc<SchemaBuilder>> = LazyLock::new(|| {
    Arc::new(Schema::structure_builder(
        ShapeId::from("test#Organization"),
        traits![],
    ))
});

pub static ORG_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    ORG_BUILDER
        .put_member("name", &STRING, traits![])
        .put_member("department", &STRING, traits![])
        .put_member("parent", &*ORG_BUILDER, traits![])
        .build()
});

static ORG_NAME: LazyLock<&SchemaRef> = LazyLock::new(|| ORG_SCHEMA.expect_member("name"));
static ORG_DEPARTMENT: LazyLock<&SchemaRef> =
    LazyLock::new(|| ORG_SCHEMA.expect_member("department"));
static ORG_PARENT: LazyLock<&SchemaRef> = LazyLock::new(|| ORG_SCHEMA.expect_member("parent"));

#[derive(Debug, PartialEq, DeserializableStruct)]
#[smithy_schema(ORG_SCHEMA)]
struct Organization {
    #[smithy_schema(ORG_NAME)]
    name: String,
    #[smithy_schema(ORG_DEPARTMENT)]
    department: String,
    #[smithy_schema(ORG_PARENT)]
    parent: Option<Box<Organization>>,
}

#[test]
fn test_nested_struct_deserialization() {
    // JSON matching the exact same structure as the serialization test
    let json = r#"{
        "name": "John Doe",
        "age": 35,
        "primary_address": {
            "street": "123 Main St",
            "city": "Springfield",
            "zip_code": "12345"
        },
        "secondary_address": {
            "street": "456 Oak Ave",
            "city": "Shelbyville",
            "zip_code": "67890"
        },
        "all_addresses": [
            {
                "street": "789 Elm St",
                "city": "Capital City",
                "zip_code": "11111"
            },
            {
                "street": "321 Pine Rd",
                "city": "Ogdenville",
                "zip_code": "22222"
            }
        ],
        "named_addresses": {
            "vacation": {
                "street": "999 Beach Blvd",
                "city": "Coastal Town",
                "zip_code": "33333"
            },
            "office": {
                "street": "111 Business Park",
                "city": "Downtown",
                "zip_code": "44444"
            }
        }
    }"#;

    let mut deserializer = JsonDeserializer::from_str(json);
    let person = Person::deserialize(&PERSON_SCHEMA, &mut deserializer).unwrap();

    println!("Deserialized nested person: {:?}", person);

    // Verify structure - matching the serialization test assertions
    assert_eq!(person.name, "John Doe");
    assert_eq!(person.age, 35);

    // Primary address
    assert_eq!(person.primary_address.street, "123 Main St");
    assert_eq!(person.primary_address.city, "Springfield");
    assert_eq!(person.primary_address.zip_code, "12345");

    // Secondary address
    assert!(person.secondary_address.is_some());
    let secondary = person.secondary_address.as_ref().unwrap();
    assert_eq!(secondary.street, "456 Oak Ave");
    assert_eq!(secondary.city, "Shelbyville");
    assert_eq!(secondary.zip_code, "67890");

    // All addresses
    assert_eq!(person.all_addresses.len(), 2);
    assert_eq!(person.all_addresses[0].street, "789 Elm St");
    assert_eq!(person.all_addresses[0].city, "Capital City");
    assert_eq!(person.all_addresses[0].zip_code, "11111");
    assert_eq!(person.all_addresses[1].street, "321 Pine Rd");
    assert_eq!(person.all_addresses[1].city, "Ogdenville");
    assert_eq!(person.all_addresses[1].zip_code, "22222");

    // Named addresses
    assert_eq!(person.named_addresses.len(), 2);
    let vacation = person.named_addresses.get("vacation").unwrap();
    assert_eq!(vacation.street, "999 Beach Blvd");
    assert_eq!(vacation.city, "Coastal Town");
    assert_eq!(vacation.zip_code, "33333");
    let office = person.named_addresses.get("office").unwrap();
    assert_eq!(office.street, "111 Business Park");
    assert_eq!(office.city, "Downtown");
    assert_eq!(office.zip_code, "44444");
}

#[test]
fn test_recursive_struct_deserialization() {
    // JSON matching the exact recursive structure from serialization test
    let json = r#"{
        "name": "Backend Team",
        "department": "Software Engineering",
        "parent": {
            "name": "Engineering Division",
            "department": "Technology",
            "parent": {
                "name": "Acme Corp",
                "department": "Executive",
                "parent": null
            }
        }
    }"#;

    let mut deserializer = JsonDeserializer::from_str(json);
    let child = Organization::deserialize(&ORG_SCHEMA, &mut deserializer).unwrap();

    println!("Deserialized recursive organization: {:?}", child);

    // Verify recursive structure - matching the serialization test assertions
    assert_eq!(child.name, "Backend Team");
    assert_eq!(child.department, "Software Engineering");

    // Parent level
    assert!(child.parent.is_some());
    let parent = child.parent.as_ref().unwrap();
    assert_eq!(parent.name, "Engineering Division");
    assert_eq!(parent.department, "Technology");

    // Grandparent level
    assert!(parent.parent.is_some());
    let grandparent = parent.parent.as_ref().unwrap();
    assert_eq!(grandparent.name, "Acme Corp");
    assert_eq!(grandparent.department, "Executive");
    assert!(grandparent.parent.is_none());
}

#[test]
fn test_deeply_nested_without_recursion() {
    // JSON matching the exact deeply nested structure from serialization test
    let json = r#"{
        "name": "Deeply Nested",
        "age": 42,
        "primary_address": {
            "street": "Level 1",
            "city": "L1 City",
            "zip_code": "11111"
        },
        "secondary_address": null,
        "all_addresses": [
            {
                "street": "Level 2 Array Item 1",
                "city": "L2 City",
                "zip_code": "22222"
            },
            {
                "street": "Level 2 Array Item 2",
                "city": "L2 City",
                "zip_code": "22223"
            }
        ],
        "named_addresses": {
            "inner_key": {
                "street": "Deep Street",
                "city": "Nested City",
                "zip_code": "99999"
            }
        }
    }"#;

    let mut deserializer = JsonDeserializer::from_str(json);
    let person = Person::deserialize(&PERSON_SCHEMA, &mut deserializer).unwrap();

    println!("Deserialized deeply nested person: {:?}", person);

    // Verify all levels are present - matching the serialization test assertions
    assert_eq!(person.name, "Deeply Nested");
    assert_eq!(person.age, 42);
    assert_eq!(person.primary_address.street, "Level 1");
    assert_eq!(person.primary_address.city, "L1 City");
    assert_eq!(person.primary_address.zip_code, "11111");

    assert!(person.secondary_address.is_none());

    assert_eq!(person.all_addresses.len(), 2);
    assert_eq!(person.all_addresses[0].street, "Level 2 Array Item 1");
    assert_eq!(person.all_addresses[0].city, "L2 City");
    assert_eq!(person.all_addresses[0].zip_code, "22222");
    assert_eq!(person.all_addresses[1].street, "Level 2 Array Item 2");
    assert_eq!(person.all_addresses[1].city, "L2 City");
    assert_eq!(person.all_addresses[1].zip_code, "22223");

    assert_eq!(person.named_addresses.len(), 1);
    let inner = person.named_addresses.get("inner_key").unwrap();
    assert_eq!(inner.street, "Deep Street");
    assert_eq!(inner.city, "Nested City");
    assert_eq!(inner.zip_code, "99999");
}
