use std::collections::BTreeSet;

use bomboni_core::format_comment;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::parse::{
    field_type_info::get_field_type_info,
    oneof::utility::get_variant_extract,
    oneof::utility::get_variant_source_ident,
    options::{ParseDerive, ParseOptions, ParseTaggedUnion, ParseVariant},
    parse_utility::{expand_field_extract, expand_field_parse_type, make_field_error_path},
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
            let parse_variant = expand_parse_variant(options, variant)?;
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
            let parse_variant = expand_parse_variant(options, variant)?;
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

fn expand_parse_variant(
    options: &ParseOptions,
    variant: &ParseVariant,
) -> syn::Result<TokenStream> {
    if let Some(ParseDerive {
        parse,
        module,
        borrowed,
        ..
    }) = variant.options.derive.as_ref()
    {
        if let Some(parse_impl) = parse
            .as_ref()
            .map(ToTokens::to_token_stream)
            .or_else(|| module.as_ref().map(|module| quote!(#module::parse)))
        {
            let value = if *borrowed {
                quote!(&source)
            } else {
                quote!(source)
            };
            return Ok(quote! {
                #parse_impl(#value)
                    .map_err(|err: RequestError| err.wrap_field(variant_name))?
            });
        }
    }

    if variant.options.keep {
        return Ok(quote! {
            source.clone()
        });
    }

    let variant_type = variant.fields.iter().next().unwrap();
    let field_type_info = get_field_type_info(options, &variant.options, variant_type)?;

    let extract = get_variant_extract(variant)?;
    let field_error_path_wrapper = quote! {
        PathErrorStep::Field(variant_name.into())
    };
    let (extract_impl, field_path) =
        expand_field_extract(&extract, &BTreeSet::new(), Some(&field_error_path_wrapper));

    let field_error_path = make_field_error_path(&field_path, Some(&field_error_path_wrapper));
    let parse_inner_impl =
        expand_field_parse_type(&variant.options, &field_type_info, field_error_path);

    let comment = format_comment!(
        "\nParse variant `{}`\n{:#?}\n{:#?}",
        &variant.ident,
        field_type_info,
        extract,
    );

    Ok(quote! {
        #comment
        {
            #extract_impl
            #parse_inner_impl
            target
        }
    })
}
