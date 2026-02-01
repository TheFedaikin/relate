//! Unified types for field mapping across all macros.

use proc_macro2::TokenStream;
use syn::Expr;
pub use syn::Ident;

/// Clone mode for field access.
///
/// Controls when fields are cloned during conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CloneMode {
    /// Automatic: clone when needed (reference source or multiple usage)
    #[default]
    Auto,
    /// Always clone field accesses
    Cloned,
    /// Never implicitly clone (move/take ownership)
    Move,
    /// Field is Copy - no clone needed even for ref impl
    /// User is responsible for ensuring the type actually implements Copy
    Copy,
}

/// Describes how to map a single field from source to target.
#[derive(Debug, Clone)]
pub struct FieldMapping {
    /// The target field name
    pub target_field: Ident,
    /// Where and how to get the value
    pub source:       FieldSource,
}

/// Where a field's value comes from and how to transform it.
#[derive(Debug, Clone)]
pub struct FieldSource {
    /// The source field name (None = same as target for auto-mapping)
    pub field_name: Option<Ident>,
    /// How to transform the value
    pub transform:  Transform,
    /// Field-level clone mode override (None = use struct default)
    pub clone_mode: Option<CloneMode>,
}

impl FieldSource {
    /// Create an auto-mapping source (same field name, no transform)
    #[must_use]
    pub const fn auto() -> Self {
        Self {
            field_name: None,
            transform:  Transform::Identity,
            clone_mode: None,
        }
    }

    /// Create a source with a transform (same field name)
    #[must_use]
    pub const fn with_transform(transform: Transform) -> Self {
        Self {
            field_name: None,
            transform,
            clone_mode: None,
        }
    }

    /// Create a default value source
    #[must_use]
    pub const fn default_value() -> Self {
        Self {
            field_name: None,
            transform:  Transform::Default,
            clone_mode: None,
        }
    }

    /// Create a default value source with a specific expression
    #[must_use]
    pub const fn default_expr(expr: Expr) -> Self {
        Self {
            field_name: None,
            transform:  Transform::DefaultExpr(expr),
            clone_mode: None,
        }
    }

    /// Create a source using `with = expr` syntax
    #[must_use]
    pub fn with_expr(tokens: TokenStream, fallible: bool) -> Self {
        Self {
            field_name: None,
            transform:  Transform::WithExpr(tokens, fallible),
            clone_mode: None,
        }
    }

    /// Get the effective source field name (falls back to target field if None)
    #[must_use]
    pub fn get_field_name<'a>(&'a self, target: &'a Ident) -> &'a Ident {
        self.field_name.as_ref().unwrap_or(target)
    }

    /// Check if this source reads from a field (not a
    /// `default`/`skip`/`from_expr`)
    #[must_use]
    pub const fn reads_field(&self) -> bool { !self.transform.is_default_kind() }

    /// Get a usage key for tracking field usage.
    /// For WithExpr, this is the normalized token stream (with `_` replaced).
    /// For other transforms, this is the source field name.
    #[must_use]
    pub fn get_usage_key(&self, target: &Ident) -> String {
        match &self.transform {
            Transform::WithExpr(tokens, _) => {
                // Normalize the token stream by replacing `_` with field name
                let normalized = replace_underscore_in_tokens(tokens, target);
                // Normalize the string: remove whitespace and leading dots
                // so `.name` and `name` produce the same key
                let key = normalized.to_string().replace(' ', "");
                key.trim_start_matches('.').to_string()
            }
            Transform::CollectionMap(tokens) => {
                // Collection maps also use a path-based key
                tokens.to_string()
            }
            _ => {
                // For other transforms, use the source field name
                self.get_field_name(target).to_string()
            }
        }
    }
}

