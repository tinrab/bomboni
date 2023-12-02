use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::parse::{ParseOptions, ParseVariant};
use crate::utility::check_proto_type;

pub fn expand(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
    let parse = expand_parse(options, variants)?;
    let write = if options.write {
        expand_write(options, variants)?
    } else {
        quote!()
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

    if let Some(with) = variant.with.as_ref() {
        return Ok(quote! {
            #with(source)
        });
    }

    let variant_type = variant.fields.iter().next().unwrap();
    let (is_option, is_nested, is_string) = check_proto_type(variant_type);

    let mut result = quote! {
        let target = source;
    };

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

    if variant.with.is_some() || variant.parse_with.is_some() {
        let parse_with = if let Some(with) = variant.with.as_ref() {
            quote! {
                #with::parse(target)
            }
        } else {
            variant.parse_with.as_ref().unwrap().to_token_stream()
        };

        result.extend(quote! {
            let target = #parse_with(target)?;
        });
        if is_option {
            result.extend(quote! {
                let target = Some(target);
            });
        }
    } else if is_nested {
        result.extend(quote! {
            let target = target.parse_into()?;
        });
    } else if is_string {
        if let Some(regex) = variant.regex.as_ref() {
            result.extend(quote! {{
                static REGEX: ::std::sync::OnceLock<::regex::Regex> = ::std::sync::OnceLock::new();
                let re = REGEX.get_or_init(|| ::regex::Regex::new(#regex).unwrap());
                if !re.is_match(&target) {
                    return Err(CommonError::InvalidStringFormat {
                        expected: #regex.into(),
                    }.into());
                }
            }});
        }
        if is_option {
            result.extend(quote! {
                let target = if target.is_empty() {
                    None
                } else {
                    Some(target)
                };
            });
        } else {
            result.extend(quote! {
                if target.is_empty() {
                    return Err(CommonError::RequiredFieldMissing.into());
                }
            });
        }
    } else {
        // Parse primitive
        if is_option {
            result.extend(quote! {
                let target = Some(target);
            });
        }
    }

    Ok(quote! {
        #result
        target
    })
}

fn expand_write(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
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

        let write_variant = expand_write_variant(variant)?;
        write_variants.extend(quote! {
            #ident::#target_variant_ident(source) => {
                #source::#source_variant_ident({
                    #write_variant
                })
            }
        });
    }

    Ok(quote! {
        impl From<#ident> for #source {
            fn from(value: #ident) -> Self {
                match value {
                    #write_variants
                    _ => panic!("unknown oneof variant"),
                }
            }
        }
    })
}

fn expand_write_variant(variant: &ParseVariant) -> syn::Result<TokenStream> {
    let variant_type = variant.fields.iter().next().unwrap();
    let (is_option, is_nested, _is_string) = check_proto_type(variant_type);
    if is_option {
        return Err(syn::Error::new_spanned(
            &variant.ident,
            "oneof variants cannot be optional",
        ));
    }

    let mut result = quote! {
        let target = source;
    };

    if let Some(with) = variant.with.as_ref() {
        result.extend(quote! {
            let target = #with::write(target);
        });
    } else if let Some(write_with) = variant.write_with.as_ref() {
        result.extend(quote! {
            let target = #write_with(target);
        });
    } else if is_nested {
        result.extend(quote! {
            let target = target.into();
        });
    }

    Ok(quote! {
        #result
        target
    })
}
