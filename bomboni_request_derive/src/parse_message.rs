use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::parse::{ParseField, ParseOptions};
use crate::utility::check_proto_type;

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

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();

        if field.skip {
            skipped_fields.extend(quote! {
                #field_ident: Default::default(),
            });
            continue;
        }

        if field.resource.is_some() {
            parse_fields.extend(expand_parse_resource_field(field)?);
        } else {
            parse_fields.extend(expand_parse_field(field)?);
        };
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
    let target_ident = field.ident.as_ref().unwrap();

    if let Some(derive) = field.derive.as_ref() {
        return Ok(quote! {
            #target_ident: { #derive(&source)? },
        });
    }

    let (is_option, is_nested, is_string) = check_proto_type(&field.ty);

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

    let mut result = quote! {
        let target = source.#source_ident;
    };

    let default_expr = if let Some(default) = field.default.as_ref() {
        quote! { #default }
    } else {
        quote! { Default::default() }
    };

    if field.with.is_some() || field.parse_with.is_some() {
        let parse_with = if let Some(with) = field.with.as_ref() {
            quote! {
                #with::parse
            }
        } else {
            field.parse_with.as_ref().unwrap().to_token_stream()
        };
        if is_option {
            if field.source_option {
                result.extend(quote! {
                    let target = if let Some(target) = target {
                        Some(
                            #parse_with(target)
                                .map_err(|err: RequestError| err.wrap(#field_name))?
                        )
                    } else {
                        None
                    };
                });
            } else {
                result.extend(quote! {
                    let target = Some(
                        #parse_with(target)
                            .map_err(|err: RequestError| err.wrap(#field_name))?
                    );
                });
            }
        } else {
            if field.source_option {
                result.extend(quote! {
                    let target = target.unwrap_or_else(|| #default_expr);
                });
            }
            result.extend(quote! {
                let target = #parse_with(target)
                    .map_err(|err: RequestError| err.wrap(#field_name))?;
            });
        }
    } else if field.enumeration {
        if field.source_option {
            result.extend(quote! {
                let target = target.unwrap_or_else(|| #default_expr);
            });
        }
        // Assume that missing enum value is represented by `0`
        if is_option {
            result.extend(quote! {
                let target = if target == 0 {
                    None
                } else {
                    Some(
                        target.try_into()
                        .map_err(|_| RequestError::field(#field_name, CommonError::InvalidEnumValue))?
                    )
                };
            });
        } else if field.default.is_some() {
            result.extend(quote! {
                let target = target.try_into()
                    .map_err(|_| RequestError::field(#field_name, CommonError::InvalidEnumValue))?;
            });
        } else {
            result.extend(quote! {
                if target == 0 {
                    return Err(RequestError::field(
                        #field_name,
                        CommonError::RequiredFieldMissing,
                    ));
                }
                let target = target
                    .try_into()
                    .map_err(|_| RequestError::field(#field_name, CommonError::InvalidEnumValue))?;
            });
        }
    } else if field.oneof {
        if is_option {
            result.extend(quote! {
                let target = if let Some(target) = target {
                    let variant_name = target.get_variant_name();
                    Some(
                        target.parse_into()
                            .map_err(|err: RequestError| err.wrap(variant_name))?
                    )
                } else {
                    None
                };
            });
        } else if field.default.is_some() {
            result.extend(quote! {
                let target = target.unwrap_or_else(|| #default_expr);
                let variant_name = target.get_variant_name();
                let target = target.parse_into()
                    .map_err(|err: RequestError| err.wrap(variant_name))?;
            });
        } else {
            result.extend(quote! {
                let target = target
                .ok_or_else(|| {
                    RequestError::field(
                        #field_name,
                        CommonError::RequiredFieldMissing,
                    )
                })?;
                let variant_name = target.get_variant_name();
                let target = target.parse_into()
                    .map_err(|err: RequestError| err.wrap(variant_name))?;
            });
        }
    } else if is_nested {
        // Source field for nested messages is always wrapped in `Option`
        if is_option {
            result.extend(quote! {
                let target = if let Some(target) = target {
                    Some(
                        target.parse_into()
                            .map_err(|err: RequestError| err.wrap(#field_name))?
                    )
                } else {
                    None
                };
            });
        } else if field.default.is_some() {
            result.extend(quote! {
                let target = target
                .unwrap_or_else(|| #default_expr)
                .parse_into()
                .map_err(|err: RequestError| err.wrap(#field_name))?;
            });
        } else {
            result.extend(quote! {
                let target = target
                .ok_or_else(|| {
                    RequestError::field(
                        #field_name,
                        CommonError::RequiredFieldMissing,
                    )
                })?
                .parse_into()
                .map_err(|err: RequestError| err.wrap(#field_name))?;
            });
        }
    } else if is_string {
        if field.source_option {
            result.extend(quote! {
                let target = target.unwrap_or_else(|| #default_expr);
            });
        }
        if let Some(regex) = field.regex.as_ref() {
            result.extend(quote! {{
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
        // Treat empty string as `None`
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
                    return Err(RequestError::field(
                        #field_name,
                        CommonError::RequiredFieldMissing,
                    ));
                }
            });
        }
    } else {
        // Parse primitive
        if field.source_option {
            result.extend(quote! {
                let target = target.unwrap_or_else(|| #default_expr);
            });
        }
        if is_option {
            result.extend(quote! {
                let target = Some(target);
            });
        }
    }

    Ok(quote! {
        #target_ident: {
            #result
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
            result.name = source.name;
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
            result.etag = source.etag.filter(|etag| !etag.is_empty());
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
    let source_ident = field.ident.as_ref().unwrap();

    let (is_option, is_nested, is_string) = check_proto_type(&field.ty);

    let mut result = quote! {
        let target = value.#source_ident;
    };

    if let Some(with) = field.with.as_ref() {
        result.extend(quote! {
            let target = #with::write(target);
        });
    } else if let Some(write_with) = field.write_with.as_ref() {
        result.extend(quote! {
            let target = #write_with(target);
        });
    } else if field.enumeration {
        result.extend(quote! {
            let target = target as i32;
        });
    } else if field.oneof {
        if !is_option {
            result.extend(quote! {
                let target = Some(target.into());
            });
        }
    } else if is_nested {
        if is_option {
            result.extend(quote! {
                let target = target.map(Into::into);
            });
        } else {
            result.extend(quote! {
                let target = Some(target.into());
            });
        }
    } else if is_string {
        if !is_option && field.source_option {
            result.extend(quote! {
                let target = Some(target);
            });
        }
    } else if !is_option && field.source_option {
        result.extend(quote! {
            let target = Some(target);
        });
    }

    let source_ident = if let Some(name) = field.source_name.as_ref() {
        Ident::from_string(name).unwrap()
    } else {
        field.ident.clone().unwrap()
    };

    quote! {
        #source_ident: {
            #result
            target
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
