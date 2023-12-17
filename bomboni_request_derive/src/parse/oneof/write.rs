use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::parse::{ParseOptions, ParseTaggedUnion, ParseVariant};
use crate::utility::{get_proto_type_info, ProtoTypeInfo};

pub fn expand(options: &ParseOptions, variants: &[ParseVariant]) -> TokenStream {
    if let Some(tagged_union) = options.tagged_union.as_ref() {
        expand_write_tagged_union(options, variants, tagged_union)
    } else {
        expand_write(options, variants)
    }
}

fn expand_write(options: &ParseOptions, variants: &[ParseVariant]) -> TokenStream {
    let source = &options.source;
    let ident = &options.ident;

    let mut write_variants = quote!();

    for variant in variants {
        if variant.skip {
            continue;
        }

        let source_variant_ident = if let Some(name) = variant.source_name.as_ref() {
            Ident::from_string(name).unwrap()
        } else {
            variant.ident.clone()
        };
        let target_variant_ident = &variant.ident;

        if variant.fields.is_empty() {
            write_variants.extend(if variant.source_empty {
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
            let write_variant = expand_write_variant(variant);
            write_variants.extend(quote! {
                #ident::#target_variant_ident(value) => {
                    #source::#source_variant_ident({
                        #write_variant
                    })
                }
            });
        }
    }

    quote! {
        impl From<#ident> for #source {
            fn from(value: #ident) -> Self {
                match value {
                    #write_variants
                    _ => panic!("unknown oneof variant"),
                }
            }
        }
    }
}

fn expand_write_tagged_union(
    options: &ParseOptions,
    variants: &[ParseVariant],
    tagged_union: &ParseTaggedUnion,
) -> TokenStream {
    let source = &options.source;
    let ident = &options.ident;
    let oneof_ident = &tagged_union.oneof;
    let field_ident = &tagged_union.field;

    let mut write_variants = quote!();
    for variant in variants {
        if variant.skip {
            continue;
        }

        let source_variant_ident = if let Some(name) = variant.source_name.as_ref() {
            Ident::from_string(name).unwrap()
        } else {
            variant.ident.clone()
        };
        let target_variant_ident = &variant.ident;

        if variant.fields.is_empty() {
            write_variants.extend(if variant.source_empty {
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
            let write_variant = expand_write_variant(variant);
            write_variants.extend(quote! {
                #ident::#target_variant_ident(value) => {
                    #oneof_ident::#source_variant_ident({
                        #write_variant
                    })
                }
            });
        }
    }

    quote! {
        impl From<#ident> for #source {
            fn from(value: #ident) -> Self {
                #source {
                    #field_ident: Some(match value {
                        #write_variants
                        _ => panic!("unknown oneof variant"),
                    }),
                }
            }
        }
    }
}

fn expand_write_variant(variant: &ParseVariant) -> TokenStream {
    let variant_type = variant.fields.iter().next().unwrap();
    let ProtoTypeInfo {
        is_option,
        is_nested,
        is_box,
        ..
    } = get_proto_type_info(variant_type);

    let mut write_target = if variant.keep {
        if is_box {
            quote! {
                let source = *source;
            }
        } else {
            quote!()
        }
    } else if variant.with.is_some() || variant.write_with.is_some() {
        let write_with = if let Some(with) = variant.with.as_ref() {
            quote! {
                #with::write
            }
        } else {
            variant.write_with.as_ref().unwrap().to_token_stream()
        };
        quote! {
            let source = #write_with(source);
        }
    } else if variant.enumeration {
        quote! {
            let source = source as i32;
        }
    } else if is_nested {
        let write_target = if is_box {
            quote! {
                let source = *source;
            }
        } else {
            quote!()
        };
        quote! {
            #write_target
            let source = source.into();
        }
    } else if is_box {
        quote! {
            let source = *source;
        }
    } else {
        quote!()
    };

    if let Some(source_try_from) = variant.source_try_from.as_ref() {
        let err_literal = format!(
            "failed to convert `{}` to `{}`",
            &variant.ident,
            source_try_from.to_token_stream(),
        );
        write_target.extend(quote! {
            let source: #source_try_from = source.try_into()
                .expect(#err_literal);
        });
    }

    let mut write = quote! {
        let source = value;
    };

    let default_expr = if let Some(default) = variant.default.as_ref() {
        quote! { #default }
    } else {
        quote! { Default::default() }
    };

    let source_option = variant.source_option || is_option;

    if is_option {
        write.extend(if source_option {
            quote! {
                let source = if let Some(source) = source {
                    #write_target
                    Some(source)
                } else {
                    None
                };
            }
        } else {
            quote! {
                let source = source.unwrap_or_else(|| #default_expr);
                #write_target
            }
        });
    } else {
        write.extend(if source_option {
            quote! {
                #write_target
                let source = Some(source);
            }
        } else {
            write_target
        });
    }

    if is_box || variant.source_box {
        if source_option {
            write.extend(quote! {
                let source = source.map(Box::new);
            });
        } else {
            write.extend(quote! {
                let source = Box::new(source);
            });
        }
    }

    if variant.wrapper {
        write.extend(quote! {
            let source: #variant_type = source;
            let source = source.into();
        });
    }

    quote! {
        #write
        source
    }
}
