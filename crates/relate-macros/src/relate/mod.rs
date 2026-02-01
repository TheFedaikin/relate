//! The unified `relate_structs!` macro for generating From implementations.
//!
//! Supports multiple patterns:
//! - `A ~> B { fields }` - relate existing structs (forward only)
//! - `A ~ B { fields }` - bidirectional relation
//! - `#[attrs] struct A ~ #[attrs] struct B { fields }` - define and relate

mod generator;
mod parser;
mod types;

pub use generator::generate_relate_output;
pub use types::RelateInput;
