//! # Smithy Traits
//! [Smithy Trait](https://smithy.io/2.0/spec/model.html#traits) definition and
//! associated utilities.
//!
//! ## Using Smithy Traits from as Schema
//!
//! Smithy [`crate::schema::Schema`]'s may contain one or more Smithy Traits. These
//! traits provide structured metadata for the schema and are the primary mechanism to
//! customize runtime/serde behavior of structures modeled with the schema.
//!
//! For example, the [`crate::prelude::JsonNameTrait`] customizes name of a field
//! when serialized by a JSON protocol.
//!
//! Traits on a [`Schema`] can be accessed using the [`Schema::get_trait`] or
//! [`Schema::get_trait_as`] method.
//!
//! Examples of accessing traits from a [`Schema`]:
//! ```rust
//! # use std::sync::LazyLock;
//! # use smithy4rs_core::{lazy_schema, traits, Ref};
//! # use smithy4rs_core::prelude::{LengthTrait, SensitiveTrait, STRING};
//! # use smithy4rs_core::schema::{Schema, StaticTraitId, SchemaRef, DocumentValue};
//!
//! lazy_schema!(
//!     EXAMPLE_SCHEMA,
//!     Schema::create_string("com.example#Map", traits![SensitiveTrait::new(), LengthTrait::builder().max(4).min(1).build()])
//! );
//!
//! /// Checking if a trait is present on a schema
//!  // Check by ID
//!  assert!(&EXAMPLE_SCHEMA.contains_trait(&"smithy.api#sensitive".into()));
//!  // Check by type
//!  assert!(&EXAMPLE_SCHEMA.contains_type::<SensitiveTrait>());
//!
//! /// Accessing trait data from a schema
//!  // Access as a dynamic trait object.
//!  let trait_object = EXAMPLE_SCHEMA.get_trait(&"smithy.api#sensitive".into()).unwrap();
//!  let document_value = trait_object.value();
//!  assert_eq!(document_value, &DocumentValue::Null);
//!
//!  // Downcast trait to specific impl
//!  let trait_impl = EXAMPLE_SCHEMA.get_trait_as::<LengthTrait>().unwrap();
//!  assert_eq!(*trait_impl.min(), Some(1usize));
//!  assert_eq!(*trait_impl.max(), Some(4usize));
//! ```
//!
//! ## Custom Traits
//!
//! Custom traits on a model are supported automatically with
//! [`DynamicTrait::from`] method. This maps detected traits into a [`dyn SmithyTrait`]
//! that can be queried from schemas using their `ShapeId`.
//!
//! Example:
//! ```rust
//! use smithy4rs_core::schema::{DocumentValue, DynamicTrait, ShapeId};
//!
//! // Create a `dyn SmithyTrait` from just the ID and object value.
//! // This corresponds to a custom trait in the smithy model like:
//! // use com.example#myCustomTrait
//! //
//! // @myCustomTrait(true)
//! // structure MyStruct { ... }
//! let custom_trait = DynamicTrait::from("com.example#myCustomTrait".into(), DocumentValue::Boolean(true));
//! ```
//!
//! Custom traits can also have either manually defined or code-generated concrete implementations.
//! Traits with concrete implementations that implement [`StaticTraitId`] (automatically provided
//! in code-generated traits) can be downcast into from a Schema.
//!
//! As a general rule, if your code needs to check more than just the presence of a
//! trait it is recommended to create a concrete implementation to make access to the
//! trait data easier and more structured.
//!
//! Base Smithy Trait implementations such as `@sensitive` and `@default`
//! can be found in [`crate::schema::prelude`].

use std::{collections::BTreeMap, fmt::Debug, ops::Deref};

use downcast_rs::{DowncastSync, impl_downcast};

use crate::{
    Ref,
    schema::{DocumentValue, ShapeId},
};

/// Base trait for all [Smithy Trait](https://smithy.io/2.0/spec/model.html#traits) implementations.
///
/// This trait can be downcast into a specific trait implementation.
///
/// ```rust,ignore
/// my_trait.downcast_ref::<SpecificTraitImpl>()
/// ```
///
/// **NOTE**: All Smithy Trait implementations MUST implement this trait.
pub trait SmithyTrait: DowncastSync {
    /// The ID of the trait as expressed in the Smithy model.
    fn id(&self) -> &ShapeId;

    /// The data stored inside the trait as a [`crate::schema::documents::Document`] value.
    fn value(&self) -> &DocumentValue;
}
impl_downcast!(sync SmithyTrait);
impl Debug for dyn SmithyTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "dyn SmithyTrait {{ id: {:?}, value: {:?} }}",
            self.id(),
            self.value()
        )
    }
}

/// Pre-defined [`SmithyTrait`] implementations that have a static ID.
///
/// Generated or pre-defined Smithy Traits _should_ implement this trait.
/// [`SmithyTrait`] implementations that do not implement this trait cannot
/// be downcast into by the [`TraitMap::get_as`] or [`Schema::get_trait_as`]
/// methods.
pub trait StaticTraitId: SmithyTrait {
    /// Static trait ID as found in Smithy model definition of the trait.
    fn trait_id() -> &'static ShapeId;
}

