//! Parser for the `relate_structs!` macro.
//!
//! Supports multiple syntax patterns:
//! - `A ~ B { fields }` - bidirectional
//! - `A ~> B { fields }` - forward only
//! - `A ~>? B { fields }` - fallible forward (TryFrom)
//!
//! Field syntax uses semicolon terminators:
//! - `field;` - identity mapping
//! - `field: cloned;` - with clone mode
//! - `field: default = expr;` - default value
//! - `field: with = expr;` - transform expression

use proc_macro2::TokenStream;
use syn::{
    Error, Expr, Ident, Result, Token, braced,
    parse::{Parse, ParseStream},
    token,
};

use super::types::{
    Direction, ExistingRelation, FieldMapping, FieldSource, RelateInput, Relation, RelationBody,
    Transform, TypeRef,
};
use crate::core::{CloneMode, parse_tokens_until_terminator, parse_trailing_clone_mode};

impl Parse for RelateInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut relations = Vec::new();

        while !input.is_empty() {
            relations.push(input.parse()?);

            // Optional semicolon between relations
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
            }
        }

        Ok(Self { relations })
    }
}

impl Parse for Relation {
    fn parse(input: ParseStream) -> Result<Self> { Ok(Self(input.parse()?)) }
}

impl Parse for ExistingRelation {
    fn parse(input: ParseStream) -> Result<Self> {
        let source = input.parse()?;
        let direction = input.parse()?;
        let target = input.parse()?;

        let body = if input.peek(token::Brace) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            source,
            direction,
            target,
            body,
        })
    }
}

impl Parse for TypeRef {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;

        let generics = if input.peek(Token![<]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self { name, generics })
    }
}

impl Parse for Direction {
    fn parse(input: ParseStream) -> Result<Self> {
        // Must have ~ for all directions
        if !input.peek(Token![~]) {
            return Err(input.error(
                "Expected `~>` (forward), `~` (bidirectional), or `~>?` (fallible forward)",
            ));
        }
        input.parse::<Token![~]>()?;

        // ~ alone = bidirectional
        if !input.peek(Token![>]) {
            return Ok(Self::Bidirectional);
        }
        input.parse::<Token![>]>()?;

        // ~> without ? = forward
        if !input.peek(Token![?]) {
            return Ok(Self::Forward);
        }
        input.parse::<Token![?]>()?;

        // ~>? with optional [ErrorType]
        let error_type = if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            Some(Box::new(content.parse::<syn::Type>()?))
        } else {
            None
        };

        Ok(Self::TryForward(error_type))
    }
}

impl Parse for RelationBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);

        let mut has_spread = false;
        let mut fields = Vec::new();

        while !content.is_empty() {
            // Check for spread `..`
            if content.peek(Token![..]) {
                content.parse::<Token![..]>()?;
                has_spread = true;
                // Optional semicolon after spread
                if content.peek(Token![;]) {
                    content.parse::<Token![;]>()?;
                }
                continue;
            }

            // Parse field mapping with new syntax
            fields.push(parse_field_mapping(&content)?);

            // Semicolon terminator (required, but be lenient for trailing)
            if content.peek(Token![;]) {
                content.parse::<Token![;]>()?;
            } else if !content.is_empty() {
                return Err(Error::new(
                    content.span(),
                    "Expected `;` after field mapping",
                ));
            }
        }

        Ok(Self { has_spread, fields })
    }
}

/// Parse a single field mapping with new syntax:
/// `field;` or `field: modifier;`
///
/// Modifier can be:
/// - `cloned`, `copy`, `move` (clone mode)
/// - `default` or `default = expr`
/// - `with = expr` optionally followed by `, clone_mode`
fn parse_field_mapping(input: ParseStream) -> Result<FieldMapping> {
    // Parse field name
    let field: Ident = input.parse()?;

    // Check for modifier (`:` followed by something)
    if !input.peek(Token![:]) {
        // Simple identity mapping: `field;`
        return Ok(FieldMapping {
            target_field: field,
            source:       FieldSource::auto(),
        });
    }

    // Consume the `:`
    input.parse::<Token![:]>()?;

    // Parse modifier
    parse_field_modifier(input, field)
}

