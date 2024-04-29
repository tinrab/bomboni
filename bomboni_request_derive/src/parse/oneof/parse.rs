use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::parse::{
    oneof::utility::get_variant_extract,
    oneof::utility::get_variant_source_ident,
    options::{ParseDerive, ParseOptions, ParseTaggedUnion, ParseVariant},
    parse_utility::{expand_field_extract, expand_parse_field_type, make_field_error_path},
};

pub fn expand(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
    Ok(if let Some(tagged_union) = options.tagged_union.as_ref() {
        expand_tagged_union(options, variants, tagged_union)?
    } else {
        expand_parse(options, variants)?
    })
}

fn expand_parse(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
    let source = &options.source;
    let ident = &options.ident;
    let mut parse_variants = quote!();

    for variant in variants {
        if variant.options.skip || variant.options.derive.is_some() {
            continue;
        }

        let source_variant_ident = get_variant_source_ident(variant)?;
        let target_variant_ident = &variant.ident;

        if variant.fields.is_empty() {
            parse_variants.extend(if variant.source_unit {
                quote! {
                    #source::#source_variant_ident => {
                        #ident::#target_variant_ident
                    }
                }
            } else {
                quote! {
                    #source::#source_variant_ident(_) => {
                        #ident::#target_variant_ident
                    }
                }
            });
        } else {
            let parse_variant = expand_parse_variant(variant)?;
            parse_variants.extend(quote! {
                #source::#source_variant_ident(source) => {
                    #ident::#target_variant_ident({
                        #parse_variant
                    })
                }
            });
        }
    }

    let (impl_generics, type_generics, where_clause) = options.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics RequestParse<#source> for #ident #type_generics #where_clause {
            fn parse(source: #source) -> RequestResult<Self> {
                let variant_name = source.get_variant_name();
                Ok(match source {
                    #parse_variants
                    _ => {
                        return Err(RequestError::generic(CommonError::UnknownOneofVariant));
                    }
                })
            }
        }
    })
}

fn expand_tagged_union(
    options: &ParseOptions,
    variants: &[ParseVariant],
    tagged_union: &ParseTaggedUnion,
) -> syn::Result<TokenStream> {
    let ident = &options.ident;
    let oneof_ident = &tagged_union.oneof;

    let mut parse_variants = quote!();
    for variant in variants {
        if variant.options.skip {
            continue;
        }

        let source_variant_ident = get_variant_source_ident(variant)?;
        let target_variant_ident = &variant.ident;

        if variant.fields.is_empty() {
            parse_variants.extend(if variant.source_unit {
                quote! {
                    #oneof_ident::#source_variant_ident => {
                        #ident::#target_variant_ident
                    }
                }
            } else {
                quote! {
                    #oneof_ident::#source_variant_ident(_) => {
                        #ident::#target_variant_ident
                    }
                }
            });
        } else {
            let parse_variant = expand_parse_variant(variant)?;
            parse_variants.extend(quote! {
                #oneof_ident::#source_variant_ident(source) => {
                    #ident::#target_variant_ident({
                        #parse_variant
                    })
                }
            });
        }
    }

    let field_ident = &tagged_union.field;
    let field_literal = tagged_union.field.to_string();
    let source = &options.source;
    let (impl_generics, type_generics, where_clause) = options.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics RequestParse<#source> for #ident #type_generics #where_clause {
            #[allow(ignored_unit_patterns)]
            fn parse(source: #source) -> RequestResult<Self> {
                let source = source.#field_ident
                    .ok_or_else(|| RequestError::field(#field_literal, CommonError::RequiredFieldMissing))?;
                let variant_name = source.get_variant_name();
                Ok(match source {
                    #parse_variants
                    _ => {
                        return Err(RequestError::generic(CommonError::UnknownOneofVariant));
                    }
                })
            }
        }
    })
}

fn expand_parse_variant(variant: &ParseVariant) -> syn::Result<TokenStream> {
    let extract = get_variant_extract(variant)?;
    let field_error_path_wrapper = quote! {
        PathErrorStep::Field(variant_name.into())
    };

    if let Some(ParseDerive {
        parse,
        module,
        source_borrow,
        ..
    }) = variant.options.derive.as_ref()
    {
        let parse_impl = parse
            .as_ref()
            .map(ToTokens::to_token_stream)
            .or_else(|| module.as_ref().map(|module| quote!(#module::parse)))
            .unwrap();

        if variant.options.source.is_some() || variant.options.extract.is_some() {
            let (extract_impl, _get_impl, field_path) = expand_field_extract(
                &extract,
                &BTreeSet::new(),
                None,
                Some(&field_error_path_wrapper),
                *source_borrow,
            );
            let field_error_path =
                make_field_error_path(&field_path, Some(&field_error_path_wrapper));

            return Ok(quote! {
                #extract_impl
                let target = #parse_impl(target)
                    .map_err(|err: RequestError| err.wrap_path(#field_error_path))?;
                target
            });
        }

        let source_value = if *source_borrow {
            quote!(&source)
        } else {
            quote!(source)
        };

        return Ok(quote! {
            #parse_impl(#source_value)
                .map_err(|err: RequestError| err.wrap_field(variant_name))?
        });
    }

    if variant.options.keep {
        return Ok(quote! {
            source.clone()
        });
    }

    let field_type_info = variant.type_info.as_ref().unwrap();
    let (extract_impl, get_impl, field_path) = expand_field_extract(
        &extract,
        &BTreeSet::new(),
        Some(field_type_info),
        Some(&field_error_path_wrapper),
        false,
    );
    let field_error_path = make_field_error_path(&field_path, Some(&field_error_path_wrapper));
    let parse_field_impl = expand_parse_field_type(
        &variant.options,
        field_type_info,
        field_error_path,
        get_impl,
    );

    Ok(quote! {
        #extract_impl
        #parse_field_impl
        target
    })
}
