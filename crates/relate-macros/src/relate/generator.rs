//! Code generator for the `relate_structs!` macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Result};

use super::types::*;
use crate::core::{
    CloneMode, ReverseStrategy, count_field_usage, count_reverse_field_usage, generate_field_init,
    generate_reverse_field_init,
};

/// Check if any field mapping has a fallible transform.
fn has_fallible_fields(fields: &[FieldMapping]) -> bool {
    fields.iter().any(|f| f.source.transform.is_fallible())
}

/// Get the effective direction, auto-upgrading to TryForward if fallible
/// transforms detected. For Bidirectional, only upgrades the forward direction;
/// backward stays as From.
fn effective_direction(direction: &Direction, fields: &[FieldMapping]) -> Direction {
    if has_fallible_fields(fields) {
        match direction {
            Direction::Forward => Direction::TryForward(None),
            Direction::TryForward(e) => Direction::TryForward(e.clone()),
            Direction::Bidirectional => Direction::TryForward(None), // Forward becomes TryFrom
        }
    } else {
        direction.clone()
    }
}

/// Generate a pair of From implementations (owned and reference).
///
/// Generates:
/// - `impl From<source_type> for target_type`
/// - `impl From<&source_type> for target_type`
fn generate_from_impl_pair(
    source_type: &TokenStream,
    target_type: &TokenStream,
    impl_generics: &TokenStream,
    where_clause: &TokenStream,
    owned_fields: &[TokenStream],
    ref_fields: &[TokenStream],
) -> TokenStream {
    quote! {
        impl #impl_generics ::core::convert::From<#source_type> for #target_type #where_clause {
            fn from(src: #source_type) -> Self {
                Self {
                    #(#owned_fields),*
                }
            }
        }

        impl #impl_generics ::core::convert::From<&#source_type> for #target_type #where_clause {
            fn from(src: &#source_type) -> Self {
                Self {
                    #(#ref_fields),*
                }
            }
        }
    }
}

/// Generate a pair of TryFrom implementations (owned and reference).
fn generate_try_from_impl_pair(
    source_type: &TokenStream,
    target_type: &TokenStream,
    impl_generics: &TokenStream,
    where_clause: &TokenStream,
    error_type: &TokenStream,
    owned_fields: &[TokenStream],
    ref_fields: &[TokenStream],
) -> TokenStream {
    quote! {
        impl #impl_generics ::core::convert::TryFrom<#source_type> for #target_type #where_clause {
            type Error = #error_type;

            fn try_from(src: #source_type) -> ::core::result::Result<Self, Self::Error> {
                ::core::result::Result::Ok(Self {
                    #(#owned_fields),*
                })
            }
        }

        impl #impl_generics ::core::convert::TryFrom<&#source_type> for #target_type #where_clause {
            type Error = #error_type;

            fn try_from(src: &#source_type) -> ::core::result::Result<Self, Self::Error> {
                ::core::result::Result::Ok(Self {
                    #(#ref_fields),*
                })
            }
        }
    }
}

/// Main entry point for generating output from parsed input.
pub fn generate_relate_output(input: &RelateInput) -> Result<TokenStream> {
    let mut output = TokenStream::new();

    for relation in &input.relations {
        output.extend(generate_relation(relation)?);
    }

    Ok(output)
}

fn generate_relation(relation: &Relation) -> Result<TokenStream> {
    generate_existing_relation(&relation.0)
}

