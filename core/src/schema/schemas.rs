#![allow(dead_code)]
#![allow(unused_variables)]

use std::{hash::Hash, sync::LazyLock};

use indexmap::IndexSet;
use rustc_hash::FxBuildHasher;

use crate::{
    FxIndexMap, FxIndexSet, Ref,
    schema::{ShapeId, ShapeType, SmithyTrait, StaticTraitId, TraitMap, TraitRef},
};

/// Reference to a Smithy Schema type.
///
/// Allows for cheap copying and read only access to schema data.
/// This type is primarily used to handle indirection required to build
/// aggregate schemas and potentially recursive schemas.
pub type SchemaRef = Ref<Schema>;

/// Convenience type representing a list of trait implementations.
pub type TraitList = Vec<TraitRef>;

/// Describes a generated shape with metadata from a Smithy model.
#[derive(Debug, PartialEq)]
pub enum Schema {
    Scalar(ScalarSchema),
    Struct(StructSchema),
    Enum(EnumSchema<String>),
    IntEnum(EnumSchema<i32>),
    List(ListSchema),
    Map(MapSchema),
    Member(MemberSchema),
}

/// Schema for simple data with no members.
#[derive(Debug, PartialEq)]
pub struct ScalarSchema {
    id: ShapeId,
    shape_type: ShapeType,
    traits: TraitMap,
}

/// Schema for a Smithy [Structure](https://smithy.io/2.0/spec/aggregate-types.html#structure)
/// or [Union](https://smithy.io/2.0/spec/aggregate-types.html#union) data type.
#[derive(Debug, PartialEq)]
pub struct StructSchema {
    id: ShapeId,
    shape_type: ShapeType,
    pub members: FxIndexMap<String, SchemaRef>,
    traits: TraitMap,
}

/// Schema for a Smithy [List](https://smithy.io/2.0/spec/aggregate-types.html#list) data type.
#[derive(Debug, PartialEq)]
pub struct ListSchema {
    id: ShapeId,
    pub member: SchemaRef,
    traits: TraitMap,
}

/// Schema for a Smithy [Map](https://smithy.io/2.0/spec/aggregate-types.html#map) data type.
#[derive(Debug, PartialEq)]
pub struct MapSchema {
    id: ShapeId,
    pub key: SchemaRef,
    pub value: SchemaRef,
    traits: TraitMap,
}

/// Schema for a Smithy [Enum](https://smithy.io/2.0/spec/aggregate-types.html#map) data type.
#[derive(Debug, PartialEq)]
pub struct EnumSchema<T: PartialEq + Hash + Eq> {
    id: ShapeId,
    pub values: FxIndexSet<T>,
    traits: TraitMap,
}

/// Member of another aggregate type.
#[derive(Debug, PartialEq)]
pub struct MemberSchema {
    id: ShapeId,
    pub target: SchemaRef,
    pub name: String,
    pub index: usize,
    traits: TraitMap,
}

// =======  FACTORY METHODS ==========
impl Schema {
    fn scalar(shape_type: ShapeType, id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Ref::new(Schema::Scalar(ScalarSchema {
            id: id.into(),
            shape_type,
            traits: TraitMap::of(traits),
        }))
    }

