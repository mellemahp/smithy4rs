namespace com.test

// Trait that has no mappings registered
// Should default to catch-all dynamic trait
@trait
structure myAnnotationTrait {}

@myAnnotationTrait
string MyString