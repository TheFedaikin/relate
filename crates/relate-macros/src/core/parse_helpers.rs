//! Shared parsing utilities for token collection.

use proc_macro2::{TokenStream, TokenTree};
use syn::{Error, Ident, Result, Token, parse::ParseStream};

use super::CloneMode;

/// Check if we're at a terminator position.
fn is_at_terminator(input: ParseStream, check_semicolon: bool) -> bool {
    if input.is_empty() {
        return true;
    }
    if input.peek(Token![,]) {
        return true;
    }
    if check_semicolon && input.peek(Token![;]) {
        return true;
    }
    false
}

/// Parse tokens until a terminator is found, detecting trailing `?` for
/// fallibility.
///
/// This is a shared implementation used by both `relate_structs!` and
/// `#[derive(Relate)]` for parsing expressions that may end with `?` to
/// indicate fallibility.
///
/// # Arguments
/// * `input` - The parse stream to read from
/// * `check_semicolon` - If true, also stops at `;` (used by relate_structs!)
///
/// # Returns
/// A tuple of (collected tokens, is_fallible)
pub fn parse_tokens_until_terminator(
    input: ParseStream,
    check_semicolon: bool,
) -> Result<(TokenStream, bool)> {
    let mut tokens = TokenStream::new();
    let mut fallible = false;

    while !is_at_terminator(input, check_semicolon) {
        // Check if this is a trailing `?`
        if input.peek(Token![?]) {
            let fork = input.fork();
            fork.parse::<Token![?]>()?;
            if is_at_terminator(&fork, check_semicolon) {
                // This is a trailing `?`, consume it and mark as fallible
                input.parse::<Token![?]>()?;
                fallible = true;
                break;
            }
        }
        let tt: TokenTree = input.parse()?;
        tokens.extend(std::iter::once(tt));
    }

    Ok((tokens, fallible))
}

/// Parse an optional trailing clone mode after a comma.
///
/// This handles the common pattern of `, cloned`, `, copy`, or `, move` after
/// an expression. Used by both `relate_structs!` and `#[derive(Relate)]`.
///
/// # Arguments
/// * `input` - The parse stream to read from
/// * `consume_comma` - If true, also consumes the leading comma
///   (relate_structs! style). If false, uses forked parsing to check before
///   consuming (derive style).
///
/// # Returns
/// `Some(CloneMode)` if a clone mode was found, `None` otherwise.
pub fn parse_trailing_clone_mode(
    input: ParseStream,
    consume_comma: bool,
) -> Result<Option<CloneMode>> {
    if !input.peek(Token![,]) {
        return Ok(None);
    }

    if consume_comma {
        // relate_structs! style: consume comma then check
        input.parse::<Token![,]>()?;
        parse_clone_mode_ident(input)
    } else {
        // derive style: fork to check before consuming
        let fork = input.fork();
        fork.parse::<Token![,]>()?;

        // Check for `move` keyword first
        if fork.peek(Token![move]) {
            input.parse::<Token![,]>()?;
            input.parse::<Token![move]>()?;
            return Ok(Some(CloneMode::Move));
        }

        // Check for `cloned` or `copy` identifier
        if fork.peek(Ident) {
            let ident: Ident = fork.parse()?;
            if ident == "cloned" {
                input.parse::<Token![,]>()?;
                input.parse::<Ident>()?;
                return Ok(Some(CloneMode::Cloned));
            }
            if ident == "copy" {
                input.parse::<Token![,]>()?;
                input.parse::<Ident>()?;
                return Ok(Some(CloneMode::Copy));
            }
        }

        Ok(None)
    }
}

/// Parse a clone mode identifier after the comma has been consumed.
fn parse_clone_mode_ident(input: ParseStream) -> Result<Option<CloneMode>> {
    // Check for `move` keyword
    if input.peek(Token![move]) {
        input.parse::<Token![move]>()?;
        return Ok(Some(CloneMode::Move));
    }

    if !input.peek(Ident) {
        return Err(Error::new(
            input.span(),
            "Expected clone mode after `,`.\n\
             Valid options: `cloned`, `copy`, `move`",
        ));
    }

    let ident: Ident = input.parse()?;
    match ident.to_string().as_str() {
        "cloned" => Ok(Some(CloneMode::Cloned)),
        "copy" => Ok(Some(CloneMode::Copy)),
        _ => Err(Error::new_spanned(
            ident,
            "Expected clone mode: `cloned`, `copy`, or `move`",
        )),
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse::Parser;

    use super::*;

    fn parse_tokens(input: proc_macro2::TokenStream) -> (TokenStream, bool) {
        let parser = |stream: ParseStream| parse_tokens_until_terminator(stream, true);
        parser.parse2(input).expect("failed to parse")
    }

    #[test]
    fn test_parse_tokens_simple() {
        let input = quote! { foo.bar };
        let (tokens, fallible) = parse_tokens(input);

        assert!(!fallible);
        assert!(tokens.to_string().contains("foo"));
        assert!(tokens.to_string().contains("bar"));
    }

    #[test]
    fn test_parse_tokens_trailing_question() {
        let input = quote! { foo.bar()? };
        let (tokens, fallible) = parse_tokens(input);

        assert!(fallible);
        // The `?` should be consumed, not in tokens
        assert!(!tokens.to_string().ends_with('?'));
    }

    #[test]
    fn test_parse_tokens_nested_groups() {
        let input = quote! { (foo.bar()) };
        let (tokens, fallible) = parse_tokens(input);

        assert!(!fallible);
        assert!(tokens.to_string().contains("foo"));
    }

    #[test]
    fn test_parse_tokens_stops_at_comma() {
        // Create input with comma - simulate real parsing scenario
        let input = quote! { foo.bar };
        let (tokens, fallible) = parse_tokens(input);

        assert!(!fallible);
        assert!(tokens.to_string().contains("foo"));
    }

    #[test]
    fn test_parse_tokens_stops_at_semicolon() {
        // With check_semicolon=true, should stop at semicolon
        let input = quote! { foo.bar };
        let (tokens, _) = parse_tokens(input);
        assert!(tokens.to_string().contains("foo"));
    }

    #[test]
    fn test_is_at_terminator_empty() {
        let parser = |stream: ParseStream| {
            let result = is_at_terminator(stream, true);
            Ok(result)
        };

        let result: bool = parser.parse2(quote! {}).expect("parse failed");
        assert!(result);
    }
}
