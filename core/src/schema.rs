use std::collections::{HashSet};
use indexmap::IndexMap;
use crate::shapes::{ShapeId, ShapeType};

// TODO: Support traits
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub id: ShapeId,
    pub shape_type: ShapeType,
    // pub traits: Option<String>,
    pub members: IndexMap<String, Schema>,
    pub member_target: Option<&'static Schema>,
    pub member_index: Option<usize>
}


// TODO: Support traits
impl Schema {
    fn root_schema(shape_type: ShapeType, id: ShapeId) -> Self {
        Schema {
            id,
            shape_type,
            members: Default::default(),
            member_target: None,
            member_index: None,
        }
    }

    pub fn create_boolean(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::BOOLEAN, id)
    }

    pub fn create_byte(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::BYTE, id)
    }

    pub fn create_short(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::SHORT, id)
    }

    pub fn create_integer(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::INTEGER, id)
    }

    pub fn create_int_enum(id: ShapeId, values: HashSet<i32>) -> Self {
        todo!()
    }

    pub fn create_long(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::LONG, id)
    }

    pub fn create_float(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::FLOAT, id)
    }

    pub fn create_double(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::DOUBLE, id)
    }

    pub fn create_big_integer(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::BIG_INTEGER, id)
    }

    pub fn create_big_decimal(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::BIG_DECIMAL, id)
    }

    pub fn create_string(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::STRING, id)
    }

    pub fn create_enum(id: ShapeId, values: HashSet<String>) -> Self {
        todo!()
    }

    pub fn create_blob(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::BLOB, id)
    }

    pub fn create_document(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::DOCUMENT, id)
    }

    pub fn create_timestamp(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::TIMESTAMP, id)
    }

    pub fn create_operation(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::OPERATION, id)
    }

    pub fn create_resource(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::RESOURCE, id)
    }

    pub fn create_service(id: ShapeId) -> Self {
        Self::root_schema(ShapeType::SERVICE, id)
    }

    pub fn get_member(&self, id: &str) -> Option<&Schema> {
        self.members.get(id)
    }

    pub fn expect_member(&self, id: &str) -> &Schema {
        self.members.get(id).unwrap()
    }
}

pub struct SchemaBuilder {
    id: ShapeId,
    shape_type: ShapeType,
    // pub traits: Option<String>,
    members: Vec<MemberSchemaBuilder>,
    member_target: Option<&'static Schema>,
    member_index: Option<usize>
}

impl SchemaBuilder {
    fn new(id: ShapeId, shape_type: ShapeType) -> Self {
        SchemaBuilder {
            id,
            members: match shape_type {
                ShapeType::LIST => Vec::with_capacity(1),
                ShapeType::MAP => Vec::with_capacity(2),
                _ => Vec::new(),
            },
            shape_type,
            member_target: None,
            member_index: None,
        }
    }

    pub fn put_member(&mut self, name: &str, target: &'static Schema) -> &mut Self {
        match target.shape_type {
            ShapeType::LIST => {
                if name != "member" {
                    // TODO: Real error
                    panic!("Lists can only have members named `member`")
                }
            }
            ShapeType::MAP => {
                if !(name == "key" || name == "value") {
                    panic!("Map can only have members named `key` or `value`")
                }
            }
            _ => { /* fall through otherwise */ }
        }
        self.members.push(MemberSchemaBuilder::new(name.into(), self.id.with_member(name), target));
        self
    }

    fn sort_members(&mut self) {
        // TODO: Implement.
    }

    // TODO: does this need cloning?
    pub fn build(&mut self) -> Schema {
        // Structure shapes need to sort members so that required members come before optional members.
        if self.shape_type == ShapeType::STRUCTURE {
            self.sort_members();
        }
        let mut member_map = IndexMap::with_capacity(self.members.len());
        // TODO: Could clone be avoided?
        for (idx, mut member_builder) in self.members.iter_mut().enumerate() {
            member_builder.set_index(idx);
            member_map.insert(member_builder.name.clone(), member_builder.build());
        }

        Schema {
            id: self.id.clone(),
            shape_type: self.shape_type.clone(),
            members: member_map,
            member_target: self.member_target,
            member_index: self.member_index,
        }
    }
}

impl Schema {
    pub fn structure_builder(id: ShapeId) -> SchemaBuilder {
        SchemaBuilder::new(id, ShapeType::STRUCTURE)
    }

    pub fn union_builder(id: ShapeId) -> SchemaBuilder {
        SchemaBuilder::new(id, ShapeType::UNION)
    }

    pub fn list_builder(id: ShapeId) -> SchemaBuilder {
        SchemaBuilder::new(id, ShapeType::LIST)
    }

    pub fn map_builder(id: ShapeId) -> SchemaBuilder {
        SchemaBuilder::new(id,ShapeType::MAP)
    }
}

struct MemberSchemaBuilder {
    pub (super) name: String,
    id: ShapeId,
    member_target: &'static Schema,
    member_index: Option<usize>
}

impl MemberSchemaBuilder {
    pub (super) fn new(name: String, id: ShapeId, target: &'static Schema) -> MemberSchemaBuilder {
        MemberSchemaBuilder {
            name,
            id,
            member_target: target,
            member_index: None,
        }
    }

    pub (super) fn set_index(&mut self, index: usize) {
        self.member_index = Some(index);
    }

    pub (super) fn build(&self) -> Schema {
        if self.member_index.is_none() {
            // TODO: real error
            panic!("Expected member index!");
        }
        Schema {
            id: self.id.clone(),
            shape_type: ShapeType::MEMBER,
            members: Default::default(),
            member_target: Some(self.member_target),
            member_index: self.member_index,
        }
    }
}

