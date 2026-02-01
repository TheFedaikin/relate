//! Generate `From` and `TryFrom` implementations between structs.
//!
//! This crate provides two ways to generate conversions:
//!
//! # `relate_structs!` - Macro for relating existing structs
//!
//! ```rust,ignore
//! use relate::relate_structs;
//!
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
//! use relate::Relate;
//!
//! pub struct SettingType {
//!     pub id: Option<i32>,
//!     pub name: String,
//!     pub label: String,
//! }
//!
//! #[derive(Relate)]
//! #[relate(SettingType)]
//! pub struct SettingTypeResponse {
//!     #[relate(_.unwrap_or(0))]
//!     pub id: i32,
//!     pub name: String,
//!     pub label: String,
//! }
//! ```
//!
//! # Fallible conversions with `TryFrom`
//!
//! ```rust,ignore
//! use relate::{relate_structs, ConversionError};
//!
//! relate_structs! {
//!     RawConfig ~>? Config {
//!         port: with = _.parse()?;
//!         host;
//!     }
//! }
//!
//! // Default error type is ConversionError
//! let config: Result<Config, ConversionError> = raw.try_into();
//! ```

mod error;

pub use error::ConversionError;
// Re-export macros when the derive feature is enabled
#[cfg(feature = "derive")]
pub use relate_macros::{Relate, relate_structs};
