use proc_macro2::TokenStream;

use crate::parse::ParseOptions;

use super::ParseField;

mod parse;
mod write;

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> syn::Result<TokenStream> {
    let mut result = parse::expand(options, fields)?;
    if options.write {
        result.extend(write::expand(options, fields));
    }
    Ok(result)
}
