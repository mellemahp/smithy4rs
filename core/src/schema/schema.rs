#![allow(dead_code)]
#![allow(unused_variables)]

use crate::schema::shapes::{ShapeId, ShapeType};
use crate::schema::{SmithyTrait, StaticTraitId, TraitList, TraitMap};
use indexmap::IndexMap;
use std::collections::HashSet;
use std::sync::Arc;

pub type Ref<T> = Arc<T>;

#[derive(Debug)]
pub enum Schema<'schema> {
    Scalar(ScalarSchema),
    Struct(StructSchema<'schema>),
    Enum(EnumSchema<String>),
    IntEnum(EnumSchema<i32>),
    List(ListSchema<'schema>),
    Map(MapSchema<'schema>),
    Member(MemberSchema<'schema>)
}

#[derive(Debug)]
pub struct ScalarSchema {
    id: ShapeId,
    shape_type: ShapeType,
    traits: TraitMap,
}
#[derive(Debug)]
pub struct StructSchema<'schema> {
    id: ShapeId,
    shape_type: ShapeType,
    members: IndexMap<String, Ref<Schema<'schema>>>,
    traits: TraitMap,
}
#[derive(Debug)]
pub struct ListSchema<'schema> {
    id: ShapeId,
    member: Ref<Schema<'schema>>,
    traits: TraitMap,
}
impl ListSchema<'_> {
    fn member(&self) -> &Schema {
        &*self.member
    }
}

#[derive(Debug)]
pub struct MapSchema<'schema> {
    id: ShapeId,
    pub key: Ref<Schema<'schema>>,
    value: Ref<Schema<'schema>>,
    traits: TraitMap
}
#[derive(Debug)]
pub struct EnumSchema<T> {
    id: ShapeId,
    pub values: HashSet<T>,
    traits: TraitMap
}

#[derive(Debug)]
pub struct MemberSchema<'schema> {
    id: ShapeId,
    pub target: Ref<&'schema Schema<'schema>>,
    pub name: String,
    pub index: usize,
    traits: TraitMap,
}

