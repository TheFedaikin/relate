//! Derive macro for generating `From` implementations.
//!
//! This module provides a derive macro that generates `From<Source>` and
//! `From<&Source>` implementations for a struct based on attributes.

mod generator;
mod parser;
mod types;

pub use generator::generate_from_derive;
pub use parser::parse_from_derive;
