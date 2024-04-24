use proc_macro2::TokenStream;

use crate::parse::options::{ParseOptions, ParseVariant};

mod parse;
mod utility;
mod write;

pub fn expand(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
    for variant in variants {
        if variant.options.extract.is_some() && variant.options.source.is_some() {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "`extract` and `source` cannot be used together",
            ));
        }

        if variant.options.wrapper
            && (variant.options.keep
                || variant.options.keep_primitive
                || variant.options.derive.is_some()
                || variant.options.enumeration)
        {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "`wrapper` cannot be used with these options`",
            ));
        }

        if variant.options.derive.is_some() && variant.options.source.is_some() {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "`derive` and `source` cannot be used together",
            ));
        }
    }

    let mut result = parse::expand(options, variants)?;
    if options.write {
        result.extend(write::expand(options, variants)?);
    }

    Ok(result)
}