// FACTORY METHODS
// TODO: What should be inlined?
impl Schema<'_> {

    fn into_arc(self) -> Ref<Self> {
        Ref::new(self)
    }

    // TODO: Can these generics be simplified at all?
    // TODO: Could arrays somehow be used instead of vecs?
    fn scalar(
        shape_type: ShapeType,
        id: impl Into<ShapeId>,
        traits: TraitList,
    ) -> Self {
        Schema::Scalar(ScalarSchema {
            id: id.into(),
            shape_type,
            traits: TraitMap::of(traits)
        })
    }

    pub fn create_boolean(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Boolean, id, traits)
    }

    pub fn create_byte(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Byte, id, traits)
    }

    pub fn create_short(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Short, id, traits)
    }

    pub fn create_integer(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Integer, id, traits)
    }

    pub fn create_int_enum(id: impl Into<ShapeId>, values: HashSet<i32>, traits: TraitList) -> Self {
        Self::IntEnum(EnumSchema { id: id.into(), values, traits: TraitMap::of(traits) })
    }

    pub fn create_long(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Long, id, traits)
    }

    pub fn create_float(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Float, id, traits)
    }

    pub fn create_double(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Double, id, traits)
    }

    pub fn create_big_integer(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::BigInteger, id, traits)
    }

    pub fn create_big_decimal(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::BigDecimal, id, traits)
    }

    pub fn create_string(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::String, id, traits)
    }

    pub fn create_enum(id: impl Into<ShapeId>, values: HashSet<String>, traits: TraitList) -> Self {
        Self::Enum(EnumSchema { id: id.into(), values, traits: TraitMap::of(traits) })
    }

    pub fn create_blob(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Blob, id, traits)
    }

    pub fn create_document(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Document, id, traits)
    }

    pub fn create_timestamp(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Timestamp, id, traits)
    }

    pub fn create_operation(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Operation, id, traits)
    }

    pub fn create_resource(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Resource, id, traits)
    }

    pub fn create_service(id: impl Into<ShapeId>, traits: TraitList) -> Self {
        Self::scalar(ShapeType::Service, id, traits)
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

// GETTERS
impl Schema<'_> {
    pub fn shape_type(&self) -> &ShapeType {
        match self {
            Schema::Scalar(ScalarSchema { shape_type, ..}) => shape_type,
            Schema::Struct(StructSchema { shape_type, ..}) => shape_type,
            Schema::Enum(_) => &ShapeType::Enum,
            Schema::IntEnum(_) => &ShapeType::IntEnum,
            Schema::List(_) => &ShapeType::List,
            Schema::Map(_) => &ShapeType::Map,
            Schema::Member(_) => &ShapeType::Member
        }
    }

    pub fn id(&self) -> &ShapeId {
        match self {
            Schema::Scalar(ScalarSchema { id, .. }) |
            Schema::Struct(StructSchema { id, .. }) |
            Schema::List(ListSchema { id, .. }) |
            Schema::Enum(EnumSchema { id, .. }) |
            Schema::IntEnum(EnumSchema { id, .. }) |
            Schema::Map(MapSchema { id, .. }) |
            Schema::Member(MemberSchema{ id, ..} ) => id,
        }
    }

    fn traits(&self) -> &TraitMap {
        match self {
            Schema::Scalar(ScalarSchema { traits, .. }) |
            Schema::Struct(StructSchema { traits, .. }) |
            Schema::List(ListSchema { traits, .. }) |
            Schema::Map(MapSchema { traits, .. }) |
            Schema::Enum(EnumSchema { traits, .. }) |
            Schema::IntEnum(EnumSchema { traits, .. }) |
            Schema::Member(MemberSchema{ traits, ..} ) => traits,
        }
    }

    pub fn get_member(&self, member_name: &str) -> Option<Ref<Schema>> {
        match self {
            Schema::Scalar(_) => None,
            Schema::Struct(schema) => schema.members.get(member_name).map(Clone::clone),
            Schema::Enum(_) => None,
            Schema::IntEnum(_) => None,
            Schema::List(schema) => {
                if member_name == "member" {
                    Some(schema.member.clone())
                } else {
                    panic!("GAHHHH")
                }
            },
            Schema::Map(schema) => {
                if member_name == "key" {
                    Some(schema.key.clone())
                } else if member_name == "value" {
                    Some(schema.value.clone())
                } else {
                    None
                }
            },
            Schema::Member(member) => member.target.get_member(member_name),
        }
    }

    pub fn expect_member(&self, member_name: &str) -> Ref<Schema> {
        self.get_member(member_name).expect(format!("Expected member: {member_name}").as_str())
    }

    pub fn contains_trait(&self, id: &ShapeId) -> bool {
        self.traits().contains(id)
    }

    pub fn contains_trait_type<T: StaticTraitId>(&self) -> bool {
        self.traits().contains(T::trait_id())
    }

    pub fn get_trait_as<T: SmithyTrait + StaticTraitId>(&self) -> Option<&T> {
        self.traits()
            .get(T::trait_id())
            .and_then(|dyn_trait| dyn_trait.downcast_ref::<T>())
    }

    pub fn get_trait_dyn(&self, id: &ShapeId) -> Option<&Ref<dyn SmithyTrait>> {
        self.traits().get(id)
    }
}

pub struct SchemaBuilder<'b> {
    id: ShapeId,
    shape_type: ShapeType,
    members: Vec<MemberSchemaBuilder<'b>>,
    traits: TraitMap,
}