fn generate_existing_relation(relation: &ExistingRelation) -> Result<TokenStream> {
    let source_name = &relation.source.name;
    let target_name = &relation.target.name;

    let source_generics = relation.source.generics.as_ref();
    let target_generics = relation.target.generics.as_ref();

    // For type position, we only need the type parameters (no bounds)
    // e.g., Container<T> not Container<T: Clone>
    let source_type = source_generics
        .map(|g| {
            let (_, ty_generics, _) = g.split_for_impl();
            quote! { #source_name #ty_generics }
        })
        .unwrap_or_else(|| quote! { #source_name });

    let target_type = target_generics
        .map(|g| {
            let (_, ty_generics, _) = g.split_for_impl();
            quote! { #target_name #ty_generics }
        })
        .unwrap_or_else(|| quote! { #target_name });

    // Get generics for impl (prefer source, fall back to target)
    // This includes the bounds: impl<T: Clone>
    let (impl_generics, where_clause) = source_generics
        .or(target_generics)
        .map(|g| {
            let (impl_gen, _, where_cl) = g.split_for_impl();
            (quote! { #impl_gen }, quote! { #where_cl })
        })
        .unwrap_or_else(|| (quote! {}, quote! {}));

    let Some(body) = &relation.body else {
        return Err(Error::new_spanned(
            source_name,
            "Cannot use `A :: B` without fields - proc macros cannot introspect struct fields.\n\
             Use `A :: B { field1, field2, ... }` to list fields explicitly,\n\
             or use `#[attrs] struct A :: #[attrs] struct B { fields }` to define both structs.",
        ));
    };

    if body.has_spread && body.fields.is_empty() {
        return Err(Error::new_spanned(
            source_name,
            "Cannot use `..` spread alone with existing structs - proc macros cannot introspect fields.\n\
             List the fields explicitly: `A -> B { field1, field2 }`",
        ));
    }

    // Use the core utility for counting field usage
    let field_usage = count_field_usage(&body.fields);

    // Generate field initializers using core utility
    // relate_structs! macro uses Auto clone mode (default behavior)
    let forward_fields: Vec<_> = body
        .fields
        .iter()
        .enumerate()
        .map(|(idx, f)| generate_field_init(f, idx, false, &field_usage, CloneMode::Auto))
        .collect();

    let forward_ref_fields: Vec<_> = body
        .fields
        .iter()
        .enumerate()
        .map(|(idx, f)| generate_field_init(f, idx, true, &field_usage, CloneMode::Auto))
        .collect();

    let mut output = TokenStream::new();

    // Auto-detect fallible transforms and upgrade direction if needed
    let effective_dir = effective_direction(&relation.direction, &body.fields);

    // Generate forward impls based on effective direction
    match &effective_dir {
        Direction::TryForward(custom_error) => {
            let error_type = custom_error
                .as_ref()
                .map(|t| quote! { #t })
                .unwrap_or_else(|| quote! { ::relate::ConversionError });

            output.extend(generate_try_from_impl_pair(
                &source_type,
                &target_type,
                &impl_generics,
                &where_clause,
                &error_type,
                &forward_fields,
                &forward_ref_fields,
            ));
        }
        Direction::Forward | Direction::Bidirectional => {
            output.extend(generate_from_impl_pair(
                &source_type,
                &target_type,
                &impl_generics,
                &where_clause,
                &forward_fields,
                &forward_ref_fields,
            ));
        }
    }

    // Generate backward impls if bidirectional: From<Target> for Source
    if relation.direction == Direction::Bidirectional {
        let reverse_usage = count_reverse_field_usage(&body.fields);

        // For relate_structs!, only Identity transforms can be reversed
        let backward_fields: Vec<_> = body
            .fields
            .iter()
            .filter_map(|f| {
                generate_reverse_field_init(f, false, &reverse_usage, ReverseStrategy::IdentityOnly)
            })
            .collect();

        let backward_ref_fields: Vec<_> = body
            .fields
            .iter()
            .filter_map(|f| {
                generate_reverse_field_init(f, true, &reverse_usage, ReverseStrategy::IdentityOnly)
            })
            .collect();

        output.extend(generate_from_impl_pair(
            &target_type,
            &source_type,
            &impl_generics,
            &where_clause,
            &backward_fields,
            &backward_ref_fields,
        ));
    }

    Ok(output)
}