    /// Create a Schema for a [Boolean](https://smithy.io/2.0/spec/simple-types.html#boolean) shape.
    pub fn create_boolean(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Boolean, id, traits)
    }

    /// Create a Schema for a [Byte](https://smithy.io/2.0/spec/simple-types.html#byte) shape.
    pub fn create_byte(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Byte, id, traits)
    }

    /// Create a Schema for a [Short](https://smithy.io/2.0/spec/simple-types.html#short) shape.
    pub fn create_short(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Short, id, traits)
    }

    /// Create a Schema for an [Integer](https://smithy.io/2.0/spec/simple-types.html#integer) shape.
    pub fn create_integer(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Integer, id, traits)
    }

    /// Create a Schema for an [IntEnum](https://smithy.io/2.0/spec/simple-types.html#intenum) shape.
    pub fn create_int_enum(
        id: impl Into<ShapeId>,
        values: IndexSet<i32>,
        traits: TraitList,
    ) -> SchemaRef {
        Ref::new(Self::IntEnum(EnumSchema {
            id: id.into(),
            values: FxIndexSet::from_iter(values),
            traits: TraitMap::of(traits),
        }))
    }

    /// Create a Schema for a [Long](https://smithy.io/2.0/spec/simple-types.html#long) shape.
    pub fn create_long(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Long, id, traits)
    }

    /// Create a Schema for a [Float](https://smithy.io/2.0/spec/simple-types.html#long) shape.
    pub fn create_float(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Float, id, traits)
    }

    /// Create a Schema for a [Double](https://smithy.io/2.0/spec/simple-types.html#double) shape.
    pub fn create_double(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Double, id, traits)
    }

    /// Create a Schema for a [BigInteger](https://smithy.io/2.0/spec/simple-types.html#biginteger) shape.
    pub fn create_big_integer(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::BigInteger, id, traits)
    }

    /// Create a Schema for a [BigDecimal](https://smithy.io/2.0/spec/simple-types.html#bigdecimal) shape.
    pub fn create_big_decimal(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::BigDecimal, id, traits)
    }

    /// Create a Schema for a [String](https://smithy.io/2.0/spec/simple-types.html#string) shape.
    pub fn create_string(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::String, id, traits)
    }

    /// Create a Schema for an [Enum](https://smithy.io/2.0/spec/simple-types.html#enum) shape.
    pub fn create_enum(
        id: impl Into<ShapeId>,
        values: IndexSet<String>,
        traits: TraitList,
    ) -> SchemaRef {
        Ref::new(Self::Enum(EnumSchema {
            id: id.into(),
            values: FxIndexSet::from_iter(values),
            traits: TraitMap::of(traits),
        }))
    }

    /// Create a Schema for a [Blob](https://smithy.io/2.0/spec/simple-types.html#blob) shape.
    pub fn create_blob(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Blob, id, traits)
    }

    /// Create a Schema for a [Document](https://smithy.io/2.0/spec/simple-types.html#document) shape.
    pub fn create_document(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Document, id, traits)
    }

    /// Create a Schema for a [Timestamp](https://smithy.io/2.0/spec/simple-types.html#timestamp) shape.
    pub fn create_timestamp(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Timestamp, id, traits)
    }

    /// Create a Schema for an [Operation](https://smithy.io/2.0/spec/service-types.html#operation) shape.
    pub fn create_operation(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Operation, id, traits)
    }

    /// Create a Schema for a [Resource](https://smithy.io/2.0/spec/service-types.html#resource) shape.
    pub fn create_resource(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Resource, id, traits)
    }

    /// Create a Schema for a [Service](https://smithy.io/2.0/spec/service-types.html#service) shape.
    pub fn create_service(id: impl Into<ShapeId>, traits: TraitList) -> SchemaRef {
        Self::scalar(ShapeType::Service, id, traits)
    }
}

// BUILDER FACTORIES
impl Schema {
    /// Create a new [`SchemaBuilder`] for a [Structure](https://smithy.io/2.0/spec/aggregate-types.html#structure) shape.
    #[must_use]
    pub fn structure_builder<'s, I: Into<ShapeId>>(id: I, traits: TraitList) -> SchemaBuilder<'s> {
        SchemaBuilder::new(id, ShapeType::Structure, traits)
    }

    /// Create a new [`SchemaBuilder`] for a [Union](https://smithy.io/2.0/spec/aggregate-types.html#union) shape.
    #[must_use]
    pub fn union_builder<'s, I: Into<ShapeId>>(id: I, traits: TraitList) -> SchemaBuilder<'s> {
        SchemaBuilder::new(id, ShapeType::Union, traits)
    }

    /// Create a new [`SchemaBuilder`] for a [List](https://smithy.io/2.0/spec/aggregate-types.html#list) shape.
    #[must_use]
    pub fn list_builder<'s, I: Into<ShapeId>>(id: I, traits: TraitList) -> SchemaBuilder<'s> {
        SchemaBuilder::new(id, ShapeType::List, traits)
    }

    /// Create a new [`SchemaBuilder`] for a [Map](https://smithy.io/2.0/spec/aggregate-types.html#map) shape.
    #[must_use]
    pub fn map_builder<'s, I: Into<ShapeId>>(id: I, traits: TraitList) -> SchemaBuilder<'s> {
        SchemaBuilder::new(id, ShapeType::Map, traits)
    }
}

static EMPTY: LazyLock<FxIndexMap<String, SchemaRef>> = LazyLock::new(FxIndexMap::default);

