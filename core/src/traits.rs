use std::any::Any;
use std::collections::HashMap;
use std::convert::Into;
use std::hash::Hash;
use std::sync::{Arc, LazyLock};
use downcast_rs::{impl_downcast, Downcast, DowncastSend, DowncastSync};
use crate::documents::{DocumentValue, NumberInteger, NumberValue};
use crate::lazy_shape_id;
use crate::shapes::ShapeId;

pub trait SmithyTrait: DowncastSync {
    fn id(&self) -> &ShapeId;
    fn value(&self) -> &DocumentValue;
}
impl_downcast!(sync SmithyTrait);

pub trait StaticTraitId {
    fn trait_id() -> &'static ShapeId;
}

lazy_shape_id!(HTTP_CODE_ID, "smithy.api#httpError");
pub struct HttpCode {
    code: i32,
    value: DocumentValue<'static>
}
impl HttpCode {
    pub fn new(code: i32) -> Self {
        HttpCode {
            code,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(code)))
        }
    }

    pub fn code(&self) -> i32 {
        self.code
    }
}
impl StaticTraitId for HttpCode {
    fn trait_id() -> &'static ShapeId {
        &HTTP_CODE_ID
    }
}
impl SmithyTrait for HttpCode {
    fn id(&self) -> &'static ShapeId {
        HttpCode::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

struct DynamicTrait {
    id: ShapeId,
    value: DocumentValue<'static>
}
impl SmithyTrait for DynamicTrait {
    fn id(&self) -> &ShapeId {
        &self.id
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

pub type TraitList = Vec<Arc<dyn SmithyTrait>>;

#[derive(Clone)]
pub(crate) struct TraitMap {
    map: HashMap<ShapeId, Arc<dyn SmithyTrait>>,
}
impl TraitMap {
    pub fn new() -> TraitMap {
        TraitMap { map: HashMap::new() }
    }

    pub fn insert<T: SmithyTrait>(&mut self, value: T) -> Option<Arc<dyn SmithyTrait>> {
        self.map.insert(value.id().clone(), Arc::new(value))
    }

    pub fn contains(&self, id: &ShapeId) -> bool {
        self.map.contains_key(id)
    }

    pub fn get(&self, id: &ShapeId) -> Option<&Arc<dyn SmithyTrait>> {
        self.map.get(id)
    }

    pub fn of(traits: Vec<Arc<dyn SmithyTrait>>) -> TraitMap {
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

    #[test]
    fn it_works() {
        let mut map = TraitMap::new();
        map.insert(HttpCode::new(10));
        let id = ShapeId::from("api.smithy#Example");
        map.insert(DynamicTrait { id: id.clone(), value: DocumentValue::Null });
        assert!(map.contains(HttpCode::trait_id()));
        assert!(map.contains(&id));
    }
}