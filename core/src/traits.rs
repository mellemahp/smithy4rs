// What is a trait really? (runtime traits at least)
// Well, at it's core a runtime trait provides flexible, but typed data for attachment to
// a given shape schema.
// Traits MUST provide access to their corresponding ShapeID and MAY contain Document-typed
// data. So:

use std::any::Any;
use std::collections::HashMap;
use std::convert::Into;
use std::sync::LazyLock;
use crate::documents::DocumentValue;
use crate::lazy_shape_id;
use crate::shapes::ShapeId;

pub trait SmithyTrait: Any {
    fn id(&self) -> &'static ShapeId;

    fn value(&self) -> &'static DocumentValue;
}

pub struct HTTPTrait {
    // Between 100 - 599
    code: i32,
    // TODO: What should this be type-wise?
    path: String,
    query: String
}
lazy_shape_id!(HTTP_TRAIT_ID, "smithy.api#httpError");
impl HTTPTrait {
    pub fn code(&self) -> i32 {
        self.code
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query(&self) -> &str {
        &self.query
    }
}
impl SmithyTrait for HTTPTrait {
    fn id(&self) -> &'static ShapeId {
        &*HTTP_TRAIT_ID
    }

    fn value(&self) -> &DocumentValue {
        todo!()
    }
}

struct HTTPErrorTrait {
    code: i32,
}
impl HTTPErrorTrait {
    fn code(&self) -> i32 {
        self.code
    }
}

struct DynamicTrait {
    id: &'static ShapeId,
    value: DocumentValue<'static>
}

impl SmithyTrait for DynamicTrait {
    fn id(&self) -> &'static ShapeId {
        self.id
    }

    fn value(&self) -> &'static DocumentValue {
        todo!()
    }
}

struct TraitMap {
    map: HashMap<&'static ShapeId, Box<dyn Any>>,
}
impl TraitMap {
    fn new() -> TraitMap {
        TraitMap { map: HashMap::new() }
    }

    // NOTE: Inserting into the
    fn insert<T: SmithyTrait>(&mut self, trait_type: T) -> Option<Box<dyn Any>> {
        self.map.insert(trait_type.id(), Box::new(trait_type))
    }

    fn get<T: SmithyTrait>(&self, id: &ShapeId) -> Option<&T> {
        self.map.get(id)
            .map(|b_ref| &**b_ref)
            // TODO: should this raise something more useful?
            .map(|any| any.downcast_ref::<T>().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    lazy_shape_id!(TEST_ID, "smithy.api#Test");

    #[test]
    fn it_works() {
        let mut map = TraitMap::new();
        let _ = map.insert(HTTPTrait { path: "struff".into(), code: 0, query: "".to_string() });
        let _ = map.insert(DynamicTrait { id: &TEST_ID, value: DocumentValue::Null});
        let output = map.get::<HTTPTrait>(&*HTTP_TRAIT_ID).unwrap();
        assert_eq!(output.path, "struff");
    }
}


// Ideally we provide nice accessors for common smithy traits,
// and even allow users to provide their own custom trait initializers.
// Let's start by looking at common smithy traits from smithy-python

// Some things

// 1. A whole bunch of annotation traits! These dont actually have a document value
// (i.e. their document value is NULL)
//