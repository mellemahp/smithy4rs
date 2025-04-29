#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;
use indexmap::IndexMap;
use crate::shapes::{ShapeId, ShapeType};
use crate::traits::{SmithyTrait, StaticTraitId, TraitList, TraitMap};

// TODO: Support traits
#[derive(Clone)]
pub struct Schema<'s> {
    pub id: ShapeId,
    pub shape_type: ShapeType,
    pub members: Option<IndexMap<String, Schema<'s>>>,
    pub member_target: Option<&'s Schema<'s>>,
    pub member_name: Option<String>,
    pub member_index: Option<usize>,
    pub trait_map: TraitMap,
    // pub traits: Option<String>,
}

// TODO: Support traits

// FACTORY METHODS
// TODO: What should be inlined?
impl <'s> Schema<'s> {
    // TODO: Can these generics be simplified at all?
    // TODO: Could arrays somehow be used instead of vecs?
    fn root_schema<I: Into<ShapeId>>(shape_type: ShapeType, id: I, traits: Option<TraitList>) -> Self {
        Schema {
            id: id.into(),
            shape_type,
            members: None,
            member_target: None,
            member_name: None,
            member_index: None,
            trait_map: if let Some(t) = traits { TraitMap::of(t) } else { TraitMap::new() }
        }
    }

    pub fn create_boolean<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Boolean, id, traits)
    }

    pub fn create_byte<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Byte, id, traits)
    }

    pub fn create_short<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Short, id, traits)
    }

    pub fn create_integer<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Integer, id, traits)
    }

    pub fn create_int_enum<I: Into<ShapeId>>(id: I, values: HashSet<i32>, traits: Option<TraitList>) -> Self {
        todo!()
    }

    pub fn create_long<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Long, id, traits)
    }

    pub fn create_float<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Float, id, traits)
    }

    pub fn create_double<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Double, id, traits)
    }

    pub fn create_big_integer<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::BigInteger, id, traits)
    }

    pub fn create_big_decimal<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::BigDecimal, id, traits)
    }

    pub fn create_string<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::String, id, traits)
    }

    pub fn create_enum<I: Into<ShapeId>>(id: I, values: HashSet<String>, traits: Option<TraitList>) -> Self {
        todo!()
    }

    pub fn create_blob<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Blob, id, traits)
    }

    pub fn create_document<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Document, id, traits)
    }

    pub fn create_timestamp<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Timestamp, id, traits)
    }

    pub fn create_operation<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Operation, id, traits)
    }

    pub fn create_resource<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Resource, id, traits)
    }

    pub fn create_service<I: Into<ShapeId>>(id: I, traits: Option<TraitList>) -> Self {
        Self::root_schema(ShapeType::Service, id, traits)
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

// Trait access
impl Schema<'_> {
    pub fn contains_trait(&self, id: &ShapeId) -> bool {
        self.trait_map.contains(id)
    }

    pub fn contains_trait_type<T: StaticTraitId>(&self) -> bool {
        self.trait_map.contains(T::trait_id())
    }

    // TODO: Should be fallible
    pub fn get_as<T: SmithyTrait + StaticTraitId>(&self) -> Option<&T> {
        self.trait_map.get(T::trait_id())
            .and_then(|dyn_trait| dyn_trait.downcast_ref::<T>())
    }

    pub fn get_dyn(&self, id: &ShapeId) -> Option<&Arc<dyn SmithyTrait>> {
        self.trait_map.get(id)
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
    member_index: Option<usize>,
    traits: TraitMap
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
            traits: TraitMap::new(),
        }
    }
}

impl <'b> SchemaBuilder<'b> {
    pub fn put_member<'t>(mut self, name: &str, target: &'t Schema, traits: Option<TraitList>) -> Self
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
        self.members.push(MemberSchemaBuilder::new(name.into(), self.id.with_member(name), target, traits));
        self
    }

    pub fn with_trait<T: SmithyTrait>(mut self, smithy_trait: T) -> Self {
        self.traits.insert(smithy_trait);
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

        // TODO: Could the clones be removed somehow?
        Schema {
            id: self.id.clone(),
            shape_type: self.shape_type.clone(),
            members: Some(member_map.clone()),
            member_target: None,
            member_name: None,
            member_index: None,
            trait_map: self.traits.clone(),
        }
    }
}

struct MemberSchemaBuilder<'s>{
    pub (super) name: String,
    id: ShapeId,
    member_target: &'s Schema<'s>,
    member_index: Option<usize>,
    trait_map: TraitMap,
}
// TODO: Flatten target traits into the member schema
impl <'b> MemberSchemaBuilder<'b> {
    pub(super) fn new<'t>(name: String, id: ShapeId, target: &'t Schema<'_>, traits: Option<TraitList>) -> Self
    // Schema reference outlives this builder
    where 't: 'b
    {
        // Flatten all target traits into member
        let mut trait_map = if let Some(trait_values) = traits { TraitMap::of(trait_values) } else { TraitMap::new() };
        trait_map.extend(&target.trait_map);
        MemberSchemaBuilder {
            name,
            id,
            member_target: target,
            member_index: None,
            trait_map,
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
        // TODO: Could the clones be removed somehow?
        Schema {
            id: self.id.clone(),
            shape_type: ShapeType::Member,
            members: None,
            member_target: Some(self.member_target),
            member_name: Some(self.name.clone()),
            member_index: self.member_index,
            trait_map: self.trait_map.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::*;

    #[test]
    fn single_trait() {
        let schema = Schema::create_double(ShapeId::from("api.smithy#Example"), Some(vec!(Arc::new(HttpCode::new(10)))));
        assert!(schema.contains_trait_type::<HttpCode>());
        let http_value = schema.get_as::<HttpCode>().expect("No HTTP code trait present");
        assert_eq!(http_value.code(), 10)
    }

    #[test]
    fn flattened_trait() {
        let target = Schema::create_integer(ShapeId::from("api.smithy#Target"), Some(vec![Arc::new(HttpCode::new(10))]));
        let schema = Schema::structure_builder(ShapeId::from("api.smithy#Example"))
            .put_member("target_a", &target, None)
            .build();
        let member = schema.get_member("target_a").expect("No such member");
        assert!(member.contains_trait_type::<HttpCode>());
        let http_value = member.get_as::<HttpCode>().expect("No HTTP code trait present");
        assert_eq!(http_value.code(), 10)
    }
}