/// Convenience type for cheaply-cloneable reference to a dynamic trait.
///
/// This type is a thin wrapper used primarily to allow blanket conversion
/// implementations.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct TraitRef(Ref<dyn SmithyTrait>);
impl PartialEq for TraitRef {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id() && (self.value() == other.value())
    }
}
impl Deref for TraitRef {
    type Target = dyn SmithyTrait;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl From<Ref<dyn SmithyTrait>> for TraitRef {
    fn from(value: Ref<dyn SmithyTrait>) -> Self {
        Self(value)
    }
}
impl<T: SmithyTrait> From<T> for TraitRef {
    fn from(value: T) -> Self {
        Self(Ref::new(value))
    }
}

/// Generic Representation of a trait that has no pre-defined rust implementation.
///
/// This type is used to represent any traits that do not have a corresponding
/// rust implementation, allowing user-defined traits with no generated
/// implementation to be read by runtime code.
///
///  In general, users should try to move towards a code-generated versions and downcast
/// into those if they need to access data within the trait.
///
/// NOTE: Dynamic implementations cannot be downcast into a concrete implementation.
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicTrait {
    id: ShapeId,
    value: DocumentValue,
}
impl DynamicTrait {
    /// Create a new [`SmithyTrait`] with no corresponding concrete implementation.
    ///
    /// NOTE: Traits created with this method cannot be downcast into a specific implementation.
    pub fn from(id: ShapeId, value: DocumentValue) -> Ref<dyn SmithyTrait> {
        Ref::new(Self { id, value })
    }
}

impl SmithyTrait for DynamicTrait {
    fn id(&self) -> &ShapeId {
        &self.id
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

/// Map used to track the traits applied to a [`Schema`].
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TraitMap {
    // NOTE: BTreeMap is used here b/c it outperforms HashMap for access and memory usage
    //       when the collection size is small. Schemas typically have very few traits.
    map: BTreeMap<ShapeId, TraitRef>,
}
impl Eq for TraitMap {}
impl TraitMap {
    /// Creates a new, empty [`TraitMap`].
    ///
    /// Initially created with 0 capacity so it will not allocate until it
    /// is first inserted into.
    pub fn new() -> TraitMap {
        TraitMap {
            map: BTreeMap::new(),
        }
    }

    /// Inserts a [`SmithyTrait`] into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated,
    /// and the previous value is returned.
    pub fn insert(&mut self, value: impl Into<TraitRef>) -> Option<TraitRef> {
        let trait_ref = value.into();
        self.map.insert(trait_ref.id().clone(), trait_ref)
    }

    /// Returns true if the map contains a value for the specified trait ID.
    #[must_use]
    pub fn contains(&self, id: &ShapeId) -> bool {
        self.map.contains_key(id)
    }

    /// Returns true if the map contains a trait of type `T`.
    #[must_use]
    pub fn contains_type<T: StaticTraitId>(&self) -> bool {
        self.map.contains_key(T::trait_id())
    }

    /// Returns a reference to the `SmithyTrait` corresponding to the ID.
    ///
    /// If the [`SmithyTrait`] does not exist in the map, then returns `None`.
    #[must_use]
    pub fn get(&self, id: &ShapeId) -> Option<&TraitRef> {
        self.map.get(id)
    }

    /// Gets a [`SmithyTrait`] as a specific implementation if it exists.
    ///
    /// If the [`SmithyTrait`] does not exist in the map, returns `None`.
    #[must_use]
    pub fn get_as<T: SmithyTrait + StaticTraitId>(&self) -> Option<&T> {
        self.map
            .get(T::trait_id())
            .and_then(|dyn_trait| dyn_trait.downcast_ref::<T>())
    }

    /// Extends collection with the contents of another [`TraitMap`].
    pub fn extend(&mut self, trait_map: &TraitMap) {
        self.map.extend(trait_map.map.clone());
    }

    /// Create a new [`TraitMap`] from a vector of [`SmithyTraits`].
    ///
    /// This method is primarily used for constructing Schemas.
    pub(crate) fn of(traits: Vec<TraitRef>) -> Self {
        let mut map: TraitMap = TraitMap::new();
        for smithy_trait in traits {
            map.map.insert(smithy_trait.id().clone(), smithy_trait);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};

    use super::*;
    use crate::{
        prelude::{HTTPErrorTrait, JsonNameTrait},
        traits,
    };

    #[test]
    fn basic_map_functionality() {
        let mut map = TraitMap::new();
        map.insert(JsonNameTrait::new("a"));
        let dyn_id: ShapeId = "smithy.api#Dynamic".into();
        map.insert(DynamicTrait {
            id: dyn_id.clone(),
            value: DocumentValue::String("b".to_string()),
        });
        assert!(map.contains(&dyn_id));
        assert!(map.contains(JsonNameTrait::trait_id()));
        assert!(map.contains_type::<JsonNameTrait>());
    }

    #[test]
    fn map_extension() {
        let mut map_a = TraitMap::new();
        map_a.insert(JsonNameTrait::new("a"));

        let mut map_b = TraitMap::new();
        map_b.insert(HTTPErrorTrait::new(404));

        map_a.extend(&map_b);
        assert!(map_a.contains(HTTPErrorTrait::trait_id()));
        assert!(map_a.contains_type::<HTTPErrorTrait>());
        assert!(map_a.contains_type::<JsonNameTrait>());
    }

    #[test]
    fn trait_conversion_to_type() {
        let mut map = TraitMap::new();
        map.insert(HTTPErrorTrait::new(404));
        let Some(cast_value) = map.get_as::<HTTPErrorTrait>() else {
            panic!("Could not find expected trait!!!")
        };
        assert_eq!(cast_value.code(), 404);
        assert_eq!(cast_value.type_id(), TypeId::of::<HTTPErrorTrait>());
    }

    #[test]
    fn from_trait_vec() {
        let vec = traits![HTTPErrorTrait::new(404), JsonNameTrait::new("a")];
        let map = TraitMap::of(vec);

        assert!(map.contains_type::<HTTPErrorTrait>());
        assert!(map.contains_type::<JsonNameTrait>());
    }
}
