//! Shared code generation utilities.

use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::quote;

use super::types::{CloneMode, FieldMapping, Transform, transform_with_expr_tokens};

/// Controls which transforms can be reversed in bidirectional conversions.
///
/// Different macros have different policies for what can be reversed:
/// - `relate_structs!` is strict: only identity mappings are reversible
/// - `#[derive(Relate)]` is flexible: any non-default transform can be reversed
///
/// # Example
///
/// ```ignore
/// // IdentityOnly: only `name;` can be reversed, `age: with = ...` cannot
/// relate_structs! { A ~ B { name; age: with = _.to_string(); } }
///
/// // AllNonDefault: both can be reversed (user takes responsibility)
/// #[derive(Relate)]
/// #[relate(Source, both)]
/// struct Target { name: String, age: String }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReverseStrategy {
    /// Only reverse identity transforms (`relate_structs!` behavior).
    ///
    /// Transforms like `with = expr` will cause a compile error if
    /// bidirectional, since there's no automatic way to reverse arbitrary
    /// expressions.
    IdentityOnly,
    /// Reverse all non-default transforms (`#[derive(Relate)]` behavior).
    ///
    /// Allows bidirectional with any reversible transform. The user is
    /// responsible for ensuring the reverse mapping makes semantic sense.
    AllNonDefault,
}

/// Check if a token stream contains a method/function call (parentheses).
/// Used to determine if a WithExpr produces an owned value.
#[must_use]
pub fn tokens_contain_call(tokens: &TokenStream) -> bool {
    tokens.clone().into_iter().any(|tt| match tt {
        TokenTree::Group(group) => {
            group.delimiter() == proc_macro2::Delimiter::Parenthesis
                || tokens_contain_call(&group.stream())
        }
        _ => false,
    })
}

