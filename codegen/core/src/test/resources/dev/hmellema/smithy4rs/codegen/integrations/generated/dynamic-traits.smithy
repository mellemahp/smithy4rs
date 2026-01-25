namespace com.test

@trait
integer intTrait

@trait
float floatTrait

@trait
string stringTrait

@trait
list stringListTrait {
    member: String
}

@trait
list otherListTrait {
    member: Integer
}

@trait
structure myCustomStruct {
    a: String
    b: Integer
}


@intTrait(1)
@floatTrait(2.0)
@stringTrait("stuff")
@stringListTrait(["a", "b", "c"])
@otherListTrait([1,2,3])
@myCustomStruct(a: "str", b: 2)
structure AppliedTo {}