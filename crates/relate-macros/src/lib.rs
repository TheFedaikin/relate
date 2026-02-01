//! Proc macros for generating `From` and `TryFrom` implementations between
//! structs.
//!
//! This crate provides two ways to generate `From`/`TryFrom` implementations:
//!
//! # `relate_structs!` - Macro for relating existing structs
//!
//! ```rust,ignore
//! relate_structs! {
//!     System ~> SystemResponse {
//!         id: with = _.unwrap_or(0);
//!         name;
//!         description;
//!     }
//! }
//! ```
//!
//! # `#[derive(Relate)]` - Derive-based (recommended for new code)
//!
//! ```rust,ignore
//! pub struct SettingType {
//!     pub id: Option<i32>,
//!     pub name: String,
//!     pub label: String,
//! }
//!
//! #[derive(Relate)]
//! #[relate(SettingType)]
//! pub struct SettingTypeResponse {
//!     #[relate(_.unwrap_or(0))]  // Transform
//!     pub id: i32,
//!     pub name: String,          // Auto-mapped (same name)
//!     pub label: String,         // Auto-mapped
//! }
//! ```

mod core;
mod from_derive;
mod relate;

use proc_macro::TokenStream;

/// Generates `From` implementations between existing structs.
///
/// This macro allows you to define field mappings and transformations between
/// two already-defined structs and generates the appropriate `From`
/// implementations.
///
/// ## Key Features
///
/// - **Auto dual impl**: Every relation generates BOTH `From<T>` AND `From<&T>`
/// - **Semicolon terminator**: Fields end with `;` not `,`
/// - **Unified `with =` syntax**: All transforms use `field: with = expr;`
/// - **Collection mapping**: `field: with = [_.id];` - map over collections
/// - **Generics support**: Works with generic structs (need `Clone` bound)
///
/// ## Direction Operators
///
/// - `~>` : Generate `From<Source>` + `From<&Source>` for Target (forward)
/// - `~` : Generate all 4 impls (both directions, owned + ref) (bidirectional)
/// - `~>?` : Generate `TryFrom<Source>` + `TryFrom<&Source>` (fallible forward)
/// - `~>?[E]` : Same as `~>?` but with custom error type `E`
///
/// ## Field Syntax
///
/// All fields end with semicolon (`;`):
///
/// - `field;` - Copy field with same name
/// - `field: cloned;` - Same-name with clone mode
/// - `field: copy;` - Same-name, no clone (asserts Copy)
/// - `field: move;` - Same-name, explicit move
/// - `field: default;` - Use `Default::default()`
/// - `field: default = expr;` - Use specific default value
/// - `tgt: with = .src;` - Rename (access different source field)
/// - `field: with = _.method();` - Method call on same-named field
/// - `field: with = .x + .y;` - Expression with source field access
/// - `field: with = expr?;` - Fallible transform (triggers TryFrom)
/// - `field: with = [_.x];` - Collection map
/// - `field: with = expr, cloned;` - Transform with clone mode
///
/// Inside `with = expr`:
/// - `_` expands to `src.<target_field_name>` (same-named source field)
/// - `.field` accesses `src.field` (any source field by name)
///
/// ## Examples
///
/// ```rust,ignore
/// // Bidirectional - generates all 4 From impls
/// relate_structs! {
///     Barcodes ~ DbBarcodes {
///         ean13;
///         ean8;
///         code128;
///     }
/// }
///
/// // Method transforms
/// relate_structs! {
///     System ~> SystemResponse {
///         id: with = _.unwrap_or(0);
///         name: with = _.to_uppercase();
///         description;
///     }
/// }
///
/// // Collection mapping
/// relate_structs! {
///     ProductWithVariants ~> Product {
///         id;
///         name;
///         variant_ids: with = [_.id.clone()];  // Vec<Variant> -> Vec<String>
///     }
/// }
///
/// // Renames and defaults
/// relate_structs! {
///     Store ~> Warehouse {
///         moysklad_id: with = .id;
///         name;
///         insales_id: default = None;
///         should_sync: default = false;
///     }
/// }
///
/// // Generic structs (Clone bound needed for auto ref impl)
/// relate_structs! {
///     Container<T: Clone> ~> Wrapper<T: Clone> {
///         inner: with = .value;
///     }
/// }
///
/// // Fallible conversion with TryFrom
/// relate_structs! {
///     RawConfig ~>? Config {
///         port: with = _.parse()?;    // String -> u16, can fail
///         host;                        // Infallible copy
///     }
/// }
///
/// // TryFrom with custom error type
/// relate_structs! {
///     Input ~>?[MyError] Output {
///         value: with = _.parse()?;
///     }
/// }
/// ```
#[proc_macro]
pub fn relate_structs(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as relate::RelateInput);

    match relate::generate_relate_output(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive macro for generating `From` implementations between related structs.
///
/// Place on the target struct with `#[relate(SourceType)]` to generate
/// `From<Source>` and `From<&Source>` implementations.
///
/// ## Features
///
/// - **Auto-mapping**: Fields without `#[relate(...)]` are mapped by same name
/// - **Auto dual impl**: Generates both `From<T>` and `From<&T>`
/// - **Bidirectional**: Use `#[relate(Source, both)]` for both directions
///
/// ## Field Attributes
///
/// - No attribute: Auto-map from same-named field in source
/// - `#[relate(source_field)]`: Rename - map from different field
/// - `#[relate(.method())]`: Transform with method call
/// - `#[relate(source_field, .method())]`: Rename + transform
/// - `#[relate([.field.clone()])]`: Collection map
/// - `#[relate(|x: T| expr)]`: Transform with closure
/// - `#[relate(path::to::fn)]`: Transform with function
/// - `#[relate(default)]`: Use `Default::default()`
/// - `#[relate(default = expr)]`: Use specific default
/// - `#[relate(skip)]`: Same as default
///
/// ## Examples
///
/// ```rust,ignore
/// // Basic usage - fields auto-map by name
/// #[derive(Relate)]
/// #[relate(SettingType)]
/// pub struct SettingTypeResponse {
///     #[relate(.unwrap_or(0))]
///     pub id: i32,
///     pub name: String,   // Auto-mapped
///     pub label: String,  // Auto-mapped
/// }
///
/// // Bidirectional
/// #[derive(Relate)]
/// #[relate(DbBarcodes, both)]
/// pub struct Barcodes {
///     pub ean13: Option<String>,
///     pub ean8: Option<String>,
/// }
///
/// // With defaults
/// #[derive(Relate)]
/// #[relate(Store)]
/// pub struct Warehouse {
///     #[relate(id)]  // Rename: Store.id -> Warehouse.moysklad_id
///     pub moysklad_id: String,
///     pub name: String,
///     #[relate(default = None)]
///     pub insales_id: Option<i64>,
///     #[relate(default = false)]
///     pub should_sync: bool,
/// }
/// ```
#[proc_macro_derive(Relate, attributes(relate))]
pub fn derive_relate(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match from_derive::parse_from_derive(input) {
        Ok(parsed) => from_derive::generate_from_derive(&parsed).into(),
        Err(err) => err.to_compile_error().into(),
    }
}
