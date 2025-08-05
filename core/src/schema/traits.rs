use std::{collections::HashMap, fmt::Debug};

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

/// Pre-defined [`SmithyTrait`] implementations that have a static ID.
///
/// Generated or pre-defined Smithy Traits _should_ implement this trait.
pub trait StaticTraitId {
    /// Static trait ID as found in Smithy model definition of the trait.
    fn trait_id() -> &'static ShapeId;
}

/// Generic Representation of a trait that has no pre-defined rust implementation.
///
/// This type is used to represent any traits that do not have a corresponding
/// rust implementation, allowing user-defined traits with no generated
/// implementation to be read by runtime code.
#[derive(Debug, Clone)]
pub struct DynamicTrait {
    id: ShapeId,
    value: DocumentValue,
}
impl SmithyTrait for DynamicTrait {
    fn id(&self) -> &ShapeId {
        &self.id
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

/// Convenience type for cheaply-clonable reference to a dynamic trait.
pub type TraitRef = Ref<dyn SmithyTrait>;

/// Map used to track the traits applied to a shape.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TraitMap {
    map: HashMap<ShapeId, TraitRef>,
}
impl TraitMap {
    /// Creates an empty [`TraitMap`].
    pub fn new() -> TraitMap {
        TraitMap {
            map: HashMap::new(),
        }
    }

    /// Inserts a [`SmithyTrait`] into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated,
    /// and the previous value is returned.
    pub fn insert(&mut self, value: impl SmithyTrait) -> Option<TraitRef> {
        self.map.insert(value.id().clone(), Ref::new(value))
    }

    /// Returns true if the map contains a value for the specified trait ID.
    #[must_use]
    pub fn contains_trait(&self, id: &ShapeId) -> bool {
        self.map.contains_key(id)
    }

    /// Returns true if the map contains a trait of type `T`.
    #[must_use]
    pub fn contains_trait_type<T: StaticTraitId>(&self) -> bool {
        self.map.contains_key(T::trait_id())
    }

    /// Returns a reference to the `SmithyTrait` corresponding to the ID.
    ///
    /// If the [`SmithyTrait`] does not exist in the map, then returns `None`.
    pub fn get(&self, id: &ShapeId) -> Option<&TraitRef> {
        self.map.get(id)
    }

    /// Create a new [`TraitMap`] from a vector of [`SmithyTraits`].
    ///
    /// This method is primarily used for constructing shapes.
    pub fn of(traits: Vec<TraitRef>) -> Self {
        let mut map: TraitMap = TraitMap::new();
        for smithy_trait in traits {
            map.map.insert(smithy_trait.id().clone(), smithy_trait);
        }
        map
    }

    /// Extends collection with the contents of another [`TraitMap`].
    pub fn extend(&mut self, trait_map: &TraitMap) {
        self.map.extend(trait_map.map.clone());
    }

    /// Gets a [`SmithyTrait`] as a specific implementation if it exists.
    ///
    /// If the [`SmithyTrait`] does not exist in the map, returns `None`.
    #[must_use]
    pub fn get_trait_as<T: SmithyTrait + StaticTraitId>(&self) -> Option<&T> {
        self.map
            .get(T::trait_id())
            .and_then(|dyn_trait| dyn_trait.downcast_ref::<T>())
    }
}

impl Debug for dyn SmithyTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        // TODO
        f.write_str("dyn SmithyTrait")
    }
}

impl PartialEq for dyn SmithyTrait {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id() && (self.value() == other.value())
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};

    use super::*;
    use crate::prelude::{HTTPErrorTrait, JsonNameTrait};

    #[test]
    fn basic_map_functionality() {
        let mut map = TraitMap::new();
        map.insert(JsonNameTrait::new("a"));
        let dyn_id: ShapeId = "smithy.api#Dynamic".into();
        map.insert(DynamicTrait {
            id: dyn_id.clone(),
            value: DocumentValue::String("b".to_string()),
        });
        assert!(map.contains_trait(&dyn_id));
        assert!(map.contains_trait(JsonNameTrait::trait_id()));
    }

    #[test]
    fn map_extension() {
        let mut map_a = TraitMap::new();
        map_a.insert(JsonNameTrait::new("a"));
        let mut map_b = TraitMap::new();
        map_b.insert(HTTPErrorTrait::new(404));
        map_a.extend(&map_b);
        assert!(map_a.contains_trait(HTTPErrorTrait::trait_id()));
    }

    #[test]
    fn trait_conversion_to_type() {
        let mut map = TraitMap::new();
        map.insert(HTTPErrorTrait::new(404));
        let Some(cast_value) = map.get_trait_as::<HTTPErrorTrait>() else {
            panic!("Could not find expected trait!!!")
        };
        assert_eq!(cast_value.code, 404);
        assert_eq!(cast_value.type_id(), TypeId::of::<HTTPErrorTrait>());
    }
}
