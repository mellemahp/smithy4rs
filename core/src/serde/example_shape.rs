#![allow(unused_variables, dead_code)]

use indexmap::IndexMap;
use serde::Deserialize;

use crate::{
    schema::SchemaRef,
    serde::{
        builders::{Builder, ShapeBuilder, StructOrBuilder},
        de::{Deserializer, Error, deserialize},
        validation::{ValidationErrors, Validator},
    },
};

struct Example {
    string: String,
    optional_string: Option<String>,
    number: i32,
    structure: A,
    optional_struct: Option<B>,
    vec_string: Vec<String>,
    vec_struct: Vec<A>,
    map_string: IndexMap<String, String>,
    map_struct: IndexMap<String, B>,
}
impl Builder for Example {
    type Builder = ExampleBuilder;
}
struct ExampleBuilder {
    string: Option<String>,
    optional_string: Option<String>,
    number: Option<i32>,
    structure: Option<StructOrBuilder<A>>,
    optional_struct: Option<StructOrBuilder<B>>,
    vec_string: Option<Vec<String>>,
    vec_struct: Option<Vec<StructOrBuilder<A>>>,
    map_string: Option<IndexMap<String, String>>,
    map_struct: Option<IndexMap<String, StructOrBuilder<B>>>,
}
impl ExampleBuilder {
    pub fn string(mut self, value: String) -> Self {
        self.set_string(value);
        self
    }
    fn set_string(&mut self, value: String) {
        self.string = Some(value);
    }

    pub fn optional_string(mut self, value: String) -> Self {
        self.set_optional_string(value);
        self
    }
    fn set_optional_string(&mut self, value: String) {
        self.optional_string = Some(value);
    }

    pub fn number(mut self, value: i32) -> Self {
        self.set_number(value);
        self
    }
    fn set_number(&mut self, value: i32) {
        self.number = Some(value);
    }

    pub fn structure(mut self, value: A) -> Self {
        self.set_structure(value.into());
        self
    }
    fn set_structure(&mut self, value: StructOrBuilder<A>) {
        self.structure = Some(value);
    }

    // TODO: Should optional setters allow for Optional input?
    //       this allows [`toBuilder`] calls to reset an optional field to None
    pub fn optional_structure(mut self, value: B) -> Self {
        self.set_optional_structure(value.into());
        self
    }
    fn set_optional_structure(&mut self, value: StructOrBuilder<B>) {
        self.optional_struct = Some(value);
    }

    pub fn vec_string(mut self, value: Vec<String>) -> Self {
        self.set_vec_string(value);
        self
    }
    fn set_vec_string(&mut self, value: Vec<String>) {
        self.vec_string = Some(value);
    }

    pub fn vec_structure(mut self, value: Vec<A>) -> Self {
        self.set_vec_structure(value.into_iter().map(|a| a.into()).collect());
        self
    }
    fn set_vec_structure(&mut self, value: Vec<StructOrBuilder<A>>) {
        self.vec_struct = Some(value)
    }

    pub fn map_string(mut self, value: IndexMap<String, String>) -> Self {
        self.set_map_string(value);
        self
    }
    fn set_map_string(&mut self, value: IndexMap<String, String>) {
        self.map_string = Some(value);
    }

    pub fn map_structure(mut self, value: IndexMap<String, B>) -> Self {
        self.set_map_struct(value.into_iter().map(|(k, v)| (k, v.into())).collect());
        self
    }
    fn set_map_struct(&mut self, value: IndexMap<String, StructOrBuilder<B>>) {
        self.map_struct = Some(value)
    }
}
impl ShapeBuilder<Example> for ExampleBuilder {
    fn new() -> Self {
        ExampleBuilder {
            string: None,
            optional_string: None,
            number: None,
            structure: None,
            optional_struct: None,
            vec_string: None,
            vec_struct: None,
            map_string: None,
            map_struct: None,
        }
    }

    fn deserialize_member<'de, D: Deserializer<'de>>(
        &mut self,
        member_schema: &SchemaRef,
        deserializer: D,
    ) -> Result<(), D::Error> {
        let Some(index) = member_schema.member_index() else {
            return Err(Error::custom("Expected member schema."));
        };
        // TODO: We might also want to type-check the member-target schemas provided to the builder.
        match index {
            0 => self.set_string(deserialize(member_schema, deserializer)?),
            1 => self.set_optional_string(deserialize(member_schema, deserializer)?),
            2 => self.set_number(deserialize(member_schema, deserializer)?),
            3 => self.set_structure(deserialize(member_schema, deserializer)?),
            4 => self.set_optional_structure(deserialize(member_schema, deserializer)?),
            5 => self.set_vec_string(deserialize(member_schema, deserializer)?),
            6 => self.set_vec_structure(deserialize(member_schema, deserializer)?),
            7 => self.set_map_string(deserialize(member_schema, deserializer)?),
            8 => self.set_map_struct(deserialize(member_schema, deserializer)?),
            _ => return Err(Error::custom("Unexpected Member")),
        }
        Ok(())
    }

    fn build_with_validator<V: Validator>(self, validator: V) -> Result<Example, ValidationErrors> {
        todo!()
    }
}

struct A {
    field_a: String,
}
impl Builder for A {
    type Builder = ABuilder;
}
struct ABuilder {
    field_a: Option<String>,
}
impl ABuilder {
    pub fn field_a(mut self, value: String) -> Self {
        self.set_field_a(value);
        self
    }
    fn set_field_a(&mut self, value: String) {
        self.field_a = Some(value);
    }
}
impl ShapeBuilder<A> for ABuilder {
    fn new() -> Self {
        ABuilder { field_a: None }
    }

    fn deserialize_member<'de, D: Deserializer<'de>>(
        &mut self,
        member_schema: &SchemaRef,
        deserializer: D,
    ) -> Result<(), D::Error> {
        let Some(index) = member_schema.member_index() else {
            return Err(Error::custom("Expected member schema."));
        };
        match index {
            0 => self.set_field_a(deserialize(member_schema, deserializer)?),
            _ => {
                return Err(Error::custom("Expected member schema."));
            }
        }
        Ok(())
    }

    fn build_with_validator<V: Validator>(self, validator: V) -> Result<A, ValidationErrors> {
        todo!()
    }
}

#[derive(Deserialize)]
struct B {
    field_b: String,
}
impl Builder for B {
    type Builder = BBuilder;
}
struct BBuilder {
    field_b: Option<String>,
}
impl BBuilder {
    pub fn field_b(mut self, value: String) -> Self {
        self.set_field_b(value);
        self
    }
    fn set_field_b(&mut self, value: String) {
        self.field_b = Some(value);
    }
}
impl ShapeBuilder<B> for BBuilder {
    fn new() -> Self {
        BBuilder { field_b: None }
    }

    fn deserialize_member<'de, D: Deserializer<'de>>(
        &mut self,
        member_schema: &SchemaRef,
        deserializer: D,
    ) -> Result<(), D::Error> {
        let Some(index) = member_schema.member_index() else {
            return Err(Error::custom("Expected member schema."));
        };
        match index {
            0 => self.set_field_b(deserialize(member_schema, deserializer)?),
            _ => {
                return Err(Error::custom("Expected member schema."));
            }
        }
        Ok(())
    }

    fn build_with_validator<V: Validator>(self, validator: V) -> Result<B, ValidationErrors> {
        todo!()
    }
}
