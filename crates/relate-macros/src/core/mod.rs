//! Core types and utilities shared across all macros.
//!
//! This module provides unified abstractions for field mapping and code
//! generation that are used by `relate_structs!` and `#[derive(Relate)]`.

mod codegen;
mod parse_helpers;
mod types;

pub use codegen::*;
pub use parse_helpers::*;
pub use types::*;
