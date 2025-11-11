use indexmap::IndexMap;
use smithy4rs_core::{
    lazy_schema,
    prelude::*,
    schema::{Schema, SchemaBuilder, SchemaRef, ShapeId},
    serde::serializers::SerializeWithSchema,
    traits,
};
use smithy4rs_core_derive::SerializableStruct;
use smithy4rs_json_codec::JsonSerializer;
use std::sync::{Arc, LazyLock};

// Inner struct schema
lazy_schema!(
    ADDRESS_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#Address"), traits![]),
    (STREET, "street", STRING, traits![]),
    (CITY, "city", STRING, traits![]),
    (ZIP_CODE, "zip_code", STRING, traits![])
);

#[derive(SerializableStruct)]
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
    (PRIMARY_ADDRESS, "primary_address", ADDRESS_SCHEMA, traits![]),
    (SECONDARY_ADDRESS, "secondary_address", ADDRESS_SCHEMA, traits![]),
    (ALL_ADDRESSES, "all_addresses", ADDRESS_LIST_SCHEMA, traits![]),
    (NAMED_ADDRESSES, "named_addresses", ADDRESS_MAP_SCHEMA, traits![])
);

#[derive(SerializableStruct)]
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
    Arc::new(Schema::structure_builder(ShapeId::from("test#Organization"), traits![]))
});

pub static ORG_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    ORG_BUILDER
        .put_member("name", &STRING, traits![])
        .put_member("department", &STRING, traits![])
        .put_member("parent", &*ORG_BUILDER, traits![])
        .build()
});

static ORG_NAME: LazyLock<&SchemaRef> = LazyLock::new(|| ORG_SCHEMA.expect_member("name"));
static ORG_DEPARTMENT: LazyLock<&SchemaRef> = LazyLock::new(|| ORG_SCHEMA.expect_member("department"));
static ORG_PARENT: LazyLock<&SchemaRef> = LazyLock::new(|| ORG_SCHEMA.expect_member("parent"));

#[derive(SerializableStruct)]
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
fn test_nested_struct_serialization() {
    let primary = Address {
        street: "123 Main St".to_string(),
        city: "Springfield".to_string(),
        zip_code: "12345".to_string(),
    };

    let secondary = Address {
        street: "456 Oak Ave".to_string(),
        city: "Shelbyville".to_string(),
        zip_code: "67890".to_string(),
    };

    let mut all_addresses = Vec::new();
    all_addresses.push(Address {
        street: "789 Elm St".to_string(),
        city: "Capital City".to_string(),
        zip_code: "11111".to_string(),
    });
    all_addresses.push(Address {
        street: "321 Pine Rd".to_string(),
        city: "Ogdenville".to_string(),
        zip_code: "22222".to_string(),
    });

    let mut named_addresses = IndexMap::new();
    named_addresses.insert(
        "vacation".to_string(),
        Address {
            street: "999 Beach Blvd".to_string(),
            city: "Coastal Town".to_string(),
            zip_code: "33333".to_string(),
        },
    );
    named_addresses.insert(
        "office".to_string(),
        Address {
            street: "111 Business Park".to_string(),
            city: "Downtown".to_string(),
            zip_code: "44444".to_string(),
        },
    );

    let person = Person {
        name: "John Doe".to_string(),
        age: 35,
        primary_address: primary,
        secondary_address: Some(secondary),
        all_addresses,
        named_addresses,
    };

    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    person
        .serialize_with_schema(&PERSON_SCHEMA, serializer)
        .unwrap();

    let json = String::from_utf8(buf).unwrap();
    println!("Serialized nested person JSON:\n{}", json);

    // Verify structure
    assert!(json.contains("\"name\":\"John Doe\""));
    assert!(json.contains("\"age\":35"));
    assert!(json.contains("\"primary_address\""));
    assert!(json.contains("\"123 Main St\""));
    assert!(json.contains("\"Springfield\""));
    assert!(json.contains("\"secondary_address\""));
    assert!(json.contains("\"456 Oak Ave\""));
    assert!(json.contains("\"all_addresses\""));
    assert!(json.contains("\"789 Elm St\""));
    assert!(json.contains("\"named_addresses\""));
    assert!(json.contains("\"vacation\""));
    assert!(json.contains("\"999 Beach Blvd\""));
}

#[test]
fn test_recursive_struct_serialization() {
    let grandparent = Organization {
        name: "Acme Corp".to_string(),
        department: "Executive".to_string(),
        parent: None,
    };

    let parent = Organization {
        name: "Engineering Division".to_string(),
        department: "Technology".to_string(),
        parent: Some(Box::new(grandparent)),
    };

    let child = Organization {
        name: "Backend Team".to_string(),
        department: "Software Engineering".to_string(),
        parent: Some(Box::new(parent)),
    };

    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    child
        .serialize_with_schema(&ORG_SCHEMA, serializer)
        .unwrap();

    let json = String::from_utf8(buf).unwrap();
    println!("Serialized recursive organization JSON:\n{}", json);

    // Verify recursive structure
    assert!(json.contains("\"Backend Team\""));
    assert!(json.contains("\"Software Engineering\""));
    assert!(json.contains("\"Engineering Division\""));
    assert!(json.contains("\"Technology\""));
    assert!(json.contains("\"Acme Corp\""));
    assert!(json.contains("\"Executive\""));

    // Count the nesting levels by counting "parent" occurrences
    let parent_count = json.matches("\"parent\"").count();
    assert_eq!(parent_count, 2, "Should have 2 parent references (child->parent->grandparent)");
}

#[test]
fn test_deeply_nested_without_recursion() {
    // Create a structure with many levels of nesting using different types
    let mut inner_map = IndexMap::new();
    inner_map.insert(
        "inner_key".to_string(),
        Address {
            street: "Deep Street".to_string(),
            city: "Nested City".to_string(),
            zip_code: "99999".to_string(),
        },
    );

    let person = Person {
        name: "Deeply Nested".to_string(),
        age: 42,
        primary_address: Address {
            street: "Level 1".to_string(),
            city: "L1 City".to_string(),
            zip_code: "11111".to_string(),
        },
        secondary_address: None,
        all_addresses: vec![
            Address {
                street: "Level 2 Array Item 1".to_string(),
                city: "L2 City".to_string(),
                zip_code: "22222".to_string(),
            },
            Address {
                street: "Level 2 Array Item 2".to_string(),
                city: "L2 City".to_string(),
                zip_code: "22223".to_string(),
            },
        ],
        named_addresses: inner_map,
    };

    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    person
        .serialize_with_schema(&PERSON_SCHEMA, serializer)
        .unwrap();

    let json = String::from_utf8(buf).unwrap();
    println!("Serialized deeply nested JSON:\n{}", json);

    // Verify all levels are present
    assert!(json.contains("\"Deeply Nested\""));
    assert!(json.contains("\"Level 1\""));
    assert!(json.contains("\"Level 2 Array Item 1\""));
    assert!(json.contains("\"Level 2 Array Item 2\""));
    assert!(json.contains("\"inner_key\""));
    assert!(json.contains("\"Deep Street\""));
}
