mod adapter;
pub mod builders;
pub mod correction;
pub mod debug;
pub mod deserializers;
pub mod documents;
pub mod serializers;
pub mod unit;
pub mod validation;

pub use builders::{Buildable, ShapeBuilder};
pub use deserializers as de;
pub use serializers as se;
pub use validation as validate;
