//! Test utilities and common test schemas + shapes

// TODO(verify): We don't add required to anything yet
// TODO(test): Add constraint test shapes once we have validation

mod basic_types;
mod enums;
mod nested;
mod recursive;
mod unions;

pub use basic_types::*;
pub use enums::*;
pub use nested::*;
pub use recursive::*;
pub use unions::*;