// GETTERS
impl Schema {
    /// Get the [`ShapeType`] of the schema.
    #[must_use]
    pub const fn shape_type(&self) -> &ShapeType {
        match self {
            Schema::Scalar(ScalarSchema { shape_type, .. })
            | Schema::Struct(StructSchema { shape_type, .. }) => shape_type,
            Schema::Enum(_) => &ShapeType::Enum,
            Schema::IntEnum(_) => &ShapeType::IntEnum,
            Schema::List(_) => &ShapeType::List,
            Schema::Map(_) => &ShapeType::Map,
            Schema::Member(_) => &ShapeType::Member,
        }
    }

    /// Get the [`ShapeId`] of the schema.
    #[must_use]
    pub const fn id(&self) -> &ShapeId {
        match self {
            Schema::Scalar(ScalarSchema { id, .. })
            | Schema::Struct(StructSchema { id, .. })
            | Schema::List(ListSchema { id, .. })
            | Schema::Enum(EnumSchema { id, .. })
            | Schema::IntEnum(EnumSchema { id, .. })
            | Schema::Map(MapSchema { id, .. })
            | Schema::Member(MemberSchema { id, .. }) => id,
        }
    }

    const fn traits(&self) -> &TraitMap {
        match self {
            Schema::Scalar(ScalarSchema { traits, .. })
            | Schema::Struct(StructSchema { traits, .. })
            | Schema::List(ListSchema { traits, .. })
            | Schema::Map(MapSchema { traits, .. })
            | Schema::Enum(EnumSchema { traits, .. })
            | Schema::IntEnum(EnumSchema { traits, .. })
            | Schema::Member(MemberSchema { traits, .. }) => traits,
        }
    }

    /// Get a map of all members attached to this schema.
    ///
    /// **NOTE**: Scalar schemas with no members will return an empty map.
    pub(crate) fn members(&self) -> &FxIndexMap<String, SchemaRef> {
        match self {
            Schema::Struct(StructSchema { members, .. }) => members,
            _ => &EMPTY,
        }
    }

    /// Get the schema for a specific member by member name
    #[must_use]
    pub fn get_member(&self, member_name: &str) -> Option<&SchemaRef> {
        match self {
            Schema::Struct(schema) => schema.members.get(member_name),
            Schema::List(schema) => match member_name {
                "member" => Some(&schema.member),
                _ => None,
            },
            Schema::Map(schema) => match member_name {
                "key" => Some(&schema.key),
                "value" => Some(&schema.value),
                _ => None,
            },
            Schema::Member(member) => member.target.get_member(member_name),
            _ => None,
        }
    }

    /// Returns member schema reference or *panics*
    ///
    /// **WARNING**: In general this should only be used in generated code.
    #[must_use]
    pub fn expect_member(&self, member_name: &str) -> &SchemaRef {
        self.get_member(member_name)
            .unwrap_or_else(|| panic!("Expected member: {member_name}"))
    }

    /// Returns true if the map contains a value for the specified trait ID.
    #[must_use]
    pub fn contains_trait(&self, id: &ShapeId) -> bool {
        self.traits().contains_trait(id)
    }

    /// Returns true if the map contains a trait of type `T`.
    #[must_use]
    pub fn contains_trait_type<T: StaticTraitId>(&self) -> bool {
        self.traits().contains_trait_type::<T>()
    }

    /// Gets a [`SmithyTrait`] as a specific implementation if it exists.
    ///
    /// If the [`SmithyTrait`] does not exist on this schema, returns `None`.
    #[must_use]
    pub fn get_trait_as<T: SmithyTrait + StaticTraitId>(&self) -> Option<&T> {
        self.traits().get_trait_as::<T>()
    }

    /// Get a dynamic implementation of a [`SmithyTrait`] by shape ID.
    ///
    /// If the [`SmithyTrait`] does not exist on this schema, returns `None`.
    #[must_use]
    pub fn get_trait_dyn(&self, id: &ShapeId) -> Option<&TraitRef> {
        self.traits().get(id)
    }
}

// AS-ers
impl Schema {
    /// Get as a [`MemberSchema`] type if possible, otherwise `None`.
    #[must_use]
    pub fn as_member(&self) -> Option<&MemberSchema> {
        if let Schema::Member(member) = self {
            Some(member)
        } else {
            None
        }
    }

