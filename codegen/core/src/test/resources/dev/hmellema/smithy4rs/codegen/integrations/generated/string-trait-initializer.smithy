namespace com.test

// Trait that has no mappings registered
// Should default to catch-all dynamic trait
@trait
string genericTrait

@genericTrait("stuff")
string WithGeneric