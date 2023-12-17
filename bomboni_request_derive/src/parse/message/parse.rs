use darling::FromMeta;
use itertools::Itertools;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, ToTokens};

use crate::parse::{DeriveOptions, ParseField, ParseOptions};
use crate::utility::{get_proto_type_info, ProtoTypeInfo};

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> syn::Result<TokenStream> {
    let source = &options.source;
    let ident = &options.ident;

    let mut parse_fields = quote!();
    // Set default for skipped fields
    let mut skipped_fields = quote!();

    // Parse fields in order, starting with derived ones.
    // This is needed because derived fields may depend on other fields, and we want to avoid unnecessary cloning.
    for field in fields {
        if field.derive.is_some() {
            parse_fields.extend(expand_parse_field(field)?);
        }
    }
    // Parse resource fields
    for field in fields {
        if field.resource.is_some() {
            parse_fields.extend(expand_parse_resource_field(field)?);
        }
    }

    for field in fields {
        if field.derive.is_some() || field.resource.is_some() {
            continue;
        }

        let field_ident = field.ident.as_ref().unwrap();
        if field.skip {
            skipped_fields.extend(quote! {
                #field_ident: Default::default(),
            });
            continue;
        }

        parse_fields.extend(expand_parse_field(field)?);
    }

    Ok(quote! {
        impl RequestParse<#source> for #ident {
            type Error = RequestError;

            #[allow(clippy::ignored_unit_patterns)]
            fn parse(source: #source) -> Result<Self, Self::Error> {
                Ok(Self {
                    #parse_fields
                    #skipped_fields
                })
            }
        }
    })
}

