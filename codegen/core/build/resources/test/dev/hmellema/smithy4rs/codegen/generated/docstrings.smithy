$version: "2"

namespace com.test

/// A Documented Structure
structure DocumentedStruct {
    /// Documented! Yay!
    documentedMember: String
}

/// A Documented Union
union DocumentedUnion {
    /// A String variant
    variantA: String
    /// An integer variant
    variantB: Integer
}

/// A Documented Enum
enum DocumentedEnum {
    ONE = "one"
    TWO = "two"
}

/// A Documented IntEnum
intEnum DocumentedIntEnum {
    ONE = 1
    TWO = 2
}

/// Documented Map
map DocumentedMap {
    key: String
    value: String
}

/// Documented List
list DocumentedList {
    member: String
}

/// Documented Scalar
string DocumentedScalar
