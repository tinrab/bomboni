use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::parse::{ParseField, ParseOptions};
use crate::utility::{get_proto_type_info, ProtoTypeInfo};

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> syn::Result<TokenStream> {
    let parse = expand_parse(options, fields)?;
    let write = if options.write {
        expand_write(options, fields)
    } else {
        quote!()
    };
    Ok(quote! {
        #parse
        #write
    })
}

fn expand_parse(options: &ParseOptions, fields: &[ParseField]) -> syn::Result<TokenStream> {
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

    if let Some(derive) = field.derive.as_ref() {
        return Ok(quote! {
            #target_ident: { #derive(&source)? },
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

    let ProtoTypeInfo {
        is_option,
        is_nested,
        is_string,
        is_box,
        is_vec,
        map_ident,
        ..
    } = get_proto_type_info(&field.ty);

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
            "regex can only be used with string fields",
        ));
    }
    if field.with.is_some() && (field.parse_with.is_some() || field.write_with.is_some()) {
        return Err(syn::Error::new_spanned(
            &field.ident,
            "custom `parse_with` and `write_with` functions cannot be used alongside `with`",
        ));
    }

    let parse_source = if field.with.is_some() || field.parse_with.is_some() {
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

    let mut parse = quote! {
        let target = source.#source_ident;
    };

    let default_expr = if let Some(default) = field.default.as_ref() {
        quote! { #default }
    } else {
        quote! { Default::default() }
    };

    // Source field for nested messages is always wrapped in `Option`
    let source_option = field.source_option
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
    if field.source_option || field.enumeration || field.oneof || field.regex.is_some() {
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

fn expand_write(options: &ParseOptions, fields: &[ParseField]) -> TokenStream {
    let source = &options.source;
    let ident = &options.ident;

    let mut write_fields = quote!();

    for field in fields {
        if field.skip || field.derive.is_some() {
            continue;
        }

        if field.resource.is_some() {
            write_fields.extend(expand_write_resource(field));
        } else {
            write_fields.extend(expand_write_field(field));
        }
    }

    quote! {
        impl From<#ident> for #source {
            #[allow(clippy::needless_update)]
            fn from(value: #ident) -> Self {
                #source {
                    #write_fields
                    ..Default::default()
                }
            }
        }
    }
}

fn expand_write_field(field: &ParseField) -> TokenStream {
    let target_ident = field.ident.as_ref().unwrap();
    let source_ident = if let Some(name) = field.source_name.as_ref() {
        Ident::from_string(name).unwrap()
    } else {
        field.ident.clone().unwrap()
    };

    let ProtoTypeInfo {
        is_option,
        is_nested,
        is_vec,
        is_box,
        map_ident,
        ..
    } = get_proto_type_info(&field.ty);

    let write_target = if field.with.is_some() || field.write_with.is_some() {
        let write_with = if let Some(with) = field.with.as_ref() {
            quote! {
                #with::write
            }
        } else {
            field.write_with.as_ref().unwrap().to_token_stream()
        };
        quote! {
            let source = #write_with(source);
        }
    } else if is_vec {
        let mut write_item = quote!();
        if field.enumeration {
            write_item.extend(quote! {
                let source = source as i32;
            });
        } else if is_nested {
            write_item.extend(quote! {
                let source = source.into();
            });
        }
        quote! {
            let mut v = Vec::new();
            for source in source.into_iter() {
                #write_item
                v.push(source);
            }
            let source = v;
        }
    } else if let Some(map_ident) = map_ident.as_ref() {
        let mut write_item = quote!();
        if field.enumeration {
            write_item.extend(quote! {
                let source = source as i32;
            });
        } else if is_nested {
            write_item.extend(quote! {
                let source = source.into();
            });
        }
        quote! {
            let mut m = #map_ident::new();
            for (k, source) in source.into_iter() {
                #write_item
                m.insert(k, source);
            }
            let source = m;
        }
    } else if field.enumeration {
        quote! {
            let source = source as i32;
        }
    } else if field.oneof || is_nested {
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
        let source = value.#target_ident;
    };

    let default_expr = if let Some(default) = field.default.as_ref() {
        quote! { #default }
    } else {
        quote! { Default::default() }
    };

    let source_option = field.source_option
        || (is_nested
            && (field.with.is_none() && field.parse_with.is_none())
            && !is_vec
            && map_ident.is_none()
            && !field.enumeration);

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

    if is_box || field.source_box {
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

    quote! {
        #source_ident: {
            #write
            source
        },
    }
}

fn expand_write_resource(field: &ParseField) -> TokenStream {
    let options = field.resource.as_ref().unwrap();
    let ident = field.ident.as_ref().unwrap();

    let mut result = quote!();

    if options.fields.name {
        result.extend(quote! {
            name: value.#ident.name,
        });
    }
    if options.fields.create_time {
        result.extend(quote! {
            create_time: value.#ident.create_time.map(Into::into),
        });
    }
    if options.fields.update_time {
        result.extend(quote! {
            update_time: value.#ident.update_time.map(Into::into),
        });
    }
    if options.fields.delete_time {
        result.extend(quote! {
            delete_time: value.#ident.delete_time.map(Into::into),
        });
    }
    if options.fields.deleted {
        result.extend(quote! {
            deleted: value.#ident.deleted,
        });
    }
    if options.fields.etag {
        result.extend(quote! {
            etag: value.#ident.etag,
        });
    }

    result
}
