//! AST types for the `relate_structs!` macro.
//!
//! Re-exports core types and adds macro-specific input types.

use syn::{Generics, Ident, Type};

// Re-export core types
pub use crate::core::{FieldMapping, FieldSource, Transform};

/// The complete parsed input to the `relate_structs!` macro.
#[derive(Debug)]
pub struct RelateInput {
    /// The relation definitions
    pub relations: Vec<Relation>,
}

/// A single relation between two types.
#[derive(Debug)]
pub struct Relation(pub ExistingRelation);

/// Relation between two existing structs.
#[derive(Debug)]
pub struct ExistingRelation {
    /// Source type (can include generics)
    pub source:    TypeRef,
    /// Direction of the relation
    pub direction: Direction,
    /// Target type
    pub target:    TypeRef,
    /// Field mappings (using core `FieldMapping` type)
    pub body:      Option<RelationBody>,
}

/// A type reference with optional generics.
#[derive(Debug)]
pub struct TypeRef {
    pub name:     Ident,
    pub generics: Option<Generics>,
}

/// The body of a relation with field mappings.
#[derive(Debug)]
pub struct RelationBody {
    /// Whether spread `..` is present (auto-map remaining fields)
    pub has_spread: bool,
    /// Field mappings using the unified `FieldMapping` type
    pub fields:     Vec<FieldMapping>,
}

/// Direction of the From/TryFrom implementation generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    /// `~>` Generate `From<Source> for Target`
    Forward,
    /// `~` Generate both directions (From)
    Bidirectional,
    /// `~>?` Generate `TryFrom<Source> for Target` with default error type
    /// `~>?[E]` Generate `TryFrom<Source> for Target` with custom error type E
    TryForward(Option<Box<Type>>),
}
