use proc_macro2::Ident;
use quote::format_ident;

use crate::parse::{
    options::{FieldExtract, ParseVariant},
    parse_utility::parse_field_source_extract,
};

pub fn get_variant_extract(variant: &ParseVariant) -> syn::Result<FieldExtract> {
    let target_ident = &variant.ident;
    let mut steps = if let Some(source) = variant.options.source.as_ref() {
        let mut source_extract = parse_field_source_extract(source)
            .ok_or_else(|| syn::Error::new_spanned(target_ident, "invalid source"))?;
        if !source_extract.steps.is_empty() {
            source_extract.steps.remove(0);
        }
        source_extract.steps
    } else {
        Vec::new()
    };
    if let Some(extract) = variant.options.extract.clone() {
        steps.extend(extract.steps);
    }
    Ok(FieldExtract { steps })
}

pub fn get_variant_source_ident(variant: &ParseVariant) -> syn::Result<Ident> {
    Ok(
        if let Some(variant_source) = variant.options.source.as_ref() {
            if variant_source.contains('.') || variant_source.contains('?') {
                format_ident!(
                    "{}",
                    variant_source
                        .split('.')
                        .next()
                        .map(|part| part.trim_matches('?'))
                        .ok_or_else(|| syn::Error::new_spanned(&variant.ident, "invalid source"))?
                )
            } else {
                format_ident!("{}", variant_source)
            }
        } else {
            variant.ident.clone()
        },
    )
}