/// Generate field access code: `src.field` or `src.field.clone()`
#[must_use]
pub fn field_access(field: &Ident, should_clone: bool) -> TokenStream {
    if should_clone {
        quote! { src.#field.clone() }
    } else {
        quote! { src.#field }
    }
}

/// Generate a single field initialization expression.
#[must_use]
pub fn generate_field_init(
    mapping: &FieldMapping,
    field_index: usize,
    is_ref: bool,
    field_usage: &HashMap<String, FieldUsage>,
    struct_clone_mode: CloneMode,
) -> TokenStream {
    let target = &mapping.target_field;
    let source_field = mapping.source.get_field_name(target);

    // Determine effective clone mode (field overrides struct)
    let effective_clone_mode = mapping.source.clone_mode.unwrap_or(struct_clone_mode);

    // Determine if we need to clone based on clone mode
    let should_clone = should_clone_field(
        mapping,
        field_index,
        is_ref,
        field_usage,
        effective_clone_mode,
    );

    let value = match &mapping.source.transform {
        // Default transforms don't use a source field value
        Transform::Default => quote! { ::core::default::Default::default() },
        Transform::DefaultExpr(expr) => quote! { #expr },

        // Identity: direct field access
        Transform::Identity => field_access(source_field, should_clone),

        // `with = expr` - transform tokens using `.field` and `_` syntax
        Transform::WithExpr(tokens, fallible) => {
            let transformed = transform_with_expr_tokens(tokens, source_field);
            // For simple field paths (no method calls), we need to clone in ref impl
            // Method calls typically return owned values, so no clone needed
            let needs_clone = should_clone && !tokens_contain_call(tokens);
            let value = if needs_clone {
                quote! { (#transformed).clone() }
            } else {
                transformed
            };
            if *fallible {
                quote! { #value? }
            } else {
                value
            }
        }

        // Collection map: `with = [_.field]`
        Transform::CollectionMap(tokens) => {
            let replaced = replace_placeholder(tokens, "__item");
            // With cloned mode, use .iter().cloned().map(...).collect()
            // and always apply Into::into for type conversion
            if effective_clone_mode == CloneMode::Cloned {
                quote! {
                    src.#source_field.iter()
                        .cloned()
                        .map(|__item| ::core::convert::Into::into(#replaced))
                        .collect()
                }
            } else {
                quote! { src.#source_field.iter().map(|__item| #replaced).collect() }
            }
        }
    };

    quote! { #target: #value }
}

/// Determine if a field should be cloned based on clone mode.
///
/// `field_index` is the index of this field in the struct, used to determine
/// if this is the last use of a multi-use field (last use can move instead of
/// clone).
fn should_clone_field(
    mapping: &FieldMapping,
    field_index: usize,
    is_ref: bool,
    field_usage: &HashMap<String, FieldUsage>,
    effective_clone_mode: CloneMode,
) -> bool {
    // Never clone if we don't read a field
    if !mapping.source.reads_field() {
        return false;
    }

    // Copy mode: user asserts type is Copy, never clone
    if effective_clone_mode == CloneMode::Copy {
        return false;
    }

    // Cloned mode: always clone
    if effective_clone_mode == CloneMode::Cloned {
        return true;
    }

    // Ref impl: must clone (can't move out of reference)
    if is_ref {
        return true;
    }

    // --- Below here: owned impl only ---

    // Move mode: never clone in owned impl
    if effective_clone_mode == CloneMode::Move {
        return false;
    }

    // Auto mode: clone only multi-use fields (except last use of Identity)
    let usage_key = mapping.source.get_usage_key(&mapping.target_field);
    let Some(usage) = field_usage.get(&usage_key) else {
        return false;
    };

    if usage.count <= 1 {
        return false;
    }

    // Multi-use: Identity can move on last use, others always clone
    !matches!(mapping.source.transform, Transform::Identity if field_index == usage.last_index)
}

/// Field usage information for smart cloning.
#[derive(Debug, Clone)]
pub struct FieldUsage {
    /// Total number of times this source expression is used
    pub count:      usize,
    /// The last field index where this source expression is used
    pub last_index: usize,
}

/// Count how many times each source expression is used and track the last usage
/// index. Uses `get_usage_key` to properly track ChainedAccess paths.
#[must_use]
pub fn count_field_usage(mappings: &[FieldMapping]) -> HashMap<String, FieldUsage> {
    let mut usage: HashMap<String, FieldUsage> = HashMap::new();

    for (index, mapping) in mappings.iter().enumerate() {
        if !mapping.source.reads_field() {
            continue;
        }
        let usage_key = mapping.source.get_usage_key(&mapping.target_field);
        usage
            .entry(usage_key)
            .and_modify(|u| {
                u.count += 1;
                u.last_index = index;
            })
            .or_insert(FieldUsage {
                count:      1,
                last_index: index,
            });
    }

    usage
}

/// Count field usage for reverse direction (target fields become sources).
#[must_use]
pub fn count_reverse_field_usage(mappings: &[FieldMapping]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for mapping in mappings {
        if mapping.source.transform.is_default_kind() {
            continue;
        }
        *counts.entry(mapping.target_field.to_string()).or_insert(0) += 1;
    }

    counts
}

/// Generate a reverse field initialization for bidirectional relations.
/// Returns None for fields that can't be reversed (defaults, transforms, etc.).
#[must_use]
pub fn generate_reverse_field_init(
    mapping: &FieldMapping,
    is_ref: bool,
    field_usage: &HashMap<String, usize>,
    strategy: ReverseStrategy,
) -> Option<TokenStream> {
    let target = &mapping.target_field;

    // Skip fields that don't have simple reverse mappings
    if mapping.source.transform.is_default_kind() {
        return None;
    }

    // For relate_structs!, only reverse identity transforms
    if strategy == ReverseStrategy::IdentityOnly
        && !matches!(mapping.source.transform, Transform::Identity)
    {
        return None;
    }

    let should_clone = is_ref || field_usage.get(&target.to_string()).copied().unwrap_or(0) > 1;

    let value = if should_clone {
        quote! { src.#target.clone() }
    } else {
        quote! { src.#target }
    };

    // Get the source field name (in reverse, it becomes the destination)
    let source_field = mapping.source.get_field_name(target);

    Some(quote! { #source_field: #value })
}

/// Replace `_` with the given replacement in token stream.
/// Uses `call_site` span because the replacement identifier must be visible
/// in the generated closure (e.g., `|__item| __item.field`).
#[must_use]
pub fn replace_placeholder(tokens: &TokenStream, replacement: &str) -> TokenStream {
    // Use call_site because the identifier is used in generated closures
    // and must be accessible in the expanded code
    let replacement_ident = Ident::new(replacement, proc_macro2::Span::call_site());
    tokens
        .clone()
        .into_iter()
        .map(|tt| match tt {
            TokenTree::Ident(ident) if ident == "_" => TokenTree::Ident(replacement_ident.clone()),
            TokenTree::Group(group) => {
                let replaced = replace_placeholder(&group.stream(), replacement);
                TokenTree::Group(proc_macro2::Group::new(group.delimiter(), replaced))
            }
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::FieldSource;

    fn make_identity_mapping(name: &str) -> FieldMapping {
        FieldMapping {
            target_field: Ident::new(name, proc_macro2::Span::call_site()),
            source:       FieldSource::auto(),
        }
    }

    fn make_default_mapping(name: &str) -> FieldMapping {
        FieldMapping {
            target_field: Ident::new(name, proc_macro2::Span::call_site()),
            source:       FieldSource::default_value(),
        }
    }

    #[test]
    fn test_count_field_usage_single() {
        let mappings = vec![
            make_identity_mapping("a"),
            make_identity_mapping("b"),
            make_identity_mapping("c"),
        ];

        let usage = count_field_usage(&mappings);

        assert_eq!(usage.len(), 3);
        assert_eq!(usage.get("a").map(|u| u.count), Some(1));
        assert_eq!(usage.get("b").map(|u| u.count), Some(1));
        assert_eq!(usage.get("c").map(|u| u.count), Some(1));
    }

    #[test]
    fn test_count_field_usage_skips_defaults() {
        let mappings = vec![
            make_identity_mapping("a"),
            make_default_mapping("b"),
            make_identity_mapping("c"),
        ];

        let usage = count_field_usage(&mappings);

        // Default fields are not counted
        assert_eq!(usage.len(), 2);
        assert!(usage.contains_key("a"));
        assert!(!usage.contains_key("b"));
        assert!(usage.contains_key("c"));
    }

    #[test]
    fn test_count_field_usage_tracks_last_index() {
        let mappings = vec![
            make_identity_mapping("a"),
            make_identity_mapping("b"),
            make_identity_mapping("c"),
        ];

        let usage = count_field_usage(&mappings);

        assert_eq!(usage.get("a").map(|u| u.last_index), Some(0));
        assert_eq!(usage.get("b").map(|u| u.last_index), Some(1));
        assert_eq!(usage.get("c").map(|u| u.last_index), Some(2));
    }

    #[test]
    fn test_tokens_contain_call_with_parens() {
        let tokens: TokenStream = quote! { foo.bar() };
        assert!(tokens_contain_call(&tokens));
    }

    #[test]
    fn test_tokens_contain_call_without_parens() {
        let tokens: TokenStream = quote! { foo.bar };
        assert!(!tokens_contain_call(&tokens));
    }

    #[test]
    fn test_tokens_contain_call_nested() {
        let tokens: TokenStream = quote! { (foo.bar()) };
        assert!(tokens_contain_call(&tokens));
    }

    #[test]
    fn test_replace_placeholder() {
        let tokens: TokenStream = quote! { _.field };
        let replaced = replace_placeholder(&tokens, "__item");
        let replaced_str = replaced.to_string();
        assert!(replaced_str.contains("__item"));
        assert!(!replaced_str.contains('_') || replaced_str.contains("__item"));
    }

    #[test]
    fn test_replace_placeholder_nested() {
        let tokens: TokenStream = quote! { (_.field.method()) };
        let replaced = replace_placeholder(&tokens, "__item");
        let replaced_str = replaced.to_string();
        assert!(replaced_str.contains("__item"));
    }

    #[test]
    fn test_field_access_without_clone() {
        let field = Ident::new("name", proc_macro2::Span::call_site());
        let tokens = field_access(&field, false);
        let token_str = tokens.to_string();
        assert!(token_str.contains("src . name"));
        assert!(!token_str.contains("clone"));
    }

    #[test]
    fn test_field_access_with_clone() {
        let field = Ident::new("name", proc_macro2::Span::call_site());
        let tokens = field_access(&field, true);
        let token_str = tokens.to_string();
        assert!(token_str.contains("clone"));
    }

    #[test]
    fn test_count_reverse_field_usage() {
        let mappings = vec![
            make_identity_mapping("a"),
            make_identity_mapping("b"),
            make_default_mapping("c"),
        ];

        let usage = count_reverse_field_usage(&mappings);

        // a and b count, c is default so skipped
        assert_eq!(usage.len(), 2);
        assert_eq!(usage.get("a").copied(), Some(1));
        assert_eq!(usage.get("b").copied(), Some(1));
        assert!(!usage.contains_key("c"));
    }
}