    /// Get as a [`ListSchema`] type if possible, otherwise `None`.
    #[must_use]
    pub fn as_list(&self) -> Option<&ListSchema> {
        if let Schema::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Get as a [`StructSchema`] type if possible, otherwise `None`.
    #[must_use]
    pub fn as_struct(&self) -> Option<&StructSchema> {
        if let Schema::Struct(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Get as a [`ScalarSchema`] type if possible, otherwise `None`.
    #[must_use]
    pub fn as_scalar(&self) -> Option<&ScalarSchema> {
        if let Schema::Scalar(schema) = self {
            Some(schema)
        } else {
            None
        }
    }

    /// Get as a [`MapSchema`] type if possible, otherwise `None`.
    #[must_use]
    pub fn as_map(&self) -> Option<&MapSchema> {
        if let Schema::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Get as an [`EnumSchema`] type with `String` inner value
    /// if possible, otherwise `None`.
    #[must_use]
    pub fn as_enum(&self) -> Option<&EnumSchema<String>> {
        if let Schema::Enum(enum_schema) = self {
            Some(enum_schema)
        } else {
            None
        }
    }

    /// Get as an [`EnumSchema`] type with `i32` inner type if possible,
    /// otherwise `None`.
    #[must_use]
    pub fn as_int_enum(&self) -> Option<&EnumSchema<i32>> {
        if let Schema::IntEnum(enum_schema) = self {
            Some(enum_schema)
        } else {
            None
        }
    }
}

/// Builder for aggregate [`Schema`] types.
pub struct SchemaBuilder<'b> {
    id: ShapeId,
    shape_type: ShapeType,
    members: Vec<MemberSchemaBuilder<'b>>,
    traits: TraitMap,
}

impl SchemaBuilder<'_> {
    /// Create a new [`SchemaBuilder`] with no traits or members.
    fn new(id: impl Into<ShapeId>, shape_type: ShapeType, traits: TraitList) -> Self {
        SchemaBuilder {
            id: id.into(),
            members: match shape_type {
                ShapeType::List => Vec::with_capacity(1),
                ShapeType::Map => Vec::with_capacity(2),
                _ => Vec::new(),
            },
            shape_type,
            traits: TraitMap::of(traits),
        }
    }
}

impl<'b> SchemaBuilder<'b> {
    /// Add a member to the [`SchemaBuilder`]
    #[must_use]
    pub fn put_member<'t>(mut self, name: &str, target: &'t SchemaRef, traits: TraitList) -> Self
    // Target reference will outlive this builder
    where
        't: 'b,
    {
        // TODO: Return a result instead of panicking?
        match self.shape_type {
            ShapeType::List => {
                assert_eq!(
                    name, "member",
                    "Lists can only have members named `member`. Found `{name}`"
                );
            }
            ShapeType::Map => {
                assert!(
                    name == "key" || name == "value",
                    "Map can only have members named `key` or `value`"
                );
            }
            _ => { /* fall through otherwise */ }
        }
        self.members.push(MemberSchemaBuilder::new(
            name.into(),
            self.id.with_member(name),
            target,
            traits,
        ));
        self
    }

    /// Adds a trait to the [`SchemaBuilder`]
    #[must_use]
    pub fn with_trait(mut self, smithy_trait: impl SmithyTrait) -> Self {
        self.traits.insert(smithy_trait);
        self
    }

    const fn sort_members(&mut self) {
        // TODO: Implement.
    }

    /// Build a [`Schema`] and return a [`SchemaRef`] to it.
    // TODO: Convert to `Result<SchemaRef, BuildError>
    #[must_use]
    pub fn build(mut self) -> SchemaRef {
        // Structure shapes need to sort members so that required members come before optional members.
        // Union types do not need this.
        if self.shape_type == ShapeType::Structure {
            self.sort_members();
        }

        match self.shape_type {
            ShapeType::Structure | ShapeType::Union => {
                let mut member_map =
                    FxIndexMap::with_capacity_and_hasher(self.members.len(), FxBuildHasher);
                for (idx, mut member_builder) in self.members.into_iter().enumerate() {
                    member_builder.set_index(idx);
                    member_map.insert(member_builder.name.clone(), member_builder.build());
                }
                Ref::new(Schema::Struct(StructSchema {
                    id: self.id.clone(),
                    shape_type: self.shape_type,
                    members: member_map.clone(),
                    traits: self.traits.clone(),
                }))
            }
            ShapeType::List => Ref::new(Schema::List(ListSchema {
                id: self.id,
                member: self.members.remove(0).build(),
                traits: self.traits,
            })),
            ShapeType::Map => Ref::new(Schema::Map(MapSchema {
                id: self.id,
                key: self.members.remove(0).build(),
                value: self.members.remove(0).build(),
                traits: self.traits,
            })),
            _ => unreachable!("Builder can only be created for aggregate types."),
        }
    }
}

struct MemberSchemaBuilder<'target> {
    name: String,
    id: ShapeId,
    member_target: &'target SchemaRef,
    member_index: Option<usize>,
    trait_map: TraitMap,
}
// TODO: Flatten target traits into the member schema
impl<'b> MemberSchemaBuilder<'b> {
    pub(super) fn new<'t>(
        name: String,
        id: ShapeId,
        member_target: &'t SchemaRef,
        traits: TraitList,
    ) -> Self
    // Schema reference outlives this builder
    where
        't: 'b,
    {
        // Flatten all target traits into member
        let mut trait_map = TraitMap::of(traits);
        trait_map.extend(member_target.traits());
        MemberSchemaBuilder {
            name,
            id,
            member_target,
            member_index: None,
            trait_map,
        }
    }

    pub(super) const fn set_index(&mut self, index: usize) {
        self.member_index = Some(index);
    }

    pub(super) fn build(self) -> SchemaRef {
        Ref::new(Schema::Member(MemberSchema {
            id: self.id,
            target: self.member_target.clone(),
            name: self.name,
            index: self.member_index.unwrap_or_default(),
            traits: self.trait_map,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        prelude::{JsonNameTrait, STRING},
        traits,
    };

    #[test]
    fn scalar_schemas() {
        let schema = Schema::create_integer(ShapeId::from("api.example#Integer"), traits![]);
        assert_eq!(schema.shape_type(), &ShapeType::Integer);
        assert_eq!(schema.id(), &ShapeId::from("api.example#Integer"));
    }

    #[test]
    fn structure_schema() {
        let target = Schema::create_integer(ShapeId::from("api.smithy#Target"), traits![]);
        let schema = Schema::structure_builder(ShapeId::from("api.smithy#Example"), traits![])
            .put_member("target_a", &target, traits![])
            .put_member("target_b", &STRING, traits![])
            .build();
        assert_eq!(schema.shape_type(), &ShapeType::Structure);
        assert_eq!(schema.id(), &ShapeId::from("api.smithy#Example"));
        let member = schema.get_member("target_a").unwrap();
        assert_eq!(member.shape_type(), &ShapeType::Member);
        assert_eq!(member.id(), &ShapeId::from("api.smithy#Example$target_a"));
        let Some(member_schema) = member.as_member() else {
            panic!("Should be member schema!")
        };
        assert_eq!(&member_schema.target.id(), &target.id());
    }

    #[test]
    #[should_panic(expected = "Lists can only have members named `member`. Found `bad`")]
    fn disallowed_list_schema() {
        let schema = Schema::list_builder(ShapeId::from("api.smithy#List"), traits![])
            .put_member("bad", &STRING, traits![])
            .build();
    }

    #[test]
    fn list_schema() {
        let schema = Schema::list_builder(ShapeId::from("api.smithy#List"), traits![])
            .put_member("member", &STRING, traits![])
            .build();
        assert_eq!(schema.shape_type(), &ShapeType::List);
        assert_eq!(schema.id(), &ShapeId::from("api.smithy#List"));
        let Some(list_schema) = schema.as_list() else {
            panic!("Should be list!")
        };
        let member = &list_schema.member;
        assert_eq!(member.shape_type(), &ShapeType::Member);
        assert_eq!(member.id(), &ShapeId::from("api.smithy#List$member"));
    }

    #[test]
    fn map_schema() {
        let schema = Schema::map_builder(ShapeId::from("api.smithy#Map"), traits![])
            .put_member("key", &STRING, traits![])
            .put_member("value", &STRING, traits![])
            .build();
        assert_eq!(schema.shape_type(), &ShapeType::Map);
        assert_eq!(schema.id(), &ShapeId::from("api.smithy#Map"));
        let Some(map_schema) = schema.as_map() else {
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
        let schema = Schema::structure_builder(ShapeId::from("api.smithy#Example"), traits![])
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
