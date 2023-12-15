use proc_macro2::TokenStream;

use crate::parse::{ParseOptions, ParseVariant};

mod parse;
mod write;

pub fn expand(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
    let mut result = parse::expand(options, variants)?;
    if options.write {
        result.extend(write::expand(options, variants));
    }
    Ok(result)
}