fn expand_parse_field(field: &ParseField) -> syn::Result<TokenStream> {
    let target_ident = field.ident.as_ref().unwrap();

    if let Some(DeriveOptions { func, source_field }) = field.derive.as_ref() {
        return Ok(if let Some(source_field) = source_field.as_ref() {
            let source_field_name = Literal::string(
                &source_field
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .join("."),
            );
            quote! {
                #target_ident: { #func(&source.#source_field, #source_field_name)? },
            }
        } else {
            quote! {
                #target_ident: { #func(&source)? },
            }
        });
    }

    let field_name = if let Some(name) = field.name.as_ref() {
        quote! { #name }
    } else {
        field
            .ident
            .as_ref()
            .unwrap()
            .to_string()
            .into_token_stream()
    };
    let source_ident = if let Some(name) = field.source_name.as_ref() {
        Ident::from_string(name).unwrap()
    } else {
        field.ident.clone().unwrap()
    };

    let field_type = &field.ty;
    let ProtoTypeInfo {
        is_option,
        is_nested,
        is_string,
        is_box,
        is_vec,
        map_ident,
        ..
    } = get_proto_type_info(field_type);

    if (field.with.is_some() || field.parse_with.is_some())
        && (field.enumeration || field.oneof || field.regex.is_some())
    {
        return Err(syn::Error::new_spanned(
            &field.ident,
            "some of these options cannot be used alongside `with`",
        ));
    }
    if field.regex.is_some() && !is_string {
        return Err(syn::Error::new_spanned(
            &field.ident,
            "`regex` can only be used with string fields",
        ));
    }
    if field.with.is_some() && (field.parse_with.is_some() || field.write_with.is_some()) {
        return Err(syn::Error::new_spanned(
            &field.ident,
            "custom `parse_with` and `write_with` functions cannot be used alongside `with`",
        ));
    }
    if field.wrapper && field.source_try_from.is_some() {
        return Err(syn::Error::new_spanned(
            &field.ident,
            "wrapper fields cannot be casted",
        ));
    }
    if field.keep
        && (field.skip
            || field.wrapper
            || field.enumeration
            || field.oneof
            || field.regex.is_some()
            || field.source_try_from.is_some()
            || field.with.is_some()
            || field.parse_with.is_some()
            || field.write_with.is_some()
            || field.resource.is_some())
    {
        return Err(syn::Error::new_spanned(
            &field.ident,
            "some of these options cannot be used alongside `keep`",
        ));
    }

    let mut parse_source = if field.keep {
        if is_box || field.source_box {
            quote! {
                let target = *target;
            }
        } else {
            quote!()
        }
    } else if field.with.is_some() || field.parse_with.is_some() {
        let parse_with = if let Some(with) = field.with.as_ref() {
            quote! {
                #with::parse
            }
        } else {
            field.parse_with.as_ref().unwrap().to_token_stream()
        };
        quote! {
            let target = #parse_with(target)
                .map_err(|err: RequestError| err.wrap(#field_name))?;
        }
    } else if is_vec {
        let mut parse_source = quote!();
        let mut parse_item = quote!();
        if let Some(regex) = field.regex.as_ref() {
            parse_source.extend(quote! {
                static REGEX: ::std::sync::OnceLock<::regex::Regex> = ::std::sync::OnceLock::new();
                let re = REGEX.get_or_init(|| ::regex::Regex::new(#regex).unwrap());
            });
            parse_item.extend(quote! {
                if !re.is_match(&target) {
                    return Err(RequestError::field_index(
                        #field_name,
                        i,
                        CommonError::InvalidStringFormat {
                            expected: #regex.into(),
                        },
                    ));
                }
            });
        } else if field.enumeration {
            parse_item.extend(quote! {
                let target = target.try_into()
                    .map_err(|_| RequestError::field_index(#field_name, i, CommonError::InvalidEnumValue))?;
            });
        } else if is_nested {
            parse_item.extend(quote! {
                let target = target.parse_into()
                    .map_err(|err: RequestError| err.wrap_index(#field_name, i))?;
            });
        }
        quote! {
            #parse_source
            let mut v = Vec::new();
            for (i, target) in target.into_iter().enumerate() {
                #parse_item
                v.push(target);
            }
            let target = v;
        }
    } else if let Some(map_ident) = map_ident.as_ref() {
        let mut parse_source = quote!();
        let mut parse_item = quote!();
        if let Some(regex) = field.regex.as_ref() {
            parse_source.extend(quote! {
                static REGEX: ::std::sync::OnceLock<::regex::Regex> = ::std::sync::OnceLock::new();
                let re = REGEX.get_or_init(|| ::regex::Regex::new(#regex).unwrap());
            });
            parse_item.extend(quote! {
                if !re.is_match(&target) {
                    return Err(RequestError::field(
                        #field_name,
                        CommonError::InvalidStringFormat {
                            expected: #regex.into(),
                        },
                    ));
                }
            });
        } else if field.enumeration {
            parse_item.extend(quote! {
                let target = target.try_into()
                    .map_err(|_| RequestError::field(#field_name, CommonError::InvalidEnumValue))?;
            });
        } else if is_nested {
            parse_item.extend(quote! {
                let target = target.parse_into()
                    .map_err(|err: RequestError| err.wrap(#field_name))?;
            });
        }
        // TODO: add map key to RequestError?
        quote! {
            #parse_source
            let mut m = #map_ident::new();
            for (k, target) in target.into_iter() {
                #parse_item
                m.insert(k, target);
            }
            let target = m;
        }
    } else if field.enumeration {
        quote! {
            if target == 0 {
                return Err(RequestError::field(
                    #field_name,
                    CommonError::RequiredFieldMissing,
                ));
            }
            let target = target.try_into()
                .map_err(|_| RequestError::field(#field_name, CommonError::InvalidEnumValue))?;
        }
    } else if field.oneof {
        let parse_source = if is_box || field.source_box {
            quote! {
                let target = *target;
            }
        } else {
            quote!()
        };
        quote! {
            #parse_source
            let target = target.parse_into()?;
        }
    } else if is_nested {
        let parse_source = if is_box || field.source_box {
            quote! {
                let target = *target;
            }
        } else {
            quote!()
        };
        quote! {
            #parse_source
            let target = target.parse_into()
                .map_err(|err: RequestError| err.wrap(#field_name))?;
        }
    } else if is_string {
        let mut parse_source = quote! {
            if target.is_empty() {
                return Err(RequestError::field(
                    #field_name,
                    CommonError::RequiredFieldMissing,
                ));
            }
        };
        if let Some(regex) = field.regex.as_ref() {
            parse_source.extend(quote! {{
                static REGEX: ::std::sync::OnceLock<::regex::Regex> = ::std::sync::OnceLock::new();
                let re = REGEX.get_or_init(|| ::regex::Regex::new(#regex).unwrap());
                if !re.is_match(&target) {
                    return Err(RequestError::field(
                        #field_name,
                        CommonError::InvalidStringFormat {
                            expected: #regex.into(),
                        },
                    ));
                }
            }});
        }
        parse_source
    } else if is_box || field.source_box {
        quote! {
            let target = *target;
        }
    } else {
        quote!()
    };

    if field.source_try_from.is_some() {
        parse_source.extend(quote! {
            let target = target.try_into()
                .map_err(|_| RequestError::field(#field_name, CommonError::FailedConvertValue))?;
        });
    }

    let mut parse = quote! {
        let target = source.#source_ident;
    };
    if field.wrapper {
        match field_type.to_token_stream().to_string().as_str() {
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize" => {
                parse.extend(quote! {
                    let target = target.value as #field_type;
                });
            }
            "String" | "bool" | "f32" | "f64" => {
                parse.extend(quote! {
                    let target = target.value;
                });
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    &field.ident,
                    "unsupported wrapper type",
                ));
            }
        }
    }

    let default_expr = if let Some(default) = field.default.as_ref() {
        quote! { #default }
    } else {
        quote! { Default::default() }
    };

    // Source field for nested messages is always wrapped in `Option`
    let source_option = field.source_option
        || is_option
        || (is_nested
            && (field.with.is_none() && field.parse_with.is_none())
            && !is_vec
            && map_ident.is_none()
            && !field.enumeration);

    if is_option {
        if source_option {
            if is_vec || is_string {
                parse.extend(quote! {
                    let target = if let Some(target) = target.filter(|target| !target.is_empty()) {
                        #parse_source
                        Some(target)
                    } else {
                        None
                    };
                });
            } else if field.enumeration {
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
            parse.extend(if is_vec || is_string {
                quote! {
                    let target = if target.is_empty() {
                        None
                    } else {
                        #parse_source
                        Some(target)
                    };
                }
            } else if !is_vec && field.enumeration {
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
        if source_option {
            if field.default.is_some() {
                parse.extend(quote! {
                    let target = target.unwrap_or_else(|| #default_expr);
                });
            } else {
                parse.extend(quote! {
                    let target = target.ok_or_else(|| {
                        RequestError::field(
                            #field_name,
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
        #target_ident: {
            #parse
            target
        },
    })
}

fn expand_parse_resource_field(field: &ParseField) -> syn::Result<TokenStream> {
    if field.source_option
        || field.enumeration
        || field.oneof
        || field.regex.is_some()
        || field.source_try_from.is_some()
    {
        return Err(syn::Error::new_spanned(
            &field.ident,
            "some of these options cannot be used alongside `resource`",
        ));
    }

    let target_ident = field.ident.as_ref().unwrap();
    let options = field.resource.as_ref().unwrap();

    let mut result = quote! {
        let mut result = ParsedResource::default();
    };

    if options.fields.name {
        result.extend(quote! {
            // if source.name.is_empty() {
            //     return Err(RequestError::field(
            //         "name",
            //         CommonError::RequiredFieldMissing,
            //     ));
            // }
            result.name = source.name.clone();
        });
    }
    if options.fields.create_time {
        result.extend(quote! {
            result.create_time = source.create_time
                .map(|create_time| create_time
                    .try_into()
                    .map_err(|_| RequestError::field(
                        "create_time",
                        CommonError::InvalidDateTime,
                    ))
                )
                .transpose()?;
        });
    }
    if options.fields.update_time {
        result.extend(quote! {
            result.update_time = source.update_time
                .map(|update_time| update_time
                    .try_into()
                    .map_err(|_| RequestError::field(
                        "update_time",
                        CommonError::InvalidDateTime,
                    ))
                )
                .transpose()?;
        });
    }
    if options.fields.delete_time {
        result.extend(quote! {
            result.delete_time = source.delete_time
                .map(|delete_time| delete_time
                    .try_into()
                    .map_err(|_| RequestError::field(
                        "delete_time",
                        CommonError::InvalidDateTime,
                    ))
                )
                .transpose()?;
        });
    }
    if options.fields.deleted {
        result.extend(quote! {
            result.deleted = source.deleted;
        });
    }
    if options.fields.etag {
        result.extend(quote! {
            result.etag = source.etag.clone().filter(|etag| !etag.is_empty());
        });
    }

    Ok(quote! {
        #target_ident: {
            #result
            result
        },
    })
}
