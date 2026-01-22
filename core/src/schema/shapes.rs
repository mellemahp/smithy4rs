//! # Shapes
//! Definition of the [types](https://smithy.io/2.0/spec/simple-types.html) that
//! compose the Smithy data model.
//!

use std::{
    fmt::{Debug, Display, Formatter},
    hash::Hash,
};

use fast_str::FastStr;

use crate::schema::SchemaRef;

/// Immutable identifier for a shape in a Smithy model.
///
/// A shape ID is constructed from an absolute or relative shape
/// reference. A shape reference has the following structure:
///
/// ```<NAMESPACE>#<NAME>$<MEMBER>```
///
/// The member value is optional.
#[derive(Clone, PartialEq, Eq)]
pub struct ShapeId {
    id: FastStr,
    namespace: FastStr,
    name: FastStr,
    member: Option<FastStr>,
}
impl Hash for ShapeId {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialOrd for ShapeId {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ShapeId {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
impl Debug for ShapeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id)
    }
}

impl From<&str> for ShapeId {
    fn from(value: &str) -> Self {
        let split = value.split_once('#').expect("Invalid Shape Id");
        let (namespace, mut base_name) = split;
        let mut member = None;
        if let Some((name, split_member)) = base_name.split_once('$') {
            base_name = name;
            member = Some(FastStr::from_ref(split_member));
        }
        ShapeId {
            id: FastStr::from_ref(value),
            namespace: FastStr::from_ref(namespace),
            name: FastStr::from_ref(base_name),
            member,
        }
    }
}

impl ShapeId {
    /// Creates a shape ID from parts of a shape ID.
    pub fn from_parts(namespace: &str, name: &str, member: Option<&str>) -> ShapeId {
        let mut id = namespace.to_string() + "#" + name;
        if let Some(m) = member {
            id = id + "$" + m;
        }
        ShapeId {
            id: FastStr::from(id),
            namespace: FastStr::from_ref(namespace),
            name: FastStr::from_ref(name),
            member: member.map(FastStr::from_ref),
        }
    }

    /// Creates a new member Shape ID as a child of this Shape ID.
    #[must_use]
    pub fn with_member(&self, member: &str) -> ShapeId {
        Self::from_parts(&self.namespace, &self.name, Some(member))
    }

    /// Get the ID as a string slice.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the name part of this Shape ID.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the namespace part of this Shape ID.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Get the member part of this Shape ID if present.
    #[must_use]
    pub fn member(&self) -> Option<&str> {
        self.member.as_deref()
    }
}

/// Smithy shape type representing one of the shapes in the
/// [Smithy Data Model](https://smithy.io/2.0/spec/model.html#shapes)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ShapeType {
    /// [`blob`](https://smithy.io/2.0/spec/simple-types.html#blob) type
    Blob,
    /// [`boolean`](https://smithy.io/2.0/spec/simple-types.html#boolean) type
    Boolean,
    /// [`string`](https://smithy.io/2.0/spec/simple-types.html#string) type
    String,
    /// [`timestamp`](https://smithy.io/2.0/spec/simple-types.html#timestamp) type
    Timestamp,
    /// [`byte`](https://smithy.io/2.0/spec/simple-types.html#byte) type
    Byte,
    /// [`short`](https://smithy.io/2.0/spec/simple-types.html#short) type
    Short,
    /// [`integer`](https://smithy.io/2.0/spec/simple-types.html#integer) type
    Integer,
    /// [`long`](https://smithy.io/2.0/spec/simple-types.html#long) type
    Long,
    /// [`float`](https://smithy.io/2.0/spec/simple-types.html#float) type
    Float,
    /// [`double`](https://smithy.io/2.0/spec/simple-types.html#double) type
    Double,
    /// [`bigInteger`](https://smithy.io/2.0/spec/simple-types.html#bitInteger) type
    BigInteger,
    /// [`bigDecimal`](https://smithy.io/2.0/spec/simple-types.html#bigDecimal) type
    BigDecimal,
    /// [`document`](https://smithy.io/2.0/spec/simple-types.html#document) type
    Document,
    /// [`enum`](https://smithy.io/2.0/spec/simple-types.html#enum) type
    Enum,
    /// [`intEnum`](https://smithy.io/2.0/spec/simple-types.html#IntEnum) type
    IntEnum,
    /// [`list`](https://smithy.io/2.0/spec/aggregate-types.html#list) type
    List,
    /// [`map`](https://smithy.io/2.0/spec/aggregate-types.html#map) type
    Map,
    /// [`structure`](https://smithy.io/2.0/spec/aggregate-types.html#structure) type
    Structure,
    /// [`union`](https://smithy.io/2.0/spec/aggregate-types.html#union) type
    Union,
    /// Member of an aggregate type (i.e. list, map, structure, union)
    Member,
    /// [`service`](https://smithy.io/2.0/spec/service-types.html#service) type
    Service,
    /// [`resource`](https://smithy.io/2.0/spec/service-types.html#resource) type
    Resource,
    /// [`operation`](https://smithy.io/2.0/spec/service-types.html#operation) type
    Operation,
}

impl Display for ShapeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ShapeType::Blob => write!(f, "Blob"),
            ShapeType::Boolean => write!(f, "Boolean"),
            ShapeType::String => write!(f, "String"),
            ShapeType::Timestamp => write!(f, "Timestamp"),
            ShapeType::Byte => write!(f, "Byte"),
            ShapeType::Short => write!(f, "Short"),
            ShapeType::Integer => write!(f, "Integer"),
            ShapeType::Long => write!(f, "Long"),
            ShapeType::Float => write!(f, "Float"),
            ShapeType::Double => write!(f, "Double"),
            ShapeType::BigInteger => write!(f, "BigInteger"),
            ShapeType::BigDecimal => write!(f, "BigDecimal"),
            ShapeType::Document => write!(f, "Document"),
            ShapeType::Enum => write!(f, "Enum"),
            ShapeType::IntEnum => write!(f, "IntEnum"),
            ShapeType::List => write!(f, "List"),
            ShapeType::Map => write!(f, "Map"),
            ShapeType::Structure => write!(f, "Structure"),
            ShapeType::Union => write!(f, "Union"),
            ShapeType::Member => write!(f, "Member"),
            ShapeType::Service => write!(f, "Service"),
            ShapeType::Resource => write!(f, "Resource"),
            ShapeType::Operation => write!(f, "Operation"),
        }
    }
}

/// Returns the schema for a shape instance.
///
/// ## Blanket Implementations
/// This trait is automatically implemented for any
/// shapes that implement [`StaticSchemaShape`], returning the
/// static schema reference as the instance schema reference.
pub trait SchemaShape {
    /// Get a reference to the Schema of this shape.
    fn schema(&self) -> &SchemaRef;
}

impl<T: StaticSchemaShape> SchemaShape for T {
    fn schema(&self) -> &SchemaRef {
        Self::schema()
    }
}

/// Returns a static schema for a shape (type-level).
///
/// This trait is for shapes with statically-defined schemas, which are
/// those would be generated from a model.
pub trait StaticSchemaShape {
    /// Get a reference to the (static) Schema of this shape.
    fn schema() -> &'static SchemaRef;
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
