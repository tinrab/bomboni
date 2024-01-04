use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::parse::{ParseField, ParseOptions, QueryOptions};
use crate::utility::{get_proto_type_info, ProtoTypeInfo};

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> TokenStream {
    let mut write_fields = quote!();

    for field in fields {
        if field.skip || field.derive.is_some() {
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

        if field.resource.is_some() {
            write_fields.extend(expand_write_resource(field));
        } else {
            write_fields.extend(expand_write_field(options, field));
        }
    }

    if let Some(query_options) = options
        .list_query
        .as_ref()
        .or(options.search_query.as_ref())
    {
        write_fields.extend(expand_query_resource(
            query_options,
            options.search_query.is_some(),
        ));
    }

    let source = &options.source;
    let ident = &options.ident;
    let (impl_generics, type_generics, where_clause) = options.generics.split_for_impl();

    quote! {
        impl #impl_generics From<#ident #type_generics> for #source #where_clause {
            #[allow(clippy::needless_update)]
            fn from(value: #ident #type_generics) -> Self {
                let mut source: #source = Default::default();
                #write_fields
                source
            }
        }
    }
}

fn expand_write_field(options: &ParseOptions, field: &ParseField) -> TokenStream {
    let field_type = &field.ty;
    let ProtoTypeInfo {
        is_option,
        is_nested,
        is_vec,
        is_box,
        is_generic,
        map_ident,
        ..
    } = get_proto_type_info(options, field_type);

    let mut write_target = if field.keep {
        if is_box {
            quote! {
                let source = *source;
            }
        } else {
            quote!()
        }
    } else if field.with.is_some() || field.write_with.is_some() {
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
        } else if is_nested || is_generic {
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
        } else if is_nested || is_generic {
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
    } else if field.oneof || is_nested || is_generic {
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

    if let Some(source_try_from) = field.source_try_from.as_ref() {
        let field_ident = field.ident.as_ref().unwrap();
        let err_literal = format!("failed to convert `{field_ident}` to `{source_try_from}`");
        write_target.extend(quote! {
            let source: #source_try_from = source.try_into()
                .expect(#err_literal);
        });
    }

    let target_ident = field.ident.as_ref().unwrap();
    let mut write = quote! {
        let source = value.#target_ident;
    };

    let source_option = field.source_option
        || is_option
        || (is_nested
            && !is_generic
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
                let source = if let Some(source) = source {
                    #write_target
                    source
                } else {
                    Default::default()
                };
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

    if field.wrapper {
        write.extend(quote! {
            let source: #field_type = source;
            let source = source.into();
        });
    }

    let source = if let Some(source_name) = field.source_name.as_ref() {
        if source_name.contains('.') {
            let parts: Vec<_> = source_name.split('.').collect();
            let mut inject = quote!();
            for (i, part) in parts.iter().enumerate() {
                let part_ident = Ident::from_string(part).unwrap();
                inject.extend(if i < parts.len() - 1 {
                    quote! {
                        .#part_ident
                        .get_or_insert(Default::default())
                    }
                } else {
                    quote! {
                        .#part_ident
                    }
                });
            }
            quote! {
                source #inject
            }
        } else {
            let source_ident = Ident::from_string(source_name).unwrap();
            quote! {
                source.#source_ident
            }
        }
    } else {
        let source_ident = field.ident.clone().unwrap();
        quote! {
            source.#source_ident
        }
    };
    quote! {
        #source = {
            #write
            source
        };
    }
}

fn expand_write_resource(field: &ParseField) -> TokenStream {
    let options = field.resource.as_ref().unwrap();
    let ident = field.ident.as_ref().unwrap();

    let mut result = quote!();

    if options.fields.name {
        result.extend(quote! {
            source.name = value.#ident.name;
        });
    }
    if options.fields.create_time {
        result.extend(quote! {
            source.create_time = value.#ident.create_time.map(Into::into);
        });
    }
    if options.fields.update_time {
        result.extend(quote! {
            source.update_time = value.#ident.update_time.map(Into::into);
        });
    }
    if options.fields.delete_time {
        result.extend(quote! {
            source.delete_time = value.#ident.delete_time.map(Into::into);
        });
    }
    if options.fields.deleted {
        result.extend(quote! {
            source.deleted = value.#ident.deleted;
        });
    }
    if options.fields.etag {
        result.extend(quote! {
            source.etag = value.#ident.etag;
        });
    }

    result
}

fn expand_query_resource(options: &QueryOptions, is_search: bool) -> TokenStream {
    let ident = &options.field;
    let mut result = quote!();

    if options.query.parse && is_search {
        let source_name = &options.query.source_name;
        result.extend(quote! {
            source.#source_name = value.#ident.query;
        });
    }
    if options.page_size.parse {
        let source_name = &options.page_size.source_name;
        result.extend(quote! {
            source.#source_name = Some(value.#ident.page_size.try_into().unwrap());
        });
    }
    if options.page_token.parse {
        let source_name = &options.page_token.source_name;
        result.extend(quote! {
            source.#source_name = value.#ident.page_token.map(|page_token| page_token.to_string());
        });
    }
    if options.filter.parse {
        let source_name = &options.filter.source_name;
        result.extend(quote! {
            source.#source_name = if value.#ident.filter.is_empty() {
                None
            } else {
                Some(value.#ident.filter.to_string())
            };
        });
    }
    if options.ordering.parse {
        let source_name = &options.ordering.source_name;
        result.extend(quote! {
            source.#source_name = if value.#ident.ordering.is_empty() {
                None
            } else {
                Some(value.#ident.ordering.to_string())
            };
        });
    }

    result
}