/// Replace `_` with the field name in a token stream, handling `.` context.
///
/// This function is used to normalize underscore placeholders in expressions:
/// - When `_` is preceded by `.`, it inserts just the field name
/// - When `_` is standalone, it inserts `.field` to normalize the form
///
/// This ensures `_.foo` and `._.foo` produce the same normalized form,
/// which is important for field usage tracking (identifying when the same
/// source expression is used multiple times).
///
/// # Examples (conceptual)
///
/// - `_.to_string()` with field `name` → `.name.to_string()`
/// - `._.len()` with field `value` → `.value.len()`
/// - `(_.clone())` with field `data` → `(.data.clone())`
///
/// # Arguments
///
/// * `tokens` - The token stream containing `_` placeholders
/// * `field` - The field name to replace `_` with
pub fn replace_underscore_in_tokens(tokens: &TokenStream, field: &Ident) -> TokenStream {
    use proc_macro2::TokenTree;
    use quote::quote;

    let tokens_vec: Vec<_> = tokens.clone().into_iter().collect();
    let mut result = Vec::new();

    for (i, tt) in tokens_vec.iter().enumerate() {
        match tt {
            TokenTree::Ident(ident) if ident == "_" => {
                let preceded_by_dot = i > 0
                    && matches!(&tokens_vec[i - 1], TokenTree::Punct(p) if p.as_char() == '.');

                if preceded_by_dot {
                    result.push(TokenTree::Ident(field.clone()));
                } else {
                    // Insert `.field` to normalize
                    result.extend(quote! { .#field }.into_iter());
                }
            }
            TokenTree::Group(group) => {
                let replaced = replace_underscore_in_tokens(&group.stream(), field);
                result.push(TokenTree::Group(proc_macro2::Group::new(
                    group.delimiter(),
                    replaced,
                )));
            }
            other => {
                result.push(other.clone());
            }
        }
    }

    result.into_iter().collect()
}

/// Check if an identifier is a Rust keyword that starts an expression context.
/// These keywords are followed by expressions, so `.field` after them is
/// source-access.
fn is_keyword(ident: &proc_macro2::Ident) -> bool {
    let s = ident.to_string();
    matches!(
        s.as_str(),
        "if" | "else"
            | "match"
            | "while"
            | "for"
            | "loop"
            | "return"
            | "break"
            | "continue"
            | "let"
            | "const"
            | "static"
            | "fn"
            | "pub"
            | "mod"
            | "use"
            | "struct"
            | "enum"
            | "impl"
            | "trait"
            | "type"
            | "where"
            | "async"
            | "await"
            | "move"
            | "ref"
            | "mut"
            | "as"
            | "in"
            | "unsafe"
            | "extern"
            | "crate"
            | "self"
            | "super"
            | "dyn"
            | "true"
            | "false"
    )
}

/// Check if a token at the given index is preceded by a "base" expression.
/// A `.` is a source-access dot if it's NOT preceded by:
/// - A non-keyword identifier (like `foo.bar`)
/// - A closing bracket: `)`, `]`, `}` (result of call/index/block)
/// - A `?` (like `foo?.bar`)
fn is_preceded_by_base(tokens: &[proc_macro2::TokenTree], idx: usize) -> bool {
    use proc_macro2::TokenTree;

    if idx == 0 {
        return false;
    }
    match &tokens[idx - 1] {
        TokenTree::Ident(ident) => !is_keyword(ident), // Keywords aren't bases
        TokenTree::Group(_) => true,                   // Groups end with implicit closing bracket
        TokenTree::Punct(p) => matches!(p.as_char(), ')' | ']' | '}' | '?'),
        TokenTree::Literal(_) => false, // Literals like 2.5 are single tokens
    }
}

/// Transform `with = expr` tokens:
/// - Replace `_` with `src.<field>`
/// - Insert `src` before source-access `.ident` patterns
///
/// A `.ident` is source-access if not preceded by an identifier, group, or `?`.
pub fn transform_with_expr_tokens(tokens: &TokenStream, field: &Ident) -> TokenStream {
    use proc_macro2::TokenTree;
    use quote::quote;

    let tokens_vec: Vec<_> = tokens.clone().into_iter().collect();
    let mut result = Vec::new();

    for (i, tt) in tokens_vec.iter().enumerate() {
        match tt {
            // Handle underscore → src.field
            TokenTree::Ident(ident) if ident == "_" => {
                let preceded_by_dot = i > 0
                    && matches!(&tokens_vec[i - 1], TokenTree::Punct(p) if p.as_char() == '.');

                if preceded_by_dot {
                    // `._` → just insert field name (src was already added before the dot)
                    result.push(TokenTree::Ident(field.clone()));
                } else {
                    // Standalone `_` → `src.field`
                    result.extend(quote! { src.#field }.into_iter());
                }
            }
            // Handle source-access .ident → src.ident
            TokenTree::Punct(p) if p.as_char() == '.' => {
                let next_is_ident = matches!(tokens_vec.get(i + 1), Some(TokenTree::Ident(_)));
                let is_source_access = next_is_ident && !is_preceded_by_base(&tokens_vec, i);

                if is_source_access {
                    result.extend(quote! { src }.into_iter());
                }
                result.push(tt.clone());
            }
            // Recurse into groups (parentheses, brackets, braces)
            TokenTree::Group(group) => {
                let replaced = transform_with_expr_tokens(&group.stream(), field);
                result.push(TokenTree::Group(proc_macro2::Group::new(
                    group.delimiter(),
                    replaced,
                )));
            }
            other => result.push(other.clone()),
        }
    }

    result.into_iter().collect()
}

/// How to transform a field value.
#[derive(Debug, Clone)]
pub enum Transform {
    /// Direct copy/clone, no transformation
    /// Syntax: `field;` or `field: cloned;`
    Identity,

    /// Expression using `.field` and `_` syntax.
    ///
    /// - `_` becomes the same-named source field value
    /// - `.field` becomes `src.field`
    ///
    /// Bool indicates fallibility (trailing `?`).
    /// Syntax: `field: with = expr;`
    WithExpr(TokenStream, bool),

    /// Map over a collection: `[_.id.clone()]`
    /// Syntax: `field: with = [_.id];`
    CollectionMap(TokenStream),

    /// Use `Default::default()`
    /// Syntax: `field: default;`
    Default,

    /// Use a specific default expression
    /// Syntax: `field: default = expr;`
    DefaultExpr(Expr),
}

impl Transform {
    /// Check if this is an identity transform (direct copy/move).
    ///
    /// Identity transforms are the simplest: they just copy/move the field
    /// value without any modification.
    #[must_use]
    #[allow(dead_code)]
    pub const fn is_identity(&self) -> bool { matches!(self, Self::Identity) }

    /// Check if this transform is a "default" type (doesn't read from source
    /// field).
    ///
    /// Default transforms use `Default::default()` or a custom expression,
    /// and don't read from any source field.
    #[must_use]
    pub const fn is_default_kind(&self) -> bool {
        matches!(self, Self::Default | Self::DefaultExpr(_))
    }

    /// Check if this transform contains fallible expressions (with `?`).
    ///
    /// A transform is fallible if it may fail at runtime, indicated by
    /// a trailing `?` operator in the expression.
    #[must_use]
    pub fn is_fallible(&self) -> bool {
        match self {
            Self::WithExpr(_, fallible) => *fallible,
            Self::CollectionMap(tokens) => tokens_contain_question_mark(tokens),
            _ => false,
        }
    }

    /// Get the token stream for transforms that contain tokens.
    ///
    /// Returns `Some(&TokenStream)` for `WithExpr` and `CollectionMap`,
    /// `None` for other variants.
    #[must_use]
    #[allow(dead_code)]
    pub fn tokens(&self) -> Option<&TokenStream> {
        match self {
            Self::WithExpr(tokens, _) | Self::CollectionMap(tokens) => Some(tokens),
            _ => None,
        }
    }
}

/// Check if tokens contain a `?` operator.
#[must_use]
pub fn tokens_contain_question_mark(tokens: &TokenStream) -> bool {
    use proc_macro2::TokenTree;
    tokens.clone().into_iter().any(|tt| match tt {
        TokenTree::Punct(p) => p.as_char() == '?',
        TokenTree::Group(g) => tokens_contain_question_mark(&g.stream()),
        _ => false,
    })
}
