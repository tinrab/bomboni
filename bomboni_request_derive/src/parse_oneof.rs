use darling::FromMeta;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, ToTokens};

use crate::parse::{ParseOptions, ParseTaggedUnion, ParseVariant};
use crate::utility::{get_proto_type_info, ProtoTypeInfo};

pub fn expand(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
    let (parse, write) = if let Some(tagged_union) = options.tagged_union.as_ref() {
        (
            expand_parse_tagged_union(options, variants, tagged_union)?,
            if options.write {
                expand_write_tagged_union(options, variants, tagged_union)
            } else {
                quote!()
            },
        )
    } else {
        (
            expand_parse(options, variants)?,
            if options.write {
                expand_write(options, variants)
            } else {
                quote!()
            },
        )
    };
    Ok(quote! {
        #parse
        #write
    })
}

fn expand_parse(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
    let source = &options.source;
    let ident = &options.ident;

    let mut parse_variants = quote!();
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

        let parse_variant = expand_parse_variant(variant)?;
        parse_variants.extend(quote! {
            #source::#source_variant_ident(source) => {
                #ident::#target_variant_ident({
                    #parse_variant
                })
            }
        });
    }

    Ok(quote! {
        impl RequestParse<#source> for #ident {
            type Error = RequestError;

            fn parse(source: #source) -> Result<Self, Self::Error> {
                let variant_name = source.get_variant_name();
                Ok(match source {
                    #parse_variants
                    _ => {
                        return Err(RequestError::domain(CommonError::UnknownOneofVariant));
                    }
                })
            }
        }
    })
}

fn expand_parse_tagged_union(
    options: &ParseOptions,
    variants: &[ParseVariant],
    tagged_union: &ParseTaggedUnion,
) -> syn::Result<TokenStream> {
    let source = &options.source;
    let ident = &options.ident;
    let oneof_ident = &tagged_union.oneof;
    let field_ident = &tagged_union.field;
    let field_literal = Literal::string(&tagged_union.field.to_string());

    let mut parse_variants = quote!();
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
            // Handle unit variants
            parse_variants.extend(quote! {
                #oneof_ident::#source_variant_ident(_) => #ident::#target_variant_ident,
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

    Ok(quote! {
        impl RequestParse<#source> for #ident {
            type Error = RequestError;

            #[allow(ignored_unit_patterns)]
            fn parse(source: #source) -> Result<Self, Self::Error> {
                let source = source.#field_ident
                    .ok_or_else(|| RequestError::field(#field_literal, CommonError::RequiredFieldMissing))?;
                let variant_name = source.get_variant_name();
                Ok(match source {
                    #parse_variants
                    _ => {
                        return Err(RequestError::domain(CommonError::UnknownOneofVariant));
                    }
                })
            }
        }
    })
}

fn expand_parse_variant(variant: &ParseVariant) -> syn::Result<TokenStream> {
    if variant.fields.len() != 1 {
        return Err(syn::Error::new_spanned(
            &variant.ident,
            "oneof variants cannot be tuples, units or structs",
        ));
    }

    let variant_type = variant.fields.iter().next().unwrap();
    let ProtoTypeInfo {
        is_option,
        is_nested,
        is_string,
        is_box,
        ..
    } = get_proto_type_info(variant_type);

    if (variant.with.is_some() || variant.parse_with.is_some()) && variant.regex.is_some() {
        return Err(syn::Error::new_spanned(
            &variant.ident,
            "some of these options cannot be used alongside `with`",
        ));
    }
    if variant.regex.is_some() && !is_string {
        return Err(syn::Error::new_spanned(
            &variant.ident,
            "regex can only be used with string variants",
        ));
    }
    if variant.with.is_some() && (variant.parse_with.is_some() || variant.write_with.is_some()) {
        return Err(syn::Error::new_spanned(
            &variant.ident,
            "custom `parse_with` and `write_with` functions cannot be used alongside `with`",
        ));
    }

    let parse_source = if variant.with.is_some() || variant.parse_with.is_some() {
        let parse_with = if let Some(with) = variant.with.as_ref() {
            quote! {
                #with::parse
            }
        } else {
            variant.parse_with.as_ref().unwrap().to_token_stream()
        };
        quote! {
            let target = #parse_with(target)
                .map_err(|err: RequestError| err.wrap(variant_name))?;
        }
    } else if variant.enumeration {
        quote! {
            let target = target.try_into()
                .map_err(|_| RequestError::field(variant_name, CommonError::InvalidEnumValue))?;
        }
    } else if is_nested {
        let parse_source = if is_box || variant.source_box {
            quote! {
                let target = *target;
            }
        } else {
            quote!()
        };
        quote! {
            #parse_source
            let target = target.parse_into()
                .map_err(|err: RequestError| err.wrap(variant_name))?;
        }
    } else if is_string {
        if let Some(regex) = variant.regex.as_ref() {
            quote! {
                static REGEX: ::std::sync::OnceLock<::regex::Regex> = ::std::sync::OnceLock::new();
                let re = REGEX.get_or_init(|| ::regex::Regex::new(#regex).unwrap());
                if !re.is_match(&target) {
                    return Err(RequestError::field(
                        variant_name,
                        CommonError::InvalidStringFormat {
                            expected: #regex.into(),
                        },
                    ));
                }
            }
        } else {
            quote!()
        }
    } else if is_box || variant.source_box {
        quote! {
            let target = *target;
        }
    } else {
        quote!()
    };

    let mut parse = quote! {
        let target = source;
    };

    let default_expr = if let Some(default) = variant.default.as_ref() {
        quote! { #default }
    } else {
        quote! { Default::default() }
    };

    if is_option {
        if variant.source_option {
            if is_string {
                parse.extend(quote! {
                    let target = if let Some(target) = target.filter(|target| !target.is_empty()) {
                        #parse_source
                        Some(target)
                    } else {
                        None
                    };
                });
            } else if variant.enumeration {
                parse.extend(quote! {
                    let target = if let Some(target) = target.filter(|e| *e != 0) {
                        #parse_source
                        Some(target)
                    } else {
                        None
                    };
                });
            } else {
                parse.extend(quote! {
                    let target = if let Some(target) = target {
                        #parse_source
                        Some(target)
                    } else {
                        None
                    };
                });
            }
        } else {
            parse.extend(if is_string {
                quote! {
                    let target = if target.is_empty() {
                        None
                    } else {
                        #parse_source
                        Some(target)
                    };
                }
            } else if variant.enumeration {
                quote! {
                    let target = if target == 0 {
                        None
                    } else {
                        #parse_source
                        Some(target)
                    };
                }
            } else {
                quote! {
                    #parse_source
                    let target = Some(target);
                }
            });
        }
    } else {
        if variant.source_option {
            if variant.default.is_some() {
                parse.extend(quote! {
                    let target = target.unwrap_or_else(|| #default_expr);
                });
            } else {
                parse.extend(quote! {
                    let target = target.ok_or_else(|| {
                        RequestError::field(
                            variant_name,
                            CommonError::RequiredFieldMissing,
                        )
                    })?;
                });
            }
        }
        parse.extend(parse_source);
    }

    if is_box {
        if is_option {
            parse.extend(quote! {
                let target = target.map(Box::new);
            });
        } else {
            parse.extend(quote! {
                let target = Box::new(target);
            });
        }
    }

    Ok(quote! {
        #parse
        target
    })
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
            // Handle unit variants
            write_variants.extend(quote! {
                #ident::#target_variant_ident => #source::#source_variant_ident(Default::default()),
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
            // Handle unit variants
            write_variants.extend(quote! {
                #ident::#target_variant_ident => #oneof_ident::#source_variant_ident(Default::default()),
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

    let write_target = if variant.with.is_some() || variant.write_with.is_some() {
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

    let mut write = quote! {
        let source = value;
    };

    let default_expr = if let Some(default) = variant.default.as_ref() {
        quote! { #default }
    } else {
        quote! { Default::default() }
    };

    if is_option {
        write.extend(if variant.source_option {
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
        write.extend(if variant.source_option {
            quote! {
                #write_target
                let source = Some(source);
            }
        } else {
            write_target
        });
    }

    if is_box || variant.source_box {
        if variant.source_option {
            write.extend(quote! {
                let source = source.map(Box::new);
            });
        } else {
            write.extend(quote! {
                let source = Box::new(source);
            });
        }
    }

    quote! {
        #write
        source
    }
}
