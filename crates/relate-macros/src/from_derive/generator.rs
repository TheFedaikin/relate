//! Code generator for the Relate derive macro.

use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Expr, Ident};

use super::types::{CloneMode, ConversionMode, FromDeriveInput};
use crate::core::{
    FieldMapping, FieldUsage, ReverseStrategy, Transform, count_field_usage,
    count_reverse_field_usage, generate_field_init, generate_reverse_field_init,
    tokens_contain_call,
};

/// Tracks default expressions that should be hoisted to let bindings.
///
/// When the same default expression (like `Utc::now()`) is used for multiple
/// fields, we hoist it to a `let` binding to avoid calling it multiple times.
struct DefaultBindings {
    /// Map from expression string -> (binding name, total usage count)
    bindings: HashMap<String, (Ident, usize)>,
}

impl DefaultBindings {
    fn new(fields: &[FieldMapping]) -> Self {
        let mut expr_counts: HashMap<String, usize> = HashMap::new();

        // Count how many times each default expression is used
        // Only count expressions that are safe to hoist (function/method calls)
        for field in fields {
            let Transform::DefaultExpr(expr) = &field.source.transform else {
                continue;
            };
            if !Self::is_hoistable(expr) {
                continue;
            }
            let key = expr.to_token_stream().to_string();
            *expr_counts.entry(key).or_insert(0) += 1;
        }

        // Create bindings only for expressions used more than once
        let mut bindings = HashMap::new();
        let mut binding_idx: usize = 0;
        for (expr_str, count) in expr_counts {
            if count <= 1 {
                continue;
            }
            // Use mixed_site for hygiene - prevents user code from accessing internal
            // identifiers
            let binding_name =
                Ident::new(&format!("__default_{}", binding_idx), Span::mixed_site());
            bindings.insert(expr_str, (binding_name, count));
            binding_idx += 1;
        }

        Self { bindings }
    }

    /// Check if an expression is safe to hoist to a let binding.
    ///
    /// Function and method calls are hoistable because they return concrete
    /// types. Type-polymorphic expressions like `None`, `true`, `false`,
    /// literals are NOT safe because they could have different types for
    /// different fields.
    const fn is_hoistable(expr: &Expr) -> bool {
        matches!(expr, Expr::Call(_) | Expr::MethodCall(_))
    }

    /// Get the binding name and usage count for an expression, if it should be
    /// hoisted.
    fn get_binding_with_count(&self, expr: &Expr) -> Option<(&Ident, usize)> {
        let key = expr.to_token_stream().to_string();
        self.bindings.get(&key).map(|(name, count)| (name, *count))
    }

