#![allow(dead_code)]

// TODO: Could this be made more efficient?
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeId {
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub member: Option<String>,
}

impl From<&str> for ShapeId {
    fn from(value: &str) -> Self {
        let split = value.split_once("#").expect("Invalid Shape Id");
        let (namespace, mut base_name) = split;
        let mut member = None;
        if let Some((name, split_member)) = base_name.split_once("$") {
            base_name = name;
            member = Some(split_member.to_string());
        }
        ShapeId {
            id: value.to_string(),
            namespace: namespace.to_string(),
            name: base_name.to_string(),
            member,
        }
    }
}

impl ShapeId {
    pub fn from_parts<'a>(namespace: &'a str, name: &'a str, member: Option<&'a str>) -> ShapeId {
        let mut id = namespace.to_string() + "#" + name;
        if let Some(m) = member {
            id = id + "$" + m;
        }
        ShapeId {
            id,
            namespace: namespace.to_string(),
            name: name.to_string(),
            member: member.map(ToString::to_string),
        }
    }

    pub fn with_member(&self, member: &str) -> ShapeId {
        Self::from_parts(&self.namespace, &self.name, Some(member))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShapeType {
    Blob,
    Boolean,
    String,
    Timestamp,
    Byte,
    Short,
    Integer,
    Long,
    Float,
    Double,
    BigInteger,
    BigDecimal,
    Document,
    Enum,
    IntEnum,
    List,
    Map,
    Structure,
    Union,
    Member,
    Service,
    Resource,
    Operation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_id_from_str() {
        let shape_id = ShapeId::from("com.example#MyShape");
        assert_eq!(shape_id.namespace, "com.example");
        assert_eq!(shape_id.name, "MyShape");
        assert_eq!(shape_id.member.as_ref(), None);
    }

    #[test]
    fn shape_id_from_str_with_member() {
        let shape_id = ShapeId::from("com.example#MyShape$member");
        assert_eq!(shape_id.namespace, "com.example");
        assert_eq!(shape_id.name, "MyShape");
        assert_eq!(shape_id.member.unwrap(), "member");
    }

    #[test]
    #[should_panic(expected = "Invalid Shape Id")]
    fn invalid_id_from_str() {
        let _ = ShapeId::from("com.example.no.shape");
    }

    #[test]
    fn shape_id_from_parts() {
        let shape_id = ShapeId::from_parts("com.example", "MyShape", Some("member"));
        assert_eq!(shape_id.namespace, "com.example");
        assert_eq!(shape_id.name, "MyShape");
        assert_eq!(shape_id.member.unwrap(), "member");
        assert_eq!(shape_id.id, "com.example#MyShape$member");
    }

    #[test]
    fn shape_id_with_member() {
        let shape_id_base = ShapeId::from_parts("com.example", "MyShape", None);
        let shape_id = shape_id_base.with_member("member");
        assert_eq!(shape_id.namespace, "com.example");
        assert_eq!(shape_id.name, "MyShape");
        assert_eq!(shape_id.member.unwrap(), "member");
        assert_eq!(shape_id.id, "com.example#MyShape$member");
    }
}
