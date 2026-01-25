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
