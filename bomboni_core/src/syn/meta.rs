use syn::{
    Attribute, Expr, ExprLit, ExprPath, Ident, Lit, Meta, MetaList, MetaNameValue, Path, Token,
    punctuated::Punctuated,
};

/// Parses all list entries from attributes with the given name.
///
/// # Errors
///
/// Returns an error if a matching attribute is not a list or its entries are invalid.
pub fn attribute_metas(attrs: &[Attribute], name: &str) -> syn::Result<Vec<Meta>> {
    let mut metas = Vec::new();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident(name)) {
        let Meta::List(list) = &attr.meta else {
            return Err(syn::Error::new_spanned(attr, "expected attribute options"));
        };
        metas.extend(meta_list(list)?);
    }
    Ok(metas)
}

/// Parses a comma-separated metadata list.
///
/// # Errors
///
/// Returns an error if the list does not contain valid comma-separated metadata.
pub fn meta_list(list: &MetaList) -> syn::Result<Punctuated<Meta, Token![,]>> {
    list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
}

/// Returns the expression from a name-value metadata item.
///
/// # Errors
///
/// Returns an error if the metadata item is not a name-value pair.
pub fn meta_expr(meta: &Meta) -> syn::Result<&Expr> {
    match meta {
        Meta::NameValue(MetaNameValue { value, .. }) => Ok(value),
        _ => Err(syn::Error::new_spanned(meta, "expected a value")),
    }
}

/// Parses a path expression, accepting either a path or a string literal.
///
/// # Errors
///
/// Returns an error if the expression does not contain a valid path.
pub fn expr_path(expr: &Expr) -> syn::Result<Path> {
    match expr {
        Expr::Path(ExprPath { path, .. }) => Ok(path.clone()),
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => value.parse(),
        _ => Err(syn::Error::new_spanned(expr, "expected a path")),
    }
}

/// Parses a path from a name-value metadata item.
///
/// # Errors
///
/// Returns an error if the metadata item does not contain a valid path.
pub fn meta_path(meta: &Meta) -> syn::Result<Path> {
    expr_path(meta_expr(meta)?)
}

/// Parses a path stored specifically as a string literal.
///
/// # Errors
///
/// Returns an error if the metadata value is not a string containing a valid path.
pub fn meta_string_path(meta: &Meta) -> syn::Result<Path> {
    let value = meta_string(meta)?;
    syn::parse_str(&value).map_err(|_| syn::Error::new_spanned(meta, "expected a path"))
}

/// Parses a boolean flag or a boolean name-value metadata item.
///
/// # Errors
///
/// Returns an error if the metadata item is neither a flag nor a boolean value.
pub fn meta_bool(meta: &Meta) -> syn::Result<bool> {
    match meta {
        Meta::Path(_) => Ok(true),
        Meta::NameValue(MetaNameValue {
            value:
                Expr::Lit(ExprLit {
                    lit: Lit::Bool(value),
                    ..
                }),
            ..
        }) => Ok(value.value),
        _ => Err(syn::Error::new_spanned(meta, "expected a boolean")),
    }
}

/// Validates and parses a flag-only metadata item.
///
/// # Errors
///
/// Returns an error if the metadata item is not a flag.
pub fn meta_flag(meta: &Meta) -> syn::Result<bool> {
    if matches!(meta, Meta::Path(_)) {
        Ok(true)
    } else {
        Err(syn::Error::new_spanned(meta, "expected a flag"))
    }
}

/// Parses a string literal from a name-value metadata item.
///
/// # Errors
///
/// Returns an error if the metadata value is not a string literal.
pub fn meta_string(meta: &Meta) -> syn::Result<String> {
    match meta_expr(meta)? {
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => Ok(value.value()),
        expr => Err(syn::Error::new_spanned(expr, "expected a string literal")),
    }
}

/// Parses an expression path from a name-value metadata item.
///
/// # Errors
///
/// Returns an error if the metadata value is not a path expression.
pub fn meta_expr_path(meta: &Meta) -> syn::Result<ExprPath> {
    match meta_expr(meta)? {
        Expr::Path(path) => Ok(path.clone()),
        expr => Err(syn::Error::new_spanned(expr, "expected a path")),
    }
}

/// Parses a single identifier from a path metadata value.
///
/// # Errors
///
/// Returns an error if the metadata value is not a single identifier.
pub fn meta_ident(meta: &Meta) -> syn::Result<Ident> {
    meta_path(meta)?
        .get_ident()
        .cloned()
        .ok_or_else(|| syn::Error::new_spanned(meta, "expected an identifier"))
}
