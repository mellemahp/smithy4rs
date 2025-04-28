// What is a trait really? (runtime traits at least)
// Well, at it's core a runtime trait provides flexible, but typed data for attachment to
// a given shape schema.
// Traits MUST provide access to their corresponding ShapeID and MAY contain Document-typed
// data. So:

use std::any::Any;
use std::collections::HashMap;
use std::convert::Into;
use indexmap::IndexMap;
use crate::documents::{Document, DocumentValue};
use crate::shapes::ShapeId;

#[derive(Debug, Clone, PartialEq)]
pub struct SmithyTrait<'st> {
    id: ShapeId,
    value: DocumentValue<'st>
}

impl SmithyTrait<'_> {
    fn id(&self) -> &ShapeId {
        &self.id
    }

    fn value(&self) -> &DocumentValue<'_> {
        &self.value
    }
}

pub struct HTTPTrait {
    method: String,
    // TODO: What should this be type-wise?
    uri: String,
    // Between 100 - 599
    code: Option<i32>,
}
impl HTTPTrait {
    const ID: &'static str = "smithy.api#httpError";
    pub fn method(&self) -> &str {
        &self.method
    }
    pub fn uri(&self) -> &str {
        &self.uri
    }
    pub fn code(&self) -> Option<i32> {
        self.code
    }
}

impl <'a> From<HTTPTrait> for SmithyTrait<'a> {
    fn from(value: HTTPTrait) -> Self {
        let mut map: HashMap<String, Document> = HashMap::new();

        // TODO: Use into methods.
        map.insert("method".to_string(), value.method.into());
        map.insert("uri".to_string(), value.uri.into());
        if let Some(code) = value.code {
            map.insert("code".to_string(), code.into());
        }

        SmithyTrait {
            id: HTTPTrait::ID.into(),
            value: DocumentValue::Map(map)
        }
    }
}

// TODO: Should be fallible.
impl <'a> From<&'a SmithyTrait<'a>> for HTTPTrait {
    fn from(value: &'a SmithyTrait<'a>) -> Self {
        // Must have matching value!
        if (value.id != HTTPTrait::ID.into()) {
            panic!("EEK!");
        }
        if let DocumentValue::Map(map) = &value.value {
            let method: String = map.get(&"method".to_string())
                .map(|d| d.to_owned().try_into().expect("Bad!"))
                .expect("SmithyTrait didn't defined HTTP method");
            let uri: String = map.get(&"uri".to_string())
                .map(|d| d.to_owned().try_into().expect("Bad!"))
                .expect("SmithyTrait didn't defined HTTP method");
            let code: Option<i32> = map.get(&"code".to_string())
                .map(|d| d.to_owned().try_into().expect("Bad!"));
            return HTTPTrait { method, uri, code };
        }
        panic!("EEEK!");
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

struct 

struct DynamicTrait {
    id: ShapeId,
    value: DocumentValue<'static>
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMap<'map> {
    map: HashMap<ShapeId, SmithyTrait<'map>>,
}
impl <'map> TraitMap<'map> {
    pub fn new<'a>() -> TraitMap<'a> {
        TraitMap { map: HashMap::new() }
    }

    // NOTE: Inserting into the
    pub fn insert<T: Into<SmithyTrait<'map>>>(&mut self, trait_type: T) -> Option<SmithyTrait> {
        let smithy_trait = trait_type.into();
        self.map.insert(smithy_trait.id().clone(), smithy_trait)
    }

    pub fn insert_all(&mut self, trait_map: &TraitMap<'map>) {
        trait_map.map.iter().for_each(|(key, value)| {
            self.map.insert(key.clone(), value.clone());
        })
    }

    pub fn get(&self, id: &ShapeId) -> Option<&SmithyTrait> {
        self.map.get(id)
    }

    pub fn get_as<T: for <'a> From<&'a SmithyTrait<'a>>>(&self, id: &ShapeId) -> Option<T> {
        self.get(id).map(|trait_type| trait_type.into())
    }

    pub fn of<'a, T: Into<SmithyTrait<'a>>>(trait_list: Vec<T>) -> TraitMap<'a> {
        let mut map: TraitMap = TraitMap::new();
        for item in trait_list {
            map.insert(item);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut map = TraitMap::new();
        let _ = map.insert(HTTPTrait {
            method: "GET".to_string(),
            uri: "/stuff".to_string(),
            code: None,
        });
        // let _ = map.insert(DynamicTrait { id: &TEST_ID, value: DocumentValue::Null});
        let output = map.get_as::<HTTPTrait>(&HTTPTrait::ID.into()).expect("SmithyTrait Could not be converted");
        assert_eq!(output.uri, "/stuff");
    }
}

pub const EMPTY_TRAIT_LIST: Vec<SmithyTrait<'static>> = vec![];