/// Parse the modifier after `field:`
fn parse_field_modifier(input: ParseStream, field: Ident) -> Result<FieldMapping> {
    // Check for `move` keyword (special handling since it's a Rust keyword)
    if input.peek(Token![move]) {
        input.parse::<Token![move]>()?;
        let mut source = FieldSource::auto();
        source.clone_mode = Some(CloneMode::Move);
        return Ok(FieldMapping {
            target_field: field,
            source,
        });
    }

    // Must be an identifier for other modifiers
    if !input.peek(Ident) {
        return Err(Error::new(
            input.span(),
            "Expected modifier after `:`. Valid modifiers:\n\
             - `cloned`, `copy`, `move` (clone mode)\n\
             - `default` or `default = expr`\n\
             - `with = expr`",
        ));
    }

    let modifier: Ident = input.parse()?;

    // Clone modes: `cloned`, `copy`
    if modifier == "cloned" {
        let mut source = FieldSource::auto();
        source.clone_mode = Some(CloneMode::Cloned);
        return Ok(FieldMapping {
            target_field: field,
            source,
        });
    }
    if modifier == "copy" {
        let mut source = FieldSource::auto();
        source.clone_mode = Some(CloneMode::Copy);
        return Ok(FieldMapping {
            target_field: field,
            source,
        });
    }

    // Default: `default` or `default = expr`
    if modifier == "default" {
        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let expr: Expr = parse_expr_until_semicolon(input)?;
            return Ok(FieldMapping {
                target_field: field,
                source:       FieldSource::default_expr(expr),
            });
        }
        return Ok(FieldMapping {
            target_field: field,
            source:       FieldSource::default_value(),
        });
    }

    // With expression: `with = expr` optionally followed by `, clone_mode`
    if modifier == "with" {
        if !input.peek(Token![=]) {
            return Err(Error::new(input.span(), "Expected `=` after `with`"));
        }
        input.parse::<Token![=]>()?;

        // Check for collection map syntax: `with = [_.field]`
        if input.peek(token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            let inner: TokenStream = content.parse()?;

            // If it starts with `.`, it's shorthand: [.id] -> [_.id]
            let inner_str = inner.to_string();
            let tokens = if inner_str.starts_with('.') {
                let underscore = Ident::new("_", proc_macro2::Span::call_site());
                quote::quote! { #underscore #inner }
            } else {
                inner
            };

            let clone_mode = parse_trailing_clone_mode(input, true)?;
            let mut source = FieldSource::with_transform(Transform::CollectionMap(tokens));
            source.clone_mode = clone_mode;
            return Ok(FieldMapping {
                target_field: field,
                source,
            });
        }

        // Regular expression
        let (tokens, fallible) = parse_tokens_until_terminator(input, true)?;
        let clone_mode = parse_trailing_clone_mode(input, true)?;
        let mut source = FieldSource::with_expr(tokens, fallible);
        source.clone_mode = clone_mode;
        return Ok(FieldMapping {
            target_field: field,
            source,
        });
    }

    Err(Error::new_spanned(
        &modifier,
        format!(
            "Unknown modifier `{}`. Valid modifiers:\n\
             - `cloned`, `copy`, `move` (clone mode)\n\
             - `default` or `default = expr`\n\
             - `with = expr`",
            modifier
        ),
    ))
}

/// Parse an expression until semicolon (for `default = expr;`)
fn parse_expr_until_semicolon(input: ParseStream) -> Result<Expr> {
    // We need to parse tokens until `;` and then parse as Expr
    // This is tricky because Expr parsing is greedy
    // For now, just parse a single Expr and hope it works
    input.parse()
}
