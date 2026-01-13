$version: "2"

namespace com.test

@deprecated(since: "1.0", message: "Plz dont use")
structure DeprecatedStruct {
    @deprecated(since: "5ever", message: "Dont use this")
    deprecatedMember: String
}

@since("1.2.3")
structure SinceStruct {
    @since("1.2.3")
    sinceMember: String
}

@externalDocumentation(
    "Homepage": "https://www.example.com/"
    "API Reference": "https://www.example.com/api-ref"
)
structure ExternalDocumentationStruct {
    string: String
}

@unstable
structure UnstableStructure {
    @unstable
    unstableMember: String
}