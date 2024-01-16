use bomboni_core::syn::type_is_phantom;
use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::parse::{DeriveOptions, ParseField, ParseOptions, QueryOptions};
use crate::utility::{get_proto_type_info, get_query_field_token_type, ProtoTypeInfo};

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> syn::Result<TokenStream> {
    if options.list_query.is_some() && options.search_query.is_some() {
        return Err(syn::Error::new_spanned(
            &options.ident,
            "list and search query cannot be used together",
        ));
    }

    let mut parse_fields = quote!();
    // Set default for skipped fields
    let mut skipped_fields = quote!();

    // Parse fields in order, starting with derived ones.
    // This is needed because derived fields may depend on other fields, and we want to avoid unnecessary cloning.
    for field in fields {
        if field.derive.is_some() {
            parse_fields.extend(expand_parse_field(options, field)?);
        }
    }

    // Parse resource fields
    for field in fields {
        if field.resource.is_some() {
            parse_fields.extend(expand_parse_resource_field(field)?);
        }
    }

    // Parse nested fields
    for field in fields {
        if field.derive.is_some() || field.resource.is_some() {
            continue;
        }

        if matches!(field.source_name.as_ref(), Some(name) if name.contains('.')) {
            parse_fields.extend(expand_parse_field(options, field)?);
        }
    }

    for field in fields {
        if field.derive.is_some()
            || field.resource.is_some()
            || matches!(field.source_name.as_ref(), Some(name) if name.contains('.'))
        {
            continue;
        }

        // Skip query fields
        if let Some(list_query) = options.list_query.as_ref() {
            if &list_query.field == field.ident.as_ref().unwrap() {
                continue;
            }
        } else if let Some(search_query) = options.search_query.as_ref() {
            if &search_query.field == field.ident.as_ref().unwrap() {
                continue;
            }
        }

        let field_ident = field.ident.as_ref().unwrap();
        if field.skip || type_is_phantom(&field.ty) {
            skipped_fields.extend(quote! {
                #field_ident: Default::default(),
            });
            continue;
        }

        parse_fields.extend(expand_parse_field(options, field)?);
    }

    let mut query_token_type = quote!();
    let mut parse = if let Some(query_options) = options
        .list_query
        .as_ref()
        .or(options.search_query.as_ref())
    {
        let query_field_ident = &query_options.field;
        let parse_query = expand_parse_query(query_options, options.search_query.is_some());

        let query_field = fields
            .iter()
            .find(|field| field.ident.as_ref().unwrap() == query_field_ident)
            .unwrap();
        query_token_type = if let Some(token_type) = get_query_field_token_type(&query_field.ty) {
            quote! {
                <PageToken = #token_type>
            }
        } else {
            quote! {
                <PageToken = FilterPageToken>
            }
        };

        quote! {
            Ok(Self {
                #query_field_ident: {
                    #parse_query
                    query
                },
                #parse_fields
                #skipped_fields
            })
        }
    } else {
        quote! {
            Ok(Self {
                #parse_fields
                #skipped_fields
            })
        }
    };

    let source = &options.source;
    let ident = &options.ident;
    let (impl_generics, type_generics, where_clause) = options.generics.split_for_impl();

    if let Some(request_options) = options.request.as_ref() {
        let request_name = if let Some(name) = request_options.name.as_ref() {
            quote! { #name }
        } else {
            quote! { #source::NAME }
        };
        parse = quote! {
            (|| { #parse })().map_err(|err: RequestError| err.wrap_request(#request_name))
        };
    }

    Ok(if options.search_query.is_some() {
        quote! {
            impl #ident #type_generics #where_clause {
                #[allow(clippy::ignored_unit_patterns)]
                pub fn parse_search_query<P: PageTokenBuilder #query_token_type >(
                    source: #source,
                    query_builder: &SearchQueryBuilder<P>
                ) -> Result<Self, RequestError> {
                    #parse
                }
            }
        }
    } else if options.list_query.is_some() {
        quote! {
            impl #ident #type_generics #where_clause {
                #[allow(clippy::ignored_unit_patterns)]
                pub fn parse_list_query<P: PageTokenBuilder #query_token_type >(
                    source: #source,
                    query_builder: &ListQueryBuilder<P>
                ) -> Result<Self, RequestError> {
                    #parse
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics RequestParse<#source> for #ident #type_generics #where_clause {
                #[allow(clippy::ignored_unit_patterns)]
                fn parse(source: #source) -> RequestResult<Self> {
                    #parse
                }
            }
        }
    })
}

fn expand_parse_field(options: &ParseOptions, field: &ParseField) -> syn::Result<TokenStream> {
    let target_ident = field.ident.as_ref().unwrap();

    if let Some(DeriveOptions { func, source_field }) = field.derive.as_ref() {
        return Ok(if let Some(source_field) = source_field.as_ref() {
            let source_field_name = &source_field
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join(".");
            quote! {
                #target_ident: { #func(&source.#source_field, #source_field_name)? },
            }
        } else {
            quote! {
                #target_ident: { #func(&source)? },
            }
        });
    }

    let field_type = &field.ty;
    let ProtoTypeInfo {
        is_option,
        is_nested,
        is_string,
        is_box,
        is_vec,
        is_generic,
        map_ident,
        ..
    } = get_proto_type_info(options, field_type);

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
    let custom_parse = field.with.is_some() || field.parse_with.is_some();

    let mut parse_source = if field.keep {
        if is_box || field.source_box {
            quote! {
                let target = *target;
            }
        } else {
            quote!()
        }
    } else if custom_parse {
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
        } else if is_nested || is_generic {
            parse_item.extend(quote! {
                let target = target.parse_into()
                    .map_err(|err: RequestError| err.wrap_field_index(#field_name, i))?;
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
                    return Err(RequestError::field_key(
                        #field_name,
                        &key,
                        CommonError::InvalidStringFormat {
                            expected: #regex.into(),
                        },
                    ));
                }
            });
        } else if field.enumeration {
            parse_item.extend(quote! {
                let target = target.try_into()
                    .map_err(|_| RequestError::field_key(#field_name, &key, CommonError::InvalidEnumValue))?;
            });
        } else if is_nested || is_generic {
            parse_item.extend(quote! {
                let target = target.parse_into()
                    .map_err(|err: RequestError| err.wrap_field_key(#field_name, &key))?;
            });
        }
        quote! {
            #parse_source
            let mut m = #map_ident::new();
            for (key, target) in target.into_iter() {
                #parse_item
                m.insert(key, target);
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
    } else if is_nested || is_generic {
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

    let mut parse = expand_extract_source_field(field);

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

    // Source field for nested messages is always wrapped in `Option`
    let source_option = field.source_option
        || is_option
        || (is_nested
            && !is_generic
            && (field.with.is_none() && field.parse_with.is_none())
            && !is_vec
            && map_ident.is_none()
            && !field.enumeration);

    if is_option {
        if source_option {
            parse.extend(if (is_vec || is_string) && !custom_parse {
                quote! {
                    let target = if let Some(target) = target.filter(|target| !target.is_empty()) {
                        #parse_source
                        Some(target)
                    } else {
                        None
                    };
                }
            } else if field.enumeration && !custom_parse {
                quote! {
                    let target = if let Some(target) = target.filter(|e| *e != 0) {
                        #parse_source
                        Some(target)
                    } else {
                        None
                    };
                }
            } else {
                quote! {
                    let target = if let Some(target) = target {
                        #parse_source
                        Some(target)
                    } else {
                        None
                    };
                }
            });
        } else {
            parse.extend(if (is_vec || is_string) && !custom_parse {
                quote! {
                    let target = if target.is_empty() {
                        None
                    } else {
                        #parse_source
                        Some(target)
                    };
                }
            } else if (!is_vec && field.enumeration) && !custom_parse {
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
        parse.extend(if source_option {
            if let Some(default) = field.default.as_ref() {
                quote! {
                    let target = if let Some(target) = target {
                        #parse_source
                        target
                    } else {
                        #default
                    };
                }
            } else {
                quote! {
                    let target = target.ok_or_else(|| {
                        RequestError::field(
                            #field_name,
                            CommonError::RequiredFieldMissing,
                        )
                    })?;
                    #parse_source
                }
            }
        } else {
            parse_source
        });
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
        || field.with.is_some()
        || field.parse_with.is_some()
        || field.write_with.is_some()
        || field.keep
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

fn expand_parse_query(options: &QueryOptions, is_search: bool) -> TokenStream {
    let mut parse = quote! {
        let page_size: Option<i32> = None;
        let page_token: Option<&str> = None;
        let filter: Option<&str> = None;
        let ordering: Option<&str> = None;
    };
    if options.query.parse && is_search {
        let query_source_name = &options.query.source_name;
        parse.extend(quote! {
            let query_string = &source.#query_source_name;
        });
    }
    if options.page_size.parse {
        let source_name = &options.page_size.source_name;
        parse.extend(quote! {
            let page_size = source.#source_name.map(|i| i as i32);
        });
    }
    if options.page_token.parse {
        let source_name = &options.page_token.source_name;
        parse.extend(quote! {
            let page_token = source.#source_name.as_ref().map(|s| s.as_str());
        });
    }
    if options.filter.parse {
        let source_name = &options.filter.source_name;
        parse.extend(quote! {
            let filter = source.#source_name.as_ref().map(|s| s.as_str());
        });
    }
    if options.ordering.parse {
        let source_name = &options.ordering.source_name;
        parse.extend(quote! {
            let ordering = source.#source_name.as_ref().map(|s| s.as_str());
        });
    }

    if is_search {
        quote! {
            #parse
            let query = query_builder.build(query_string, page_size, page_token, filter, ordering)?;
        }
    } else {
        quote! {
            #parse
            let query = query_builder.build(page_size, page_token, filter, ordering)?;
        }
    }
}

fn expand_extract_source_field(field: &ParseField) -> TokenStream {
    if let Some(source_name) = field.source_name.as_ref() {
        if source_name.contains('.') {
            let parts = source_name.split('.').collect::<Vec<_>>();

            let mut extract = quote!();
            for (i, part) in parts.iter().enumerate() {
                let part_ident = Ident::from_string(part).unwrap();
                let part_literal = &parts
                    .iter()
                    .take(i + 1)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(".");

                extract.extend(if i < parts.len() - 1 {
                    quote! {
                        .#part_ident.ok_or_else(|| {
                            RequestError::field(
                                #part_literal,
                                CommonError::RequiredFieldMissing,
                            )
                        })?
                    }
                } else {
                    quote! {
                        .#part_ident
                    }
                });
            }

            // Intentionally clone source on each parse.
            // Could be optimized in the future.
            quote! {
                let target = source.clone() #extract;
            }
        } else {
            let source_ident = Ident::from_string(source_name).unwrap();
            quote! {
                let target = source.#source_ident;
            }
        }
    } else {
        let source_ident = field.ident.clone().unwrap();
        quote! {
            let target = source.#source_ident;
        }
    }
}