    /// Generate let bindings for all hoisted expressions.
    fn generate_let_bindings(&self, fields: &[FieldMapping]) -> Vec<TokenStream> {
        let mut seen = HashSet::new();
        let mut bindings = Vec::new();

        // Iterate in field order to emit bindings in a predictable order
        for field in fields {
            let Transform::DefaultExpr(expr) = &field.source.transform else {
                continue;
            };
            let key = expr.to_token_stream().to_string();
            let Some((binding_name, _)) = self.bindings.get(&key) else {
                continue;
            };
            if !seen.insert(key) {
                continue; // Already emitted this binding
            }
            bindings.push(quote! { let #binding_name = #expr; });
        }

        bindings
    }
}

/// Tracks WithExpr fields that should be evaluated before the struct init.
///
/// WithExpr expressions access `src` directly, so they must be evaluated before
/// any fields are moved. By hoisting them to let bindings, we allow other
/// fields to move from `src` without causing partial move errors.
struct WithExprBindings {
    /// Map from target field name -> binding name
    bindings: HashMap<String, Ident>,
}

impl WithExprBindings {
    fn new(fields: &[FieldMapping]) -> Self {
        let mut bindings = HashMap::new();

        for field in fields {
            if let Transform::WithExpr(_, _) = &field.source.transform {
                let field_name = field.target_field.to_string();
                // Use mixed_site for hygiene
                let binding_name =
                    Ident::new(&format!("__with_{}", field_name), Span::mixed_site());
                bindings.insert(field_name, binding_name);
            }
        }

        Self { bindings }
    }

    /// Get the binding name for a field, if it's a WithExpr field.
    fn get_binding(&self, field_name: &str) -> Option<&Ident> { self.bindings.get(field_name) }

    /// Generate let bindings for all WithExpr fields.
    /// These must be evaluated BEFORE any fields are moved from src.
    fn generate_let_bindings(
        &self,
        fields: &[FieldMapping],
        is_ref: bool,
        field_usage: &HashMap<String, FieldUsage>,
    ) -> Vec<TokenStream> {
        use crate::core::transform_with_expr_tokens;

        let mut bindings = Vec::new();

        // Iterate in field order to emit bindings in a predictable order
        for field in fields {
            let Transform::WithExpr(tokens, fallible) = &field.source.transform else {
                continue;
            };
            let field_name = field.target_field.to_string();
            let Some(binding_name) = self.bindings.get(&field_name) else {
                continue;
            };

            let transformed = transform_with_expr_tokens(tokens, &field.target_field);

            // Need to clone if:
            // 1. ref impl with simple field access (no method calls), OR
            // 2. owned impl where the source field is used multiple times
            let is_simple_field = !tokens_contain_call(tokens);
            let usage_key = field.source.get_usage_key(&field.target_field);
            let is_multi_use = field_usage.get(&usage_key).is_some_and(|u| u.count > 1);
            let needs_clone = is_simple_field && (is_ref || is_multi_use);

            let value = if needs_clone {
                quote! { (#transformed).clone() }
            } else {
                transformed
            };
            let value = if *fallible {
                quote! { #value? }
            } else {
                value
            };
            bindings.push(quote! { let #binding_name = #value; });
        }

        bindings
    }
}

/// Helper for generating field initializers with all the hoisting logic.
struct FieldGenerator<'a> {
    fields:             &'a [FieldMapping],
    clone_mode:         CloneMode,
    field_usage:        HashMap<String, FieldUsage>,
    default_bindings:   DefaultBindings,
    with_expr_bindings: WithExprBindings,
}

impl<'a> FieldGenerator<'a> {
    fn new(fields: &'a [FieldMapping], clone_mode: CloneMode) -> Self {
        Self {
            fields,
            clone_mode,
            field_usage: count_field_usage(fields),
            default_bindings: DefaultBindings::new(fields),
            with_expr_bindings: WithExprBindings::new(fields),
        }
    }

    /// Generate let bindings (WithExpr first, then defaults).
    fn let_bindings(&self, is_ref: bool) -> Vec<TokenStream> {
        let mut bindings =
            self.with_expr_bindings
                .generate_let_bindings(self.fields, is_ref, &self.field_usage);
        bindings.extend(self.default_bindings.generate_let_bindings(self.fields));
        bindings
    }

    /// Generate field initializers for owned or ref conversion.
    fn field_inits(&self, is_ref: bool) -> Vec<TokenStream> {
        self.fields
            .iter()
            .enumerate()
            .map(|(idx, f)| self.field_init(f, idx, is_ref))
            .collect()
    }

    /// Generate a single field init, using hoisted bindings for repeated
    /// default expressions.
    fn field_init(&self, mapping: &FieldMapping, field_index: usize, is_ref: bool) -> TokenStream {
        let target = &mapping.target_field;

        // WithExpr fields might be hoisted - check for binding
        if let Some(binding) = self.with_expr_bindings.get_binding(&target.to_string()) {
            return quote! { #target: #binding };
        }

        // Hoisted default expressions - check if we need to clone the binding
        let Transform::DefaultExpr(expr) = &mapping.source.transform else {
            return generate_field_init(
                mapping,
                field_index,
                is_ref,
                &self.field_usage,
                self.clone_mode,
            );
        };

        let Some((binding, count)) = self.default_bindings.get_binding_with_count(expr) else {
            return generate_field_init(
                mapping,
                field_index,
                is_ref,
                &self.field_usage,
                self.clone_mode,
            );
        };

        let mode = mapping.source.clone_mode.unwrap_or(self.clone_mode);
        let needs_clone = match mode {
            CloneMode::Move | CloneMode::Copy => false,
            CloneMode::Cloned => true,
            CloneMode::Auto => is_ref || count > 1,
        };

        if needs_clone {
            quote! { #target: #binding.clone() }
        } else {
            quote! { #target: #binding }
        }
    }
}

/// Generate the From or TryFrom implementations based on conversion mode.
#[must_use]
pub fn generate_from_derive(input: &FromDeriveInput) -> TokenStream {
    match &input.conversion_mode {
        ConversionMode::Infallible => generate_from_impl(input),
        ConversionMode::Fallible(error_type) => generate_try_from_impl(input, error_type),
    }
}

/// Generate From implementations (infallible conversion).
fn generate_from_impl(input: &FromDeriveInput) -> TokenStream {
    let mut output = TokenStream::new();

    let target_name = &input.target_name;
    let source_type = &input.source_type;
    let (impl_generics, ty_generics, where_clause) = input.target_generics.split_for_impl();

    let field_gen = FieldGenerator::new(&input.fields, input.clone_mode);
    let owned_let_bindings = field_gen.let_bindings(false);
    let ref_let_bindings = field_gen.let_bindings(true);
    let owned_fields = field_gen.field_inits(false);
    let ref_fields = field_gen.field_inits(true);

    output.extend(quote! {
        impl #impl_generics ::core::convert::From<#source_type> for #target_name #ty_generics #where_clause {
            fn from(src: #source_type) -> Self {
                #(#owned_let_bindings)*
                Self { #(#owned_fields),* }
            }
        }

        impl #impl_generics ::core::convert::From<&#source_type> for #target_name #ty_generics #where_clause {
            fn from(src: &#source_type) -> Self {
                #(#ref_let_bindings)*
                Self { #(#ref_fields),* }
            }
        }
    });

    // Generate reverse impls if bidirectional
    if input.bidirectional {
        let reverse_usage = count_reverse_field_usage(&input.fields);

        let reverse_owned: Vec<_> = input
            .fields
            .iter()
            .filter_map(|f| {
                generate_reverse_field_init(
                    f,
                    false,
                    &reverse_usage,
                    ReverseStrategy::AllNonDefault,
                )
            })
            .collect();

        let reverse_ref: Vec<_> = input
            .fields
            .iter()
            .filter_map(|f| {
                generate_reverse_field_init(f, true, &reverse_usage, ReverseStrategy::AllNonDefault)
            })
            .collect();

        output.extend(quote! {
            impl #impl_generics ::core::convert::From<#target_name #ty_generics> for #source_type #where_clause {
                fn from(src: #target_name #ty_generics) -> Self {
                    Self { #(#reverse_owned),* }
                }
            }

            impl #impl_generics ::core::convert::From<&#target_name #ty_generics> for #source_type #where_clause {
                fn from(src: &#target_name #ty_generics) -> Self {
                    Self { #(#reverse_ref),* }
                }
            }
        });
    }

    output
}

/// Generate TryFrom implementations (fallible conversion).
fn generate_try_from_impl(input: &FromDeriveInput, error_type: &Option<syn::Type>) -> TokenStream {
    let mut output = TokenStream::new();

    let target_name = &input.target_name;
    let source_type = &input.source_type;
    let (impl_generics, ty_generics, where_clause) = input.target_generics.split_for_impl();

    let error = error_type
        .as_ref()
        .map(|t| quote! { #t })
        .unwrap_or_else(|| quote! { ::relate::ConversionError });

    let field_gen = FieldGenerator::new(&input.fields, input.clone_mode);
    let owned_let_bindings = field_gen.let_bindings(false);
    let ref_let_bindings = field_gen.let_bindings(true);
    let owned_fields = field_gen.field_inits(false);
    let ref_fields = field_gen.field_inits(true);

    output.extend(quote! {
        impl #impl_generics ::core::convert::TryFrom<#source_type> for #target_name #ty_generics #where_clause {
            type Error = #error;

            fn try_from(src: #source_type) -> ::core::result::Result<Self, Self::Error> {
                #(#owned_let_bindings)*
                ::core::result::Result::Ok(Self { #(#owned_fields),* })
            }
        }

        impl #impl_generics ::core::convert::TryFrom<&#source_type> for #target_name #ty_generics #where_clause {
            type Error = #error;

            fn try_from(src: &#source_type) -> ::core::result::Result<Self, Self::Error> {
                #(#ref_let_bindings)*
                ::core::result::Result::Ok(Self { #(#ref_fields),* })
            }
        }
    });

    output
}
