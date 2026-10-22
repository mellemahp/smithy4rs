//! Tests across different core Smithy functionality such as
//! validation.
//!

// ================================================================
// Equality & Smoothing Tests
// ================================================================

mod equality {
    // TODO: Partial Equality checks?
}

// ================================================================
// Validation Tests
// ================================================================

mod validation {
    use indexmap::IndexMap;

    use crate::{
        features::documents::Document,
        prelude::{
            INTEGER, LengthTrait, PatternTrait, RangeTrait, RequiredTrait, STRING, UniqueItemsTrait,
        },
        serde::validation::{DefaultValidator, Validator},
        smithy,
    };

    smithy!(
        @PatternTrait::new("^[a-z]*$");
        string com::example::Lowercase
    );

    #[test]
    fn string_document_checked() {
        let string: Box<dyn Document> = "camelCased".into();
        let mut validator = DefaultValidator::new();
        let Err(err) = validator.validate(&LOWERCASE, &string) else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_enum = err.errors.first().unwrap();
        assert_eq!(
            error_enum.error.to_string(),
            "Value `camelCased` did not conform to expected pattern `^[a-z]*$`".to_string()
        );
    }

    smithy!(
        @RangeTrait::builder().min(2).max(5).build();
        integer com::example::BetweenTwoAndFive
    );

    #[test]
    fn number_document_checked() {
        let int: Box<dyn Document> = 6i32.into();
        let mut validator = DefaultValidator::new();
        let Err(err) = validator.validate(&BETWEEN_TWO_AND_FIVE, &int) else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_enum = err.errors.first().unwrap();
        assert_eq!(
            error_enum.error.to_string(),
            "Size: 6 does not conform to @range constraint. Expected between 2 and 5.".to_string()
        );
    }

    smithy!(
        @LengthTrait::builder().max(3).build();
        list com::example::LongList {
            member: STRING
        }
    );

    #[test]
    fn list_document_length_checked() {
        let list = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let mut validator = DefaultValidator::new();
        let Err(err) = validator.validate(&LONG_LIST, &list) else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_enum = err.errors.first().unwrap();
        assert_eq!(
            error_enum.error.to_string(),
            "Size: 4 does not conform to @length constraint. Expected between 0 and 3.".to_string()
        );
    }

    smithy!(
        @UniqueItemsTrait::builder().build();
        list com::example::ListSet {
            member: STRING
        }
    );

    #[test]
    fn list_document_uniqueness_checked() {
        let list = vec![
            "a".to_string(),
            "a".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let mut validator = DefaultValidator::new();
        let Err(err) = validator.validate(&LIST_SET, &list) else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_enum = err.errors.first().unwrap();
        assert_eq!(
            error_enum.error.to_string(),
            "Items in collection should be unique.".to_string()
        );
    }

    smithy!(
        @LengthTrait::builder().max(3).build();
        map com::example::LongMap {
            key: STRING
            value: INTEGER
        }
    );

    #[test]
    fn map_document_length_checked() {
        let mut map = IndexMap::new();
        map.insert("Stuff".to_string(), 1);
        map.insert("Things".to_string(), 2);
        map.insert("Quux".to_string(), 3);
        map.insert("Baz".to_string(), 4);

        let mut validator = DefaultValidator::new();
        let Err(err) = validator.validate(&LONG_MAP, &map) else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_enum = err.errors.first().unwrap();
        assert_eq!(
            error_enum.error.to_string(),
            "Size: 4 does not conform to @length constraint. Expected between 0 and 3.".to_string()
        );
    }

    smithy!(
        structure test::Nested {
            @RequiredTrait::builder().build();
            @PatternTrait::new("^[a-z]*$");
            c: STRING
        }
    );

    smithy!(
        structure test::StructWithNested {
            nested: NESTED
            @RequiredTrait::builder().build();
            required: NESTED
        }
    );

    #[test]
    fn structure_document_checked() {
        let mut nested: IndexMap<String, String> = IndexMap::new();
        nested.insert("c".to_string(), "dataWithCaps".to_string());

        let mut root = IndexMap::new();
        root.insert("nested".to_string(), nested);

        let document: Box<dyn Document> = root.into();
        let mut validator = DefaultValidator::new();

        let Err(err) = validator.validate(&STRUCT_WITH_NESTED, &document) else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 2);
        let error_required = err.errors.first().unwrap();
        assert_eq!(
            error_required.error.to_string(),
            "Field is Required.".to_string()
        );
        let error_pattern = err.errors.get(1).unwrap();
        assert_eq!(
            error_pattern.error.to_string(),
            "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string()
        );
    }
}

// TODO: Debugging
