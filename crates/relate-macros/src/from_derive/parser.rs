//! Parser for the Relate derive macro attributes.

use proc_macro2::TokenStream;
use syn::{
    Attribute, DeriveInput, Error, Expr, Fields, Ident, Meta, Result, Token, Type, parse::Parse,
};

use super::types::{
    CloneMode, ConversionMode, FieldMapping, FieldSource, FromDeriveInput, Transform,
};
use crate::core::{parse_tokens_until_terminator, parse_trailing_clone_mode};

/// Parse a `DeriveInput` into `FromDeriveInput`.
pub fn parse_from_derive(input: DeriveInput) -> Result<FromDeriveInput> {
    let target_name = input.ident;
    let target_generics = input.generics;

    // Parse #[relate(SourceType)] or #[relate(SourceType, both, cloned)] attribute
    let relate_attr = parse_from_attr(&input.attrs)?;

    // Parse fields
    let fields = match input.data {
        syn::Data::Struct(data) => parse_fields(data.fields)?,
        _ => {
            return Err(Error::new_spanned(
                target_name,
                "Relate derive only supports structs",
            ));
        }
    };

    // Determine conversion mode: explicit try_from/error type, auto-detect from
    // fields, or infallible
    let conversion_mode =
        determine_conversion_mode(&fields, relate_attr.error_type, relate_attr.force_try_from);

    Ok(FromDeriveInput {
        target_name,
        target_generics,
        source_type: relate_attr.source_type,
        bidirectional: relate_attr.bidirectional,
        fields,
        clone_mode: relate_attr.clone_mode,
        conversion_mode,
    })
}

/// Determine the conversion mode based on explicit markers, fields, and error
/// type.
fn determine_conversion_mode(
    fields: &[FieldMapping],
    explicit_error: Option<Type>,
    force_try_from: bool,
) -> ConversionMode {
    // If explicit error type provided, it's TryFrom with that error
    if explicit_error.is_some() {
        return ConversionMode::Fallible(explicit_error);
    }

    // If try_from keyword was used explicitly, force TryFrom (with default error)
    if force_try_from {
        return ConversionMode::Fallible(None);
    }

    // Auto-detect: scan for fallible transforms (containing `?`)
    let has_fallible = fields.iter().any(|f| f.source.transform.is_fallible());

    if has_fallible {
        ConversionMode::Fallible(None) // Use default ConversionError
    } else {
        ConversionMode::Infallible
    }
}

/// Parse the #[relate(...)] attribute on the struct.
fn parse_from_attr(attrs: &[Attribute]) -> Result<RelateAttr> {
    for attr in attrs {
        if attr.path().is_ident("relate") {
            return attr.parse_args();
        }
    }

    Err(Error::new(
        proc_macro2::Span::call_site(),
        "Missing #[relate(SourceType)] attribute.\n\
         Add `#[relate(SourceType)]` above your struct, where SourceType is the struct to convert from.\n\
         Example: #[relate(User)] or #[relate(User, both)] for bidirectional",
    ))
}

/// Parsed struct-level #[relate(...)] attribute.
///
/// Supports:
/// - `#[relate(SourceType)]`
/// - `#[relate(SourceType, both)]`
/// - `#[relate(SourceType, cloned)]`
/// - `#[relate(SourceType, move)]`
/// - `#[relate(SourceType, try_from)]`
/// - `#[relate(SourceType, error = MyError)]`
/// - Combinations: `#[relate(SourceType, both, cloned, error = MyError)]`
struct RelateAttr {
    source_type:    Type,
    bidirectional:  bool,
    clone_mode:     CloneMode,
    error_type:     Option<Type>,
    force_try_from: bool,
}

impl Parse for RelateAttr {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let source_type: Type = input.parse()?;

        let mut bidirectional = false;
        let mut clone_mode = CloneMode::Auto;
        let mut error_type = None;
        let mut force_try_from = false;

        // Parse optional modifiers
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            // Handle `move` keyword specially since it's a reserved keyword
            if input.peek(Token![move]) {
                input.parse::<Token![move]>()?;
                clone_mode = CloneMode::Move;
                continue;
            }

            // Check for other options (identifiers)
            if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                let ident_str = ident.to_string();

                match ident_str.as_str() {
                    "both" => bidirectional = true,
                    "cloned" => clone_mode = CloneMode::Cloned,
                    "copy" => clone_mode = CloneMode::Copy,
                    "error" => {
                        input.parse::<Token![=]>()?;
                        error_type = Some(input.parse()?);
                    }
                    "try_from" => {
                        // Explicit try_from marker forces TryFrom generation
                        // Optionally with `= ErrorType` for custom error
                        if input.peek(Token![=]) {
                            input.parse::<Token![=]>()?;
                            error_type = Some(input.parse()?);
                        }
                        // Always force TryFrom when keyword is present
                        force_try_from = true;
                    }
                    _ => {
                        let msg = format!(
                            "Unknown option `{ident}`.\n\
                             Valid options: `both`, `cloned`, `copy`, `move`, `try_from`, `error = Type`\n\
                             Example: #[relate(SourceType, both, cloned)]"
                        );
                        return Err(Error::new_spanned(ident, msg));
                    }
                }
            } else {
                break;
            }
        }

        Ok(Self {
            source_type,
            bidirectional,
            clone_mode,
            error_type,
            force_try_from,
        })
    }
}

