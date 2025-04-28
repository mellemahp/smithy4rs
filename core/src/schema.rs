#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashSet;
use indexmap::IndexMap;
use crate::shapes::{ShapeId, ShapeType};

// TODO: Support traits
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema<'s> {
    pub id: ShapeId,
    pub shape_type: ShapeType,
    pub members: Option<IndexMap<String, Schema<'s>>>,
    pub member_target: Option<&'s Schema<'s>>,
    pub member_name: Option<String>,
    pub member_index: Option<usize>,
    // pub traits: Option<String>,
}

// TODO: Support traits

// FACTORY METHODS
// TODO: What should be inlined?
impl Schema<'_> {
    fn root_schema(shape_type: ShapeType, id: ShapeId) -> Self {
        Schema {
            id,
            shape_type,
            members: None,
            member_target: None,
            member_name: None,
            member_index: None,
        }
    }

    pub fn create_boolean(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Boolean, id)
    }

    pub fn create_byte(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Byte, id)
    }

    pub fn create_short(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Short, id)
    }

    pub fn create_integer(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Integer, id)
    }

    pub fn create_int_enum(id: ShapeId, values: HashSet<i32>) -> Self {
        todo!()
    }

    pub fn create_long(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Long, id)
    }

    pub fn create_float(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Float, id)
    }

    pub fn create_double(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Double, id)
    }

    pub fn create_big_integer(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::BigInteger, id)
    }

    pub fn create_big_decimal(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::BigDecimal, id)
    }

    pub fn create_string(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::String, id)
    }

    pub fn create_enum(id: ShapeId, values: HashSet<String>) -> Self {
        todo!()
    }

    pub fn create_blob(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Blob, id)
    }

    pub fn create_document(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Document, id)
    }

    pub fn create_timestamp(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Timestamp, id)
    }

    pub fn create_operation(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Operation, id)
    }

    pub fn create_resource(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Resource, id)
    }

    pub fn create_service(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::Service, id)
    }
}

// GETTERS
impl<'s> Schema<'s> {
    pub fn get_member(&self, id: &str) -> Option<&'s Schema> {
        // TODO: probably a better way
        self.members.as_ref().map(|m| m.get(id))?
    }

    pub fn expect_member(&self, id: &str) -> &'s Schema {
        self.members.as_ref().map(|m| m.get(id)).unwrap().unwrap()
    }

    pub fn is_member(&self) -> bool {
        self.member_target.is_some()
    }
}

// BUILDER FACTORIES
impl Schema<'_> {
    pub fn structure_builder<'s>(id: ShapeId) -> SchemaBuilder<'s> {
        SchemaBuilder::new(id, ShapeType::Structure)
    }

    pub fn union_builder<'s>(id: ShapeId) -> SchemaBuilder<'s> {
        SchemaBuilder::new(id, ShapeType::Union)
    }

    pub fn list_builder<'s>(id: ShapeId) -> SchemaBuilder<'s> {
        SchemaBuilder::new(id, ShapeType::List)
    }

    pub fn map_builder<'s>(id: ShapeId) -> SchemaBuilder<'s> {
        SchemaBuilder::new(id, ShapeType::Map)
    }
}


pub struct SchemaBuilder<'s> {
    id: ShapeId,
    shape_type: ShapeType,
    // pub traits: Option<String>,
    members: Vec<MemberSchemaBuilder<'s>>,
    member_target: Option<&'s Schema<'s>>,
    member_index: Option<usize>
}

impl SchemaBuilder<'_> {
    fn new(id: ShapeId, shape_type: ShapeType) -> Self {
        SchemaBuilder {
            id,
            members: match shape_type {
                ShapeType::List => Vec::with_capacity(1),
                ShapeType::Map => Vec::with_capacity(2),
                _ => Vec::new(),
            },
            shape_type,
            member_target: None,
            member_index: None,
        }
    }
}

impl <'b> SchemaBuilder<'b> {
    pub fn put_member<'t>(mut self, name: &str, target: &'t Schema) -> Self
    // Target reference will outlive this builder
    where 't: 'b {
        match self.shape_type {
            ShapeType::List => {
                if name != "member" {
                    // TODO: Real error
                    panic!("Lists can only have members named `member`. Found `{}`", name)
                }
            }
            ShapeType::Map => {
                if !(name == "key" || name == "value") {
                    panic!("Map can only have members named `key` or `value`")
                }
            }
            _ => { /* fall through otherwise */ }
        }
        self.members.push(MemberSchemaBuilder::new(name.into(), self.id.with_member(name), target));
        self
    }

    const fn sort_members(&mut self) {
        // TODO: Implement.
    }

    // TODO: does this need cloning?
    pub fn build(mut self) -> Schema<'b> {
        // Structure shapes need to sort members so that required members come before optional members.
        if self.shape_type == ShapeType::Structure {
            self.sort_members();
        }
        let mut member_map = IndexMap::with_capacity(self.members.len());
        // TODO: Could clone be avoided?
        for (idx, member_builder) in self.members.iter_mut().enumerate() {
            member_builder.set_index(idx);
            member_map.insert(member_builder.name.clone(), member_builder.build());
        }

        Schema {
            id: self.id.clone(),
            shape_type: self.shape_type.clone(),
            members: Some(member_map.clone()),
            member_target: None,
            member_name: None,
            member_index: None,
        }
    }
}



struct MemberSchemaBuilder<'s>{
    pub (super) name: String,
    id: ShapeId,
    member_target: &'s Schema<'s>,
    member_index: Option<usize>
}

impl <'b> MemberSchemaBuilder<'b> {
    pub(super) const fn new<'t>(name: String, id: ShapeId, target: &'t Schema<'_>) -> Self
    // Schema reference outlives this builder
    where 't: 'b
    {
        MemberSchemaBuilder {
            name,
            id,
            member_target: target,
            member_index: None,
        }
    }

    pub(super) const fn set_index(&mut self, index: usize) {
        self.member_index = Some(index);
    }

    pub (super) fn build(&self) -> Schema<'b> {
        // Schema outlives builder
        if self.member_index.is_none() {
            // TODO: real error
            panic!("Expected member index!");
        }
        Schema {
            id: self.id.clone(),
            shape_type: ShapeType::Member,
            members: Default::default(),
            member_target: Some(self.member_target),
            member_name: Some(self.name.clone()),
            member_index: self.member_index,
        }
    }
}