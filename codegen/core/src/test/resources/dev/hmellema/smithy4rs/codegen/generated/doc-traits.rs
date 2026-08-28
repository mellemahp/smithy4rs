use smithy4rs_core::{
    derive::SmithyShape,
    prelude::STRING,
    smithy,
};

smithy!("com.test#DeprecatedStruct": {
    /// Schema for [`DeprecatedStruct`]
    structure DEPRECATED_STRUCT_SCHEMA {
        deprecatedMember: STRING
    }
});

#[deprecated(since = "1.0", note = "Plz dont use")]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = DEPRECATED_STRUCT_SCHEMA)]
pub struct DeprecatedStruct {
    #[deprecated(since = "5ever", note = "Dont use this")]
    pub deprecated_member: Option<String>,
}

smithy!("com.test#ExternalDocumentationStruct": {
    /// Schema for [`ExternalDocumentationStruct`]
    structure EXTERNAL_DOCUMENTATION_STRUCT_SCHEMA {
        string: STRING
    }
});

/// ## References
/// - [**Homepage**]("https://www.example.com/")
/// - [**API Reference**]("https://www.example.com/api-ref")
///
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = EXTERNAL_DOCUMENTATION_STRUCT_SCHEMA)]
pub struct ExternalDocumentationStruct {
    pub string: Option<String>,
}

smithy!("com.test#SinceStruct": {
    /// Schema for [`SinceStruct`]
    structure SINCE_STRUCT_SCHEMA {
        sinceMember: STRING
    }
});

/// <div class="note">
///
/// **Since**: 1.2.3
///
/// </div>
///
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = SINCE_STRUCT_SCHEMA)]
pub struct SinceStruct {
    /// <div class="note">
    ///
    /// **Since**: 1.2.3
    ///
    /// </div>
    ///
    pub since_member: Option<String>,
}

smithy!("com.test#UnstableStructure": {
    /// Schema for [`UnstableStructure`]
    structure UNSTABLE_STRUCTURE_SCHEMA {
        unstableMember: STRING
    }
});

/// <div class="warning">
///
/// **WARNING**: Unstable feature
///
/// </div>
///
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = UNSTABLE_STRUCTURE_SCHEMA)]
pub struct UnstableStructure {
    /// <div class="warning">
    ///
    /// **WARNING**: Unstable feature
    ///
    /// </div>
    ///
    pub unstable_member: Option<String>,
}
