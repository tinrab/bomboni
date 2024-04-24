use proc_macro2::TokenStream;

use crate::parse::options::{ParseField, ParseOptions};

mod parse;
mod utility;
mod write;

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> syn::Result<TokenStream> {
    let mut contains_query = false;
    for field in fields {
        if field.list_query.is_some() || field.search_query.is_some() {
            if contains_query {
                return Err(syn::Error::new_spanned(
                    &field.ident,
                    "can only have one list or search query field",
                ));
            }
            contains_query = true;
        }
        if field.list_query.is_some() && field.search_query.is_some() {
            return Err(syn::Error::new_spanned(
                &field.ident,
                "list and search query cannot be used together",
            ));
        }

        if field.options.extract.is_some() && field.options.source.is_some() {
            return Err(syn::Error::new_spanned(
                &field.ident,
                "`extract` and `source` cannot be used together",
            ));
        }

        if field.options.wrapper
            && (field.options.keep
                || field.options.keep_primitive
                || field.options.derive.is_some()
                || field.options.enumeration
                || field.resource.is_some()
                || field.oneof
                || field.list_query.is_some()
                || field.search_query.is_some())
        {
            return Err(syn::Error::new_spanned(
                &field.ident,
                "`wrapper` cannot be used with these options`",
            ));
        }

        if (field.list_query.is_some() || field.search_query.is_some())
            && (field.options.keep
                || field.options.keep_primitive
                || field.options.derive.is_some()
                || field.options.enumeration
                || field.resource.is_some()
                || field.oneof)
        {
            return Err(syn::Error::new_spanned(
                &field.ident,
                "query fields cannot be used with these options",
            ));
        }
    }

    let mut result = parse::expand(options, fields)?;
    if options.write {
        result.extend(write::expand(options, fields)?);
    }

    Ok(result)
}
