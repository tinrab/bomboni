use proc_macro2::TokenStream;

use crate::parse::options::{ParseField, ParseOptions};

mod parse;
mod utility;
mod write;

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> syn::Result<TokenStream> {
    let mut result = parse::expand(options, fields)?;
    if options.write {
        result.extend(write::expand(options, fields)?);
    }
    Ok(result)
}
