/// Serialization adapters for `serde`
mod serialization;
pub use serialization::SerAdapter;

/// Deserialization adapters for `serde`
mod deserialization;
pub use deserialization::SchemaSeed;
