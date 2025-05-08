#[allow(dead_code)]
use crate::schema::documents::DocumentValue;
use crate::schema::shapes::ShapeId;
use downcast_rs::{impl_downcast, DowncastSync};
use std::collections::HashMap;
use crate::schema::Ref;

pub trait SmithyTrait: DowncastSync {
    fn id(&self) -> &ShapeId;
    fn value(&self) -> &DocumentValue;
}
impl_downcast!(sync SmithyTrait);

pub trait StaticTraitId {
    fn trait_id() -> &'static ShapeId;
}

pub struct DynamicTrait {
    id: ShapeId,
    value: DocumentValue<'static>,
}
impl SmithyTrait for DynamicTrait {
    fn id(&self) -> &ShapeId {
        &self.id
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

pub type TraitList = Vec<Ref<dyn SmithyTrait>>;

#[derive(Clone)]
pub(crate) struct TraitMap {
    map: HashMap<ShapeId, Ref<dyn SmithyTrait>>,
}
impl TraitMap {
    pub fn new() -> TraitMap {
        TraitMap {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: impl SmithyTrait) -> Option<Ref<dyn SmithyTrait>> {
        self.map.insert(value.id().clone(), Ref::new(value))
    }

    pub fn contains(&self, id: &ShapeId) -> bool {
        self.map.contains_key(id)
    }

    pub fn get(&self, id: &ShapeId) -> Option<&Ref<dyn SmithyTrait>> {
        self.map.get(id)
    }

    pub fn of(traits: Vec<Ref<dyn SmithyTrait>>) -> Self {
        let mut map: TraitMap = TraitMap::new();
        for smithy_trait in traits {
            map.map.insert(smithy_trait.id().clone(), smithy_trait);
        }
        map
    }

    pub fn extend(&mut self, trait_map: &TraitMap) {
        self.map.extend(trait_map.map.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::traits::{HTTPErrorTrait, JsonNameTrait};

    #[test]
    fn basic_map_functionality() {
        let mut map = TraitMap::new();
        map.insert(JsonNameTrait::new("a"));
        let dyn_id = ShapeId::from("smithy.api#Dynamic");
        map.insert(DynamicTrait {
            id: dyn_id.clone(),
            value: DocumentValue::String("b".to_string()),
        });
        assert!(map.contains(&dyn_id));
        assert!(map.contains(JsonNameTrait::trait_id()));
    }

    #[test]
    fn map_extension() {
        let mut map_a = TraitMap::new();
        map_a.insert(JsonNameTrait::new("a"));
        let mut map_b = TraitMap::new();
        map_b.insert(HTTPErrorTrait::new(404));
        map_a.extend(&map_b);
        assert!(map_a.contains(HTTPErrorTrait::trait_id()));
    }

    #[test]
    fn trait_conversion_to_type() {
        let mut map = TraitMap::new();
        map.insert(JsonNameTrait::new("something_else"));
        let id = ShapeId::from("api.smithy#Example");
        map.insert(DynamicTrait {
            id: id.clone(),
            value: DocumentValue::Null,
        });
        assert!(map.contains(JsonNameTrait::trait_id()));
        assert!(map.contains(&id));
    }
}