impl SchemaBuilder<'_> {
    fn new(id: impl Into<ShapeId>, shape_type: ShapeType) -> Self {
        SchemaBuilder {
            id: id.into(),
            members: match shape_type {
                ShapeType::List => Vec::with_capacity(1),
                ShapeType::Map => Vec::with_capacity(2),
                _ => Vec::new()
            },
            shape_type,
            traits: TraitMap::new()
        }
    }
}

impl<'b> SchemaBuilder<'b> {
    pub fn put_member<'t>(
        mut self,
        name: &str,
        target: &'t Schema,
        traits: TraitList,
    ) -> Self
    // Target reference will outlive this builder
    where
        't: 'b,
    {
        match self.shape_type {
            ShapeType::List => {
                if name != "member" {
                    // TODO: Real error
                    panic!(
                        "Lists can only have members named `member`. Found `{}`",
                        name
                    )
                }
            }
            ShapeType::Map => {
                if !(name == "key" || name == "value") {
                    panic!("Map can only have members named `key` or `value`")
                }
            }
            _ => { /* fall through otherwise */ }
        }
        self.members.push(MemberSchemaBuilder::new(
            name.into(),
            self.id.with_member(name),
            Ref::new(target),
            traits,
        ));
        self
    }

    pub fn with_trait(mut self, smithy_trait: impl SmithyTrait) -> Self {
        self.traits.insert(smithy_trait);
        self
    }

    const fn sort_members(&mut self) {
        // TODO: Implement.
    }

    pub fn build<'a>(mut self) -> Schema<'a>
    where 'b: 'a
    {
        // Structure shapes need to sort members so that required members come before optional members.
        // Union types do not need this.
        if self.shape_type == ShapeType::Structure {
            self.sort_members();
        }

        match self.shape_type {
            ShapeType::Structure | ShapeType::Union => {
                let mut member_map = IndexMap::with_capacity(self.members.len());
                for (idx, mut member_builder) in self.members.into_iter().enumerate() {
                    member_builder.set_index(idx);
                    member_map.insert(member_builder.name.clone(), Ref::new(member_builder.build()));
                }
                Schema::Struct(StructSchema {
                    id: self.id.clone(),
                    shape_type: self.shape_type.clone(),
                    members: member_map.clone(),
                    traits: self.traits.clone(),
                })
            },
            ShapeType::List => {
                Schema::List(ListSchema {
                    id: self.id,
                    member: Ref::new(self.members.remove(0).build()),
                    traits: self.traits,
                })
            },
            ShapeType::Map => {
                Schema::Map(MapSchema {
                    id: self.id,
                    key: Ref::new(self.members.remove(0).build()),
                    value: Ref::new(self.members.remove(0).build()),
                    traits: self.traits,
                })
            }
            _ => unreachable!("Builder can only be created for aggregate types.")
        }
        // TODO: Could the clones be removed somehow?

    }
}

