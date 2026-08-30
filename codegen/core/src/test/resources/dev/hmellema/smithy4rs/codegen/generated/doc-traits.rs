use smithy4rs_core::{
    derive::SmithyShape,
    prelude::STRING,
    smithy,
};

smithy!(
    /// Schema for [`DeprecatedStruct`]
    structure com::test::DeprecatedStruct {
        deprecatedMember: STRING
    }
);

#[deprecated(since = "1.0", note = "Plz dont use")]
#[derive(SmithyShape, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = DEPRECATED_STRUCT)]
pub struct DeprecatedStruct {
    #[deprecated(since = "5ever", note = "Dont use this")]
    pub deprecated_member: Option<String>,
}

smithy!(
    /// Schema for [`ExternalDocumentationStruct`]
    structure com::test::ExternalDocumentationStruct {
        string: STRING
    }
);

/// ## References
/// - [**Homepage**]("https://www.example.com/")
/// - [**API Reference**]("https://www.example.com/api-ref")
///
#[derive(SmithyShape, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = EXTERNAL_DOCUMENTATION_STRUCT)]
pub struct ExternalDocumentationStruct {
    pub string: Option<String>,
}

smithy!(
    /// Schema for [`SinceStruct`]
    structure com::test::SinceStruct {
        sinceMember: STRING
    }
);

/// <div class="note">
///
/// **Since**: 1.2.3
///
/// </div>
///
#[derive(SmithyShape, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = SINCE_STRUCT)]
pub struct SinceStruct {
    /// <div class="note">
    ///
    /// **Since**: 1.2.3
    ///
    /// </div>
    ///
    pub since_member: Option<String>,
}

smithy!(
    /// Schema for [`UnstableStructure`]
    structure com::test::UnstableStructure {
        unstableMember: STRING
    }
);

/// <div class="warning">
///
/// **WARNING**: Unstable feature
///
/// </div>
///
#[derive(SmithyShape, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = UNSTABLE_STRUCTURE)]
pub struct UnstableStructure {
    /// <div class="warning">
    ///
    /// **WARNING**: Unstable feature
    ///
    /// </div>
    ///
    pub unstable_member: Option<String>,
}
