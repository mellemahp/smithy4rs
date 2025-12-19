#![allow(dead_code)]

use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter},
    hash::Hash,
    ops::Deref,
    sync::{Arc, LazyLock, OnceLock, RwLock},
};

use rustc_hash::FxBuildHasher;

use crate::{
    FxIndexMap, FxIndexSet, Ref,
    prelude::{DefaultTrait, RequiredTrait},
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
    Enum(EnumSchema<&'static str>),
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
#[derive(PartialEq)]
pub struct StructSchema {
    id: ShapeId,
    shape_type: ShapeType,
    pub members: FxIndexMap<String, SchemaRef>,
    traits: TraitMap,
}

impl Debug for StructSchema {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "StructSchema {{")?;
        write!(f, "id: {:?}, ", self.id.name())?;
        write!(f, "shape_type: {:?}, ", self.shape_type)?;
        write!(f, "traits: {:?}, ", self.traits)?;
        for (key, value) in &self.members {
            if let Schema::Member(member) = &**value {
                write!(f, "[name: {}, type: {:?}]", key, member.target.id().name())?;
            }
        }
        write!(f, "}}")
    }
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
    pub target: MemberTarget,
    pub name: String,
    pub index: usize,
    traits: TraitMap,
    flattened_traits: OnceLock<TraitMap>,
}
impl MemberSchema {
    #[inline]
    fn traits(&self) -> &TraitMap {
        self.flattened_traits.get_or_init(|| {
            let mut flattened = TraitMap::new();
            flattened.extend(&self.traits);
            flattened.extend(self.target.traits());
            flattened
        })
    }
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
        values: Box<[i32]>,
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
        values: Box<[&'static str]>,
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
    pub fn structure_builder<I: Into<ShapeId>>(id: I, traits: TraitList) -> SchemaBuilder {
        SchemaBuilder::new(id, ShapeType::Structure, traits)
    }

    /// Create a new [`SchemaBuilder`] for a [Union](https://smithy.io/2.0/spec/aggregate-types.html#union) shape.
    #[must_use]
    pub fn union_builder<I: Into<ShapeId>>(id: I, traits: TraitList) -> SchemaBuilder {
        SchemaBuilder::new(id, ShapeType::Union, traits)
    }

    /// Create a new [`SchemaBuilder`] for a [List](https://smithy.io/2.0/spec/aggregate-types.html#list) shape.
    #[must_use]
    pub fn list_builder<I: Into<ShapeId>>(id: I, traits: TraitList) -> SchemaBuilder {
        SchemaBuilder::new(id, ShapeType::List, traits)
    }

    /// Create a new [`SchemaBuilder`] for a [Map](https://smithy.io/2.0/spec/aggregate-types.html#map) shape.
    #[must_use]
    pub fn map_builder<I: Into<ShapeId>>(id: I, traits: TraitList) -> SchemaBuilder {
        SchemaBuilder::new(id, ShapeType::Map, traits)
    }
}

static EMPTY: LazyLock<FxIndexMap<String, SchemaRef>> = LazyLock::new(FxIndexMap::default);

// GETTERS
impl Schema {
    /// Get the [`ShapeType`] of the schema.
    #[must_use]
    pub fn shape_type(&self) -> &ShapeType {
        match self {
            Schema::Scalar(ScalarSchema { shape_type, .. })
            | Schema::Struct(StructSchema { shape_type, .. }) => shape_type,
            Schema::Enum(_) => &ShapeType::Enum,
            Schema::IntEnum(_) => &ShapeType::IntEnum,
            Schema::List(_) => &ShapeType::List,
            Schema::Map(_) => &ShapeType::Map,
            Schema::Member(member) => member.target.shape_type(),
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

    #[inline]
    fn traits(&self) -> &TraitMap {
        match self {
            Schema::Scalar(ScalarSchema { traits, .. })
            | Schema::Struct(StructSchema { traits, .. })
            | Schema::List(ListSchema { traits, .. })
            | Schema::Map(MapSchema { traits, .. })
            | Schema::Enum(EnumSchema { traits, .. })
            | Schema::IntEnum(EnumSchema { traits, .. }) => traits,
            Schema::Member(member) => member.traits(),
        }
    }

    /// Get a map of all members attached to this schema.
    ///
    /// **NOTE**: Scalar schemas with no members will return an empty map.
    pub(crate) fn members(&self) -> &FxIndexMap<String, SchemaRef> {
        match self {
            // TODO(errors): Error handling
            Schema::Struct(StructSchema { members, .. }) => members,
            Schema::Member(member) => member.target.members(),
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
        self.traits().contains(id)
    }

    /// Returns true if the map contains a trait of type `T`.
    #[must_use]
    pub fn contains_type<T: StaticTraitId>(&self) -> bool {
        self.traits().contains_type::<T>()
    }

    /// Gets a [`SmithyTrait`] as a specific implementation if it exists.
    ///
    /// If the [`SmithyTrait`] does not exist on this schema, returns `None`.
    #[must_use]
    #[inline]
    pub fn get_trait_as<T: SmithyTrait + StaticTraitId>(&self) -> Option<&T> {
        self.traits().get_as::<T>()
    }

    /// Get a dynamic implementation of a [`SmithyTrait`] by shape ID.
    ///
    /// If the [`SmithyTrait`] does not exist on this schema, returns `None`.
    #[must_use]
    #[inline]
    pub fn get_trait(&self, id: &ShapeId) -> Option<&TraitRef> {
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
    pub fn as_enum(&self) -> Option<&EnumSchema<&'static str>> {
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
pub struct SchemaBuilder {
    id: ShapeId,
    shape_type: ShapeType,
    members: RwLock<Vec<MemberSchemaBuilder>>,
    traits: RwLock<TraitMap>,
    // Used for caching built value when constructing recursive shapes
    built: OnceLock<SchemaRef>,
}

impl SchemaBuilder {
    /// Create a new [`SchemaBuilder`] with no traits or members.
    fn new(id: impl Into<ShapeId>, shape_type: ShapeType, traits: TraitList) -> Self {
        SchemaBuilder {
            id: id.into(),
            members: match shape_type {
                ShapeType::List => RwLock::new(Vec::with_capacity(1)),
                ShapeType::Map => RwLock::new(Vec::with_capacity(2)),
                _ => RwLock::new(Vec::new()),
            },
            shape_type,
            traits: RwLock::new(TraitMap::of(traits)),
            built: OnceLock::new(),
        }
    }
}

impl SchemaBuilder {
    /// Add a member to the [`SchemaBuilder`]
    #[must_use]
    pub fn put_member<M: Into<MemberTarget>>(
        &self,
        name: &str,
        target: M,
        traits: TraitList,
    ) -> &Self {
        self.validate_member_name(name);
        self.members
            .write()
            .expect("Eek")
            .push(MemberSchemaBuilder::new(
                name.into(),
                self.id.with_member(name),
                target.into(),
                traits,
            ));
        self
    }

    fn validate_member_name(&self, name: &str) {
        // TODO(errors): Return a result instead of panicking?
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
    }

    /// Build a [`Schema`] and return a [`SchemaRef`] to it.
    // TODO(errors): Convert to `Result<SchemaRef, BuildError>
    #[must_use]
    pub fn build(&self) -> SchemaRef {
        if let Some(schema) = self.built.get() {
            return schema.clone();
        }

        let mut traits = TraitMap::new();
        traits.extend(&self.traits.read().unwrap());
        let output = match self.shape_type {
            ShapeType::Structure | ShapeType::Union => {
                let mut members_mut = self.members.write().expect("Lock poisoned.");
                members_mut.sort();
                let mut members =
                    FxIndexMap::with_capacity_and_hasher(members_mut.len(), FxBuildHasher);
                for (idx, member_builder) in members_mut.iter_mut().enumerate() {
                    member_builder.set_index(idx);
                    members.insert(member_builder.name.clone(), member_builder.build());
                }
                Ref::new(Schema::Struct(StructSchema {
                    id: self.id.clone(),
                    shape_type: self.shape_type,
                    members,
                    traits,
                }))
            }
            ShapeType::List => {
                let members = self.members.read().expect("Lock poisoned.");
                Ref::new(Schema::List(ListSchema {
                    id: self.id.clone(),
                    member: members
                        .first()
                        .expect("Expected `member` member for List Schema")
                        .build(),
                    traits,
                }))
            }
            ShapeType::Map => {
                let members = self.members.read().expect("Lock poisoned.");
                Ref::new(Schema::Map(MapSchema {
                    id: self.id.clone(),
                    key: members
                        .first()
                        .expect("Expected `key` member for Map schema")
                        .build(),
                    value: members
                        .get(1)
                        .expect("Expected `value` member for Map schema")
                        .build(),
                    traits,
                }))
            }
            _ => unreachable!("Builder can only be created for aggregate types."),
        };
        self.built.set(output.clone()).expect("Lock poisoned");
        output
    }
}

// TODO(references): Do Member targets need to use weak ref to avoid Arc cycles?
#[derive(Clone)]
pub enum MemberTarget {
    Resolved(SchemaRef),
    Lazy {
        builder: Arc<SchemaBuilder>,
        value: OnceLock<SchemaRef>,
    },
}
impl Deref for MemberTarget {
    type Target = SchemaRef;
    fn deref(&self) -> &Self::Target {
        match self {
            MemberTarget::Resolved(target) => target,
            MemberTarget::Lazy { builder, value } => value.get().unwrap_or_else(|| {
                value.set(builder.build()).expect("Lock poisoned");
                value.get().unwrap()
            }),
        }
    }
}
impl Debug for MemberTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO(formatting): Add a nicer format result
        writeln!(f, "Member Target")
    }
}
impl PartialEq for MemberTarget {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl Eq for MemberTarget {}
impl From<&SchemaRef> for MemberTarget {
    fn from(schema: &SchemaRef) -> Self {
        MemberTarget::Resolved(schema.clone())
    }
}
impl From<&LazyLock<SchemaRef>> for MemberTarget {
    fn from(schema: &LazyLock<SchemaRef>) -> Self {
        MemberTarget::Resolved(schema.deref().clone())
    }
}
impl From<&Arc<SchemaBuilder>> for MemberTarget {
    fn from(builder_ref: &Arc<SchemaBuilder>) -> Self {
        MemberTarget::Lazy {
            builder: builder_ref.clone(),
            value: OnceLock::new(),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct MemberSchemaBuilder {
    name: String,
    id: ShapeId,
    member_target: MemberTarget,
    traits: TraitMap,
    member_index: Option<usize>,
}

impl PartialOrd<Self> for MemberSchemaBuilder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for MemberSchemaBuilder {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort members to ensure that required members with no default come before other members.
        match (
            self.required_with_no_default(),
            other.required_with_no_default(),
        ) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
}
impl MemberSchemaBuilder {
    pub(super) fn new(
        name: String,
        id: ShapeId,
        member_target: MemberTarget,
        traits: TraitList,
    ) -> Self {
        MemberSchemaBuilder {
            name,
            id,
            member_target,
            traits: TraitMap::of(traits),
            member_index: None,
        }
    }

    pub(super) const fn set_index(&mut self, index: usize) {
        self.member_index = Some(index);
    }

    fn required_with_no_default(&self) -> bool {
        self.traits.contains_type::<RequiredTrait>() && !self.traits.contains_type::<DefaultTrait>()
    }

    pub(super) fn build(&self) -> SchemaRef {
        Ref::new(Schema::Member(MemberSchema {
            id: self.id.clone(),
            target: self.member_target.clone(),
            name: self.name.clone(),
            index: self.member_index.unwrap_or_default(),
            traits: self.traits.clone(),
            flattened_traits: OnceLock::new(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        prelude::{JsonNameTrait, STRING},
        schema::DocumentValue,
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
        assert_eq!(member.shape_type(), &ShapeType::Integer);
        assert_eq!(member.id(), &ShapeId::from("api.smithy#Example$target_a"));
        let Some(member_schema) = member.as_member() else {
            panic!("Should be member schema!")
        };
        assert_eq!(&member_schema.target.id(), &target.id());
    }

    #[test]
    #[should_panic(expected = "Lists can only have members named `member`. Found `bad`")]
    fn disallowed_list_schema() {
        let _schema = Schema::list_builder(ShapeId::from("api.smithy#List"), traits![])
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
        assert_eq!(member.shape_type(), &ShapeType::String);
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
        assert_eq!(key.shape_type(), &ShapeType::String);
        assert_eq!(key.id(), &ShapeId::from("api.smithy#Map$key"));

        let value = &map_schema.value;
        assert_eq!(value.shape_type(), &ShapeType::String);
        assert_eq!(value.id(), &ShapeId::from("api.smithy#Map$value"));
    }

    #[test]
    fn single_trait() {
        let schema = Schema::create_double(
            ShapeId::from("api.smithy#Example"),
            traits![JsonNameTrait::new("other")],
        );
        assert!(schema.contains_type::<JsonNameTrait>());
        let json_name_value = schema
            .get_trait_as::<JsonNameTrait>()
            .expect("No Json Name trait present");
        assert_eq!(json_name_value.name(), "other")
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
        assert!(member.contains_type::<JsonNameTrait>());
        let json_name_value = member
            .get_trait_as::<JsonNameTrait>()
            .expect("No JSON name trait present");
        assert_eq!(json_name_value.name(), "other");
    }

    #[test]
    fn self_referential_schema() {
        let builder = Arc::new(Schema::structure_builder("api.smithy#Example", traits![]));
        let output = builder
            .put_member("name", &STRING, traits![])
            .put_member("self", &builder, traits![])
            .build();
        assert_eq!(output.id(), &ShapeId::from("api.smithy#Example"));
        let member = output.get_member("self").expect("No `self` member");
        let Schema::Member(self_member) = &**member else {
            panic!("Expected `self` member");
        };
        assert_eq!(
            self_member.target.id(),
            &ShapeId::from("api.smithy#Example")
        );
    }

    #[test]
    fn mutually_recursive_schemas() {
        let builder_a = Arc::new(Schema::structure_builder("api.smithy#ExampleA", traits![]));
        let builder_b = Arc::new(Schema::structure_builder("api.smithy#ExampleB", traits![]));

        let output_a = builder_a
            .put_member("other_b", &builder_b, traits![])
            .build();
        let output_b = builder_b
            .put_member("other_a", &builder_a, traits![])
            .build();

        assert_eq!(output_a.id(), &ShapeId::from("api.smithy#ExampleA"));
        let member_b = output_a.get_member("other_b").expect("No `other_b` member");
        let Schema::Member(rec_member_b) = &**member_b else {
            panic!("Expected `self` member");
        };
        assert_eq!(
            rec_member_b.target.id(),
            &ShapeId::from("api.smithy#ExampleB")
        );

        assert_eq!(output_b.id(), &ShapeId::from("api.smithy#ExampleB"));
        let member_a = output_b.get_member("other_a").expect("No `other_a` member");
        let Schema::Member(rec_member_a) = &**member_a else {
            panic!("Expected `self` member");
        };
        assert_eq!(
            rec_member_a.target.id(),
            &ShapeId::from("api.smithy#ExampleA")
        );
    }

    #[test]
    fn recursive_via_list() {
        let intermediate_builder = Arc::new(Schema::structure_builder(
            "api.smithy#Intermediate",
            traits![],
        ));
        let list_builder = Arc::new(Schema::list_builder("api.smithy#RecursiveList", traits![]));
        let intermediate_struct = intermediate_builder
            .put_member("list", &list_builder, traits![])
            .build();
        let recursive_list = list_builder
            .put_member("member", &intermediate_struct, traits![])
            .build();
        assert_eq!(
            intermediate_struct.id(),
            &ShapeId::from("api.smithy#Intermediate")
        );
        assert_eq!(
            recursive_list.id(),
            &ShapeId::from("api.smithy#RecursiveList")
        );

        let list_member = intermediate_struct
            .get_member("list")
            .expect("No `list` member");
        let Schema::Member(rec_list) = &**list_member else {
            panic!("Expected `list` member");
        };
        assert_eq!(
            rec_list.target.id(),
            &ShapeId::from("api.smithy#RecursiveList")
        );

        let list_member = recursive_list.get_member("member").expect("No `member`");
        let Schema::Member(rec_struct) = &**list_member else {
            panic!("Expected `member` member");
        };
        assert_eq!(
            rec_struct.target.id(),
            &ShapeId::from("api.smithy#Intermediate")
        );
    }

    #[test]
    fn recursive_via_map() {
        let intermediate_builder = Arc::new(Schema::structure_builder(
            "api.smithy#Intermediate",
            traits![],
        ));
        let map_builder = Arc::new(Schema::map_builder("api.smithy#RecursiveMap", traits![]));
        let intermediate_struct = intermediate_builder
            .put_member("map", &map_builder, traits![])
            .build();
        let recursive_list = map_builder
            .put_member("key", &STRING, traits![])
            .put_member("value", &intermediate_struct, traits![])
            .build();
        assert_eq!(
            intermediate_struct.id(),
            &ShapeId::from("api.smithy#Intermediate")
        );
        assert_eq!(
            recursive_list.id(),
            &ShapeId::from("api.smithy#RecursiveMap")
        );

        let map_member = intermediate_struct
            .get_member("map")
            .expect("No `map` member");
        let Schema::Member(rec_map) = &**map_member else {
            panic!("Expected `map` member");
        };
        assert_eq!(
            rec_map.target.id(),
            &ShapeId::from("api.smithy#RecursiveMap")
        );

        let value_member = recursive_list.get_member("value").expect("No `value`");
        let Schema::Member(rec_struct) = &**value_member else {
            panic!("Expected `value` member");
        };
        assert_eq!(
            rec_struct.target.id(),
            &ShapeId::from("api.smithy#Intermediate")
        );
    }

    #[test]
    fn sorts_members() {
        let schema = Schema::structure_builder(ShapeId::from("api.smithy#Example"), traits![])
            .put_member(
                "target_b",
                &STRING,
                traits![
                    RequiredTrait,
                    DefaultTrait(DocumentValue::String("Woo".into()))
                ],
            )
            .put_member("target_a", &STRING, traits![RequiredTrait])
            .put_member("target_c", &STRING, traits![])
            .put_member("target_d", &STRING, traits![RequiredTrait])
            .put_member("target_e", &STRING, traits![])
            .build();
        assert_eq!(schema.members().len(), 5);
        let first = schema.members().get_index(0).unwrap().0;
        let second = schema.members().get_index(1).unwrap().0;
        let third = schema.members().get_index(2).unwrap().0;
        let fourth = schema.members().get_index(3).unwrap().0;
        let fifth = schema.members().get_index(4).unwrap().0;

        assert_eq!(first, "target_a");
        assert_eq!(second, "target_d");
        assert_eq!(third, "target_b");
        assert_eq!(fourth, "target_c");
        assert_eq!(fifth, "target_e");
    }
}