struct MemberSchemaBuilder<'schema> {
    name: String,
    id: ShapeId,
    member_target: Ref<&'schema Schema<'schema>>,
    member_index: Option<usize>,
    trait_map: TraitMap,
}
// TODO: Flatten target traits into the member schema
impl<'b> MemberSchemaBuilder<'b> {
    pub(super) fn new<'t>(
        name: String,
        id: ShapeId,
        target: Ref<&'t Schema>,
        traits: TraitList,
    ) -> Self
    // Schema reference outlives this builder
    where
        't: 'b,
    {
        // Flatten all target traits into member
        let mut trait_map= TraitMap::of(traits);
        trait_map.extend(target.traits());
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

    pub(super) fn build(self) -> Schema<'b> {
        Schema::Member(MemberSchema {
            id: self.id,
            target: self.member_target,
            name: self.name,
            index: self.member_index.unwrap_or_default(),
            traits: self.trait_map
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::schema::prelude;
    use super::*;
    use crate::schema::traits::JsonNameTrait;
    use crate::traits;

    #[test]
    fn scalar_schemas() {
        let schema = Schema::create_integer(ShapeId::from("api.example#Integer"), traits![]);
        assert_eq!(schema.shape_type(), &ShapeType::Integer);
        assert_eq!(schema.id(), &ShapeId::from("api.example#Integer"));
    }

    #[test]
    fn structure_schema() {
        let target = Schema::create_integer(ShapeId::from("api.smithy#Target"), traits![]);
        let schema = Schema::structure_builder(ShapeId::from("api.smithy#Example"))
            .put_member("target_a", &target, traits![])
            .put_member("target_b", &prelude::STRING, traits![])
            .build();
        assert_eq!(schema.shape_type(), &ShapeType::Structure);
        assert_eq!(schema.id(), &ShapeId::from("api.smithy#Example"));
        let member = schema.expect_member("target_a");
        assert_eq!(member.shape_type(), &ShapeType::Member);
        assert_eq!(member.id(), &ShapeId::from("api.smithy#Example$target_a"));
        let Schema::Member(member_schema) = &*member else {
            panic!("Should be member schema!")
        };
        assert_eq!(&member_schema.target.id(), &target.id());
    }

    #[test]
    #[should_panic(expected = "Lists can only have members named `member`. Found `bad`")]
    fn disallowed_list_schema() {
        let schema = Schema::list_builder(ShapeId::from("api.smithy#List"))
            .put_member("bad", &prelude::STRING, traits![])
            .build();
    }

    #[test]
    fn list_schema() {
        let schema = Schema::list_builder(ShapeId::from("api.smithy#List"))
            .put_member("member", &prelude::STRING, traits![])
            .build();
        assert_eq!(schema.shape_type(), &ShapeType::List);
        assert_eq!(schema.id(), &ShapeId::from("api.smithy#List"));
        let Schema::List(list_schema) = schema else {
            panic!("Should be list!")
        };
        let member = &list_schema.member;
        assert_eq!(member.shape_type(), &ShapeType::Member);
        assert_eq!(member.id(), &ShapeId::from("api.smithy#List$member"));
    }

    #[test]
    fn map_schema() {
        let schema = Schema::map_builder(ShapeId::from("api.smithy#Map"))
            .put_member("key", &prelude::STRING, traits![])
            .put_member("value", &prelude::STRING, traits![])
            .build();
        assert_eq!(schema.shape_type(), &ShapeType::Map);
        assert_eq!(schema.id(), &ShapeId::from("api.smithy#Map"));
        let Schema::Map(map_schema) = schema else {
            panic!("Should be map!")
        };
        let key = &map_schema.key;
        assert_eq!(key.shape_type(), &ShapeType::Member);
        assert_eq!(key.id(), &ShapeId::from("api.smithy#Map$key"));

        let value = &map_schema.value;
        assert_eq!(value.shape_type(), &ShapeType::Member);
        assert_eq!(value.id(), &ShapeId::from("api.smithy#Map$value"));
    }

    #[test]
    fn single_trait() {
        let schema = Schema::create_double(
            ShapeId::from("api.smithy#Example"),
            traits![JsonNameTrait::new("other")],
        );
        assert!(schema.contains_trait_type::<JsonNameTrait>());
        let json_name_value = schema
            .get_trait_as::<JsonNameTrait>()
            .expect("No Json Name trait present");
        assert_eq!(json_name_value.name, "other")
    }

    #[test]
    fn flattened_trait() {
        let target = Schema::create_integer(
            ShapeId::from("api.smithy#Target"),
            traits![JsonNameTrait::new("other")],
        );
        let schema = Schema::structure_builder(ShapeId::from("api.smithy#Example"))
            .put_member("target_a", &target, traits![])
            .build();
        let member = schema.get_member("target_a").expect("No such member");
        assert!(member.contains_trait_type::<JsonNameTrait>());
        let json_name_value = member
            .get_trait_as::<JsonNameTrait>()
            .expect("No JSON name trait present");
        assert_eq!(json_name_value.name, "other");
    }
}
