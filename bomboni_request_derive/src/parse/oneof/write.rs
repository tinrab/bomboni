use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::parse::{
    oneof::utility::{get_variant_extract, get_variant_source_ident},
    options::{ParseDerive, ParseOptions, ParseTaggedUnion, ParseVariant},
    write_utility::{expand_field_inject, expand_write_field_type},
};

pub fn expand(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
    if let Some(tagged_union) = options.tagged_union.as_ref() {
        expand_write_tagged_union(options, variants, tagged_union)
    } else {
        expand_write(options, variants)
    }
}

fn expand_write(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
    let source = &options.source;
    let ident = &options.ident;

    let mut write_variants = quote!();

    for variant in variants {
        if variant.options.skip {
            continue;
        }

        let source_variant_ident = get_variant_source_ident(variant)?;
        let target_variant_ident = &variant.ident;

        if variant.fields.is_empty() {
            write_variants.extend(if variant.source_unit {
                quote! {
                    #ident::#target_variant_ident => {
                        #source::#source_variant_ident
                    }
                }
            } else {
                quote! {
                    #ident::#target_variant_ident => {
                        #source::#source_variant_ident(Default::default())
                    }
                }
            });
        } else {
            let write_variant = expand_write_variant(variant)?;
            write_variants.extend(quote! {
                #ident::#target_variant_ident(target) => {
                    let mut source = #source::#source_variant_ident(Default::default());
                    if let #source::#source_variant_ident(source) = &mut source {
                        #write_variant
                    }
                    source
                }
            });
        }
    }

    let (impl_generics, type_generics, where_clause) = options.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics From<#ident #type_generics> for #source #where_clause {
            fn from(target: #ident #type_generics) -> Self {
                match target {
                    #write_variants
                    _ => panic!("unknown oneof variant"),
                }
            }
        }
    })
}

fn expand_write_tagged_union(
    options: &ParseOptions,
    variants: &[ParseVariant],
    tagged_union: &ParseTaggedUnion,
) -> syn::Result<TokenStream> {
    let ident = &options.ident;
    let oneof_ident = &tagged_union.oneof;

    let mut write_variants = quote!();
    for variant in variants {
        if variant.options.skip {
            continue;
        }

        let source_variant_ident = get_variant_source_ident(variant)?;
        let target_variant_ident = &variant.ident;

        if variant.fields.is_empty() {
            write_variants.extend(if variant.source_unit {
                quote! {
                    #ident::#target_variant_ident => {
                        #oneof_ident::#source_variant_ident
                    }
                }
            } else {
                quote! {
                    #ident::#target_variant_ident => {
                        #oneof_ident::#source_variant_ident(Default::default())
                    }
                }
            });
        } else {
            let write_variant = expand_write_variant(variant)?;
            write_variants.extend(quote! {
                #ident::#target_variant_ident(target) => {
                    let mut source = #oneof_ident::#source_variant_ident(Default::default());
                    if let #oneof_ident::#source_variant_ident(source) = &mut source {
                        #write_variant
                    }
                    source
                }
            });
        }
    }

    let source = &options.source;
    let field_ident = &tagged_union.field;
    let (impl_generics, type_generics, where_clause) = options.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics From<#ident #type_generics> for #source #where_clause {
            fn from(target: #ident #type_generics) -> Self {
                #source {
                    #field_ident: Some(match target {
                        #write_variants
                        _ => panic!("unknown oneof variant"),
                    }),
                }
            }
        }
    })
}

fn expand_write_variant(variant: &ParseVariant) -> syn::Result<TokenStream> {
    let target_ident = &variant.ident;
    let extract = get_variant_extract(variant)?;

    if let Some(ParseDerive {
        write,
        module,
        target_borrow,
        ..
    }) = variant.options.derive.as_ref()
    {
        let write_impl = write
            .as_ref()
            .map(ToTokens::to_token_stream)
            .or_else(|| module.as_ref().map(|module| quote!(#module::write)))
            .ok_or_else(|| {
                syn::Error::new_spanned(target_ident, "missing derive write implementation")
            })?;

        if variant.options.source.is_some() || variant.options.extract.is_some() {
            let inject_impl = expand_field_inject(&extract, &variant.options, None);
            let source_value = if *target_borrow {
                quote!(&target)
            } else {
                quote!(target)
            };

            return Ok(quote! {
                let source_field = #write_impl(#source_value);
                #inject_impl
            });
        }

        let target_value = if *target_borrow {
            quote!(&target)
        } else {
            quote!(target)
        };
        return Ok(quote! {
            *source = #write_impl(#target_value);
        });
    }

    let field_type_info = variant.type_info.as_ref().unwrap();
    let inject_impl = expand_field_inject(&extract, &variant.options, Some(field_type_info));
    let write_field_impl = expand_write_field_type(&variant.options, field_type_info, inject_impl);

    Ok(quote! {
        let source_field = target;
        #write_field_impl
    })
}
