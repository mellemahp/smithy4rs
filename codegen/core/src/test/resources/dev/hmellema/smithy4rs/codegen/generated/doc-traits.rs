use smithy4rs_core::{
    derive::SmithyShape,
    prelude::STRING,
    smithy,
};

smithy!("com.test#DeprecatedStruct": {
    /// Schema for [`DeprecatedStruct`]
    structure DEPRECATED_STRUCT_SCHEMA {
        DEPRECATED_MEMBER: STRING = "deprecatedMember"
    }
});

#[deprecated(since = "1.0", note = "Plz dont use")]
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(DEPRECATED_STRUCT_SCHEMA)]
pub struct DeprecatedStruct {
    #[deprecated(since = "5ever", note = "Dont use this")]
    #[smithy_schema(DEPRECATED_MEMBER)]
    pub deprecated_member: String,
}

smithy!("com.test#ExternalDocumentationStruct": {
    /// Schema for [`ExternalDocumentationStruct`]
    structure EXTERNAL_DOCUMENTATION_STRUCT_SCHEMA {
        STRING: STRING = "string"
    }
});

/// ## References
/// - [**Homepage**]("https://www.example.com/")
/// - [**API Reference**]("https://www.example.com/api-ref")
///
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(EXTERNAL_DOCUMENTATION_STRUCT_SCHEMA)]
pub struct ExternalDocumentationStruct {
    #[smithy_schema(STRING)]
    pub string: String,
}

smithy!("com.test#SinceStruct": {
    /// Schema for [`SinceStruct`]
    structure SINCE_STRUCT_SCHEMA {
        SINCE_MEMBER: STRING = "sinceMember"
    }
});

/// <div class="note">
///
/// **Since**: 1.2.3
///
/// </div>
///
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(SINCE_STRUCT_SCHEMA)]
pub struct SinceStruct {
    /// <div class="note">
    ///
    /// **Since**: 1.2.3
    ///
    /// </div>
    ///
    #[smithy_schema(SINCE_MEMBER)]
    pub since_member: String,
}

smithy!("com.test#UnstableStructure": {
    /// Schema for [`UnstableStructure`]
    structure UNSTABLE_STRUCTURE_SCHEMA {
        UNSTABLE_MEMBER: STRING = "unstableMember"
    }
});

/// <div class="warning">
///
/// **WARNING**: Unstable feature
///
/// </div>
///
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(UNSTABLE_STRUCTURE_SCHEMA)]
pub struct UnstableStructure {
    /// <div class="warning">
    ///
    /// **WARNING**: Unstable feature
    ///
    /// </div>
    ///
    #[smithy_schema(UNSTABLE_MEMBER)]
    pub unstable_member: String,
}