/// Parse struct fields and their #[relate(...)] attributes.
fn parse_fields(fields: Fields) -> Result<Vec<FieldMapping>> {
    let Fields::Named(named) = fields else {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "Relate derive only supports structs with named fields",
        ));
    };

    named
        .named
        .into_iter()
        .map(|field| {
            // SAFETY: We validated this is a named struct above, so ident is always present
            let target_field = field.ident.expect("named fields always have identifiers");
            let source = parse_field_from_attr(&field.attrs)?;

            Ok(FieldMapping {
                target_field,
                source,
            })
        })
        .collect()
}

/// Parse the #[relate(...)] attribute on a field.
fn parse_field_from_attr(attrs: &[Attribute]) -> Result<FieldSource> {
    for attr in attrs {
        if attr.path().is_ident("relate") {
            return parse_field_source(attr);
        }
    }

    // No attribute = auto-map by same name
    Ok(FieldSource::auto())
}

/// Parse the content of a field's #[relate(...)] attribute.
fn parse_field_source(attr: &Attribute) -> Result<FieldSource> {
    let Meta::List(list) = &attr.meta else {
        // #[relate] with no args = auto
        return Ok(FieldSource::auto());
    };

    let tokens = &list.tokens;

    // Handle special single-token keywords using structured parsing
    // Both "default" and "skip" mean the same: use Default::default()
    if let Ok(ident) = syn::parse2::<Ident>(tokens.clone()) {
        if ident == "default" || ident == "skip" {
            return Ok(FieldSource::default_value());
        }
    }

    // Parse the content
    syn::parse2::<FieldSourceContent>(tokens.clone()).map(|c| c.source)
}

/// Helper to parse field source content.
struct FieldSourceContent {
    source: FieldSource,
}

impl Parse for FieldSourceContent {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        // Handle `move` keyword first (reserved, needs special handling)
        if input.peek(Token![move]) {
            input.parse::<Token![move]>()?;
            let mut source = FieldSource::auto();
            source.clone_mode = Some(CloneMode::Move);
            return Ok(Self { source });
        }

        // Check for collection map: [_.field.method()] or [.field] shorthand
        if input.peek(syn::token::Bracket) {
            return parse_collection_map(input);
        }

        // Check for chained access: `.path.field` or `_.method()` or `.path._`
        if input.peek(Token![.]) || input.peek(Token![_]) {
            let (tokens, fallible) = parse_tokens_until_terminator(input, false)?;
            let clone_mode = parse_trailing_clone_mode(input, false)?;
            let mut source = FieldSource::with_transform(Transform::WithExpr(tokens, fallible));
            source.clone_mode = clone_mode;
            return Ok(Self { source });
        }

        // Check for identifier-based syntax: default, with, cloned, copy
        if !input.peek(Ident) {
            return Err(Error::new(
                input.span(),
                "Invalid #[relate(...)] syntax.\n\
                 Valid options:\n\
                 - `.field` or `.nested.field` - access source field\n\
                 - `_.method()` - call method on same-named field\n\
                 - `with = expr` - complex expression using `.field` or `_`\n\
                 - `default` or `default = expr` - use default value\n\
                 - `[_.field]` - map over collection\n\
                 Example: #[relate(.data.name)] or #[relate(with = .a + .b)]",
            ));
        }

        let ident: Ident = input.fork().parse()?;

        if ident == "default" {
            input.parse::<Ident>()?; // consume "default"
            if !input.peek(Token![=]) {
                return Ok(Self {
                    source: FieldSource::default_value(),
                });
            }
            input.parse::<Token![=]>()?;
            let expr: Expr = input.parse()?;
            return Ok(Self {
                source: FieldSource::default_expr(expr),
            });
        }

        if ident == "with" {
            input.parse::<Ident>()?; // consume "with"
            input.parse::<Token![=]>()?;
            let (tokens, fallible) = parse_tokens_until_terminator(input, false)?;
            let clone_mode = parse_trailing_clone_mode(input, false)?;
            let mut source = FieldSource::with_expr(tokens, fallible);
            source.clone_mode = clone_mode;
            return Ok(Self { source });
        }

        if ident == "cloned" {
            input.parse::<Ident>()?;
            let mut source = FieldSource::auto();
            source.clone_mode = Some(CloneMode::Cloned);
            return Ok(Self { source });
        }

        if ident == "copy" {
            input.parse::<Ident>()?;
            let mut source = FieldSource::auto();
            source.clone_mode = Some(CloneMode::Copy);
            return Ok(Self { source });
        }

        // Unknown identifier
        Err(Error::new_spanned(
            &ident,
            format!(
                "Unknown modifier `{}`.\n\
                 Valid options: `default`, `with`, `cloned`, `copy`",
                ident
            ),
        ))
    }
}

/// Parse collection map syntax: `[_.field]` or `[.field]` shorthand
fn parse_collection_map(input: syn::parse::ParseStream) -> Result<FieldSourceContent> {
    let content;
    syn::bracketed!(content in input);
    let inner: TokenStream = content.parse()?;
    let inner_str = inner.to_string();

    // If it starts with `.`, it's shorthand: [.id.clone()] -> [_.id.clone()]
    let tokens = if inner_str.starts_with('.') {
        let underscore = syn::Ident::new("_", proc_macro2::Span::call_site());
        quote::quote! { #underscore #inner }
    } else {
        inner
    };

    let clone_mode = parse_trailing_clone_mode(input, false)?;
    let mut source = FieldSource::with_transform(Transform::CollectionMap(tokens));
    source.clone_mode = clone_mode;
    Ok(FieldSourceContent { source })
}
