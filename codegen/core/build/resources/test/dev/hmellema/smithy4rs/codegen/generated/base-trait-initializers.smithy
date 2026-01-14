$version: "2"

namespace com.test

@sparse
list WithAnnotationTrait {
    member: String
}

structure MyStruct {
    @jsonName("stuff")
    withStringTrait: String
}

// Trait that has no mappings registered
// Should default to catch-all dynamic trait
// TODO(dynamic traits): Add a test for dynamic structure trait
@trait
string genericTrait

@genericTrait("stuff")
string WithGeneric

