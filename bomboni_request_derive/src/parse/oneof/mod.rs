use proc_macro2::TokenStream;

use crate::parse::{ParseOptions, ParseVariant};

mod parse;
mod write;

pub fn expand(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
    if options.list_query.is_some() || options.search_query.is_some() {
        return Err(syn::Error::new_spanned(
            &options.ident,
            "enums cannot be used with `list_query` or `search_query`",
        ));
    }

    let mut result = parse::expand(options, variants)?;
    if options.write {
        result.extend(write::expand(options, variants));
    }

    Ok(result)
}
