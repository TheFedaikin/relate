//! Types for the Relate derive macro.
//!
//! Re-exports core types and adds derive-specific input types.

use syn::{Generics, Ident, Type};

// Re-export core types
pub use crate::core::{CloneMode, FieldMapping, FieldSource, Transform};

/// How the conversion should be generated.
#[derive(Debug, Clone, Default)]
#[allow(clippy::large_enum_variant)] // syn::Type is large; acceptable for proc-macro
pub enum ConversionMode {
    /// Generate `From` implementations (infallible)
    #[default]
    Infallible,
    /// Generate `TryFrom` with specified error type (None = use
    /// ConversionError)
    Fallible(Option<Type>),
}

/// Parsed input for the Relate derive macro.
#[derive(Debug)]
pub struct FromDeriveInput {
    /// The target struct name (the one being derived)
    pub target_name:     Ident,
    /// The target struct's generics
    pub target_generics: Generics,
    /// The source type to convert from
    pub source_type:     Type,
    /// Whether to generate bidirectional impls
    pub bidirectional:   bool,
    /// Field mappings
    pub fields:          Vec<FieldMapping>,
    /// Struct-level clone mode (default for all fields)
    pub clone_mode:      CloneMode,
    /// Conversion mode (From vs TryFrom)
    pub conversion_mode: ConversionMode,
}
