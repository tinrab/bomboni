use bomboni_core::syn::type_is_phantom;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::collections::BTreeSet;

use crate::parse::{
    message::utility::{get_field_clone_set, get_field_extract, get_query_field_token_type},
    options::{FieldExtractStep, ParseDerive, ParseField, ParseOptions, ParseQuery, ParseResource},
    parse_utility::{expand_field_extract, expand_parse_field_type, make_field_error_path},
};

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> syn::Result<TokenStream> {
    let mut parse_fields = quote!();

    let field_clone_set = get_field_clone_set(fields)?;

    // Parse fields in order, starting with derived ones.
    // This is needed because derived fields may depend on other fields, and we want to avoid unnecessary cloning.
    for field in fields
        .iter()
        .filter(|field| !field.options.skip && field.options.derive.is_some())
    {
        parse_fields.extend(expand_parse_field(field, &field_clone_set)?);
    }

    // Parse resource fields
    for field in fields.iter().filter(|field| !field.options.skip) {
        if let Some(resource) = &field.resource {
            parse_fields.extend(expand_parse_resource_field(resource, field));
        }
    }

    // Parse field mask fields
    for field in fields.iter().filter(|field| !field.options.skip) {
        if let Some(field_mask) = &field.options.field_mask {
            parse_fields.extend(expand_parse_field_mask(field, field_mask)?);
        }
    }

    // Parse regular fields
    for field in fields.iter().filter(|field| {
        !field.options.skip
            && field.options.derive.is_none()
            && field.resource.is_none()
            && field.list_query.is_none()
            && field.search_query.is_none()
            && field.options.field_mask.is_none()
            && !type_is_phantom(&field.ty)
    }) {
        parse_fields.extend(expand_parse_field(field, &field_clone_set)?);
    }

    // Set default for skipped fields
    let mut skipped_fields = quote!();
    for field in fields
        .iter()
        .filter(|field| field.options.skip || type_is_phantom(&field.ty))
    {
        let target_ident = field.ident.as_ref().ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "field missing ident")
        })?;
        skipped_fields.extend(quote! {
            #target_ident: Default::default(),
        });
    }

    // Parse query fields
    let mut query_token_type = quote!();
    let mut list_or_search = None;
    let mut parse_impl = if let Some((field, query)) = fields.iter().find_map(|field| {
        if field.options.skip {
            None
        } else {
            field
                .list_query
                .as_ref()
                .or(field.search_query.as_ref())
                .map(|query| (field, query))
        }
    }) {
        list_or_search = Some(field.list_query.is_some());
        let target_ident = field.ident.as_ref().ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "field missing ident")
        })?;
        let parse_query_impl = expand_parse_query(query, field.search_query.is_some());
        query_token_type = get_query_field_token_type(&field.ty).map_or_else(
            || {
                quote! {
                    <PageToken = FilterPageToken>
                }
            },
            |token_type| {
                quote! {
                    <PageToken = #token_type>
                }
            },
        );
        quote! {
            Ok(Self {
                #target_ident: {
                    #parse_query_impl
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
        let request_name = request_options
            .name
            .as_ref()
            .map_or_else(|| quote! { #source::NAME }, |name| quote! { #name });
        parse_impl = quote! {
            (|| { #parse_impl })().map_err(|err: RequestError| err.wrap_request(#request_name))
        };
    }

    Ok(
        if list_or_search.is_some_and(|list_or_search| list_or_search) {
            quote! {
                #[automatically_derived]
                impl #ident #type_generics #where_clause {
                    pub fn parse_list_query<P: PageTokenBuilder #query_token_type >(
                        source: #source,
                        query_builder: &ListQueryBuilder<P>
                    ) -> Result<Self, RequestError> {
                        #parse_impl
                    }
                }
            }
        } else if list_or_search.is_some_and(|list_or_search| !list_or_search) {
            quote! {
                #[automatically_derived]
                impl #ident #type_generics #where_clause {
                    pub fn parse_search_query<P: PageTokenBuilder #query_token_type >(
                        source: #source,
                        query_builder: &SearchQueryBuilder<P>
                    ) -> Result<Self, RequestError> {
                        #parse_impl
                    }
                }
            }
        } else {
            quote! {
                #[automatically_derived]
                impl #impl_generics RequestParse<#source> for #ident #type_generics #where_clause {
                    fn parse(source: #source) -> RequestResult<Self> {
                        #parse_impl
                    }
                }
            }
        },
    )
}

fn expand_parse_field(
    field: &ParseField,
    field_clone_set: &BTreeSet<String>,
) -> syn::Result<TokenStream> {
    let target_ident = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "field missing ident"))?;
    let extract = get_field_extract(field)?;

    if let Some(ParseDerive {
        parse,
        module,
        source_borrow,
        ..
    }) = field.options.derive.as_ref()
    {
        let parse_impl = parse
            .as_ref()
            .map(ToTokens::to_token_stream)
            .or_else(|| module.as_ref().map(|module| quote!(#module::parse)))
            .unwrap();

        if field.options.source.is_some() || field.options.extract.is_some() {
            let (extract_impl, _get_impl, field_path) =
                expand_field_extract(&extract, field_clone_set, None, None, *source_borrow);
            let field_error_path = make_field_error_path(&field_path, None);

            return Ok(quote! {
                #target_ident: {
                    #extract_impl
                    #parse_impl(target)
                        .map_err(|err: RequestError| err.wrap_path(#field_error_path))?
                },
            });
        }

        return Ok(quote! {
            #target_ident: { #parse_impl(&source)? },
        });
    }

    if field.options.keep {
        if extract.steps.len() != 1 {
            return Err(syn::Error::new_spanned(
                target_ident,
                "invalid field extract for `keep`",
            ));
        }
        if let FieldExtractStep::Field(source_field) = &extract.steps[0] {
            let source_ident = format_ident!("{}", source_field);
            return Ok(quote! {
                #target_ident: source.#source_ident.clone(),
            });
        }
        return Err(syn::Error::new_spanned(
            target_ident,
            "invalid field extract for `keep`",
        ));
    }

    let field_type_info = field.type_info.as_ref().ok_or_else(|| {
        syn::Error::new(proc_macro2::Span::call_site(), "field missing type info")
    })?;
    let (extract_impl, get_impl, field_path) = expand_field_extract(
        &extract,
        field_clone_set,
        Some(field_type_info),
        None,
        false,
    );

    let field_error_path = make_field_error_path(&field_path, None);
    let parse_field_impl =
        expand_parse_field_type(&field.options, field_type_info, field_error_path, get_impl);

    Ok(quote! {
        #target_ident: {
            #extract_impl
            #parse_field_impl
            target
        },
    })
}

fn expand_parse_resource_field(options: &ParseResource, field: &ParseField) -> TokenStream {
    let mut parse_impl = quote! {
        let mut result = ParsedResource::default();
    };

    if options.name.parse {
        let source_ident = &options.name.source;
        parse_impl.extend(quote! {
            if source.#source_ident.is_empty() {
                return Err(RequestError::field(
                    "name",
                    CommonError::RequiredFieldMissing,
                ));
            }
            // TODO: avoid clone
            result.name = source.#source_ident.clone();
        });
    }
    if options.create_time.parse {
        let source_ident = &options.create_time.source;
        parse_impl.extend(quote! {
            result.create_time = source.#source_ident
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
    if options.update_time.parse {
        let source_ident = &options.update_time.source;
        parse_impl.extend(quote! {
            result.update_time = source.#source_ident
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
    if options.delete_time.parse {
        let source_ident = &options.delete_time.source;
        parse_impl.extend(quote! {
            result.delete_time = source.#source_ident
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
    if options.deleted.parse {
        let source_ident = &options.deleted.source;
        parse_impl.extend(quote! {
            result.deleted = source.#source_ident;
        });
    }
    if options.etag.parse {
        let source_ident = &options.etag.source;
        parse_impl.extend(quote! {
            result.etag = source.#source_ident.clone().filter(|etag| !etag.is_empty());
        });
    }

    let target_ident = field.ident.as_ref().unwrap();
    quote! {
        #target_ident: {
            #parse_impl
            result
        },
    }
}

fn expand_parse_query(options: &ParseQuery, search: bool) -> TokenStream {
    let mut parse_impl = quote! {
        let page_size: Option<i32> = None;
        let page_token: Option<&str> = None;
        let filter: Option<&str> = None;
        let order_by: Option<&str> = None;
    };
    if options.query.parse && search {
        let source_ident = &options.query.source;
        parse_impl.extend(quote! {
            let query_string = &source.#source_ident;
        });
    }
    if options.page_size.parse {
        let source_ident = &options.page_size.source;
        parse_impl.extend(quote! {
            let page_size = source.#source_ident.map(|i| i as i32);
        });
    }
    if options.page_token.parse {
        let source_ident = &options.page_token.source;
        parse_impl.extend(quote! {
            let page_token = source.#source_ident.as_ref().map(|s| s.as_str());
        });
    }
    if options.filter.parse {
        let source_ident = &options.filter.source;
        parse_impl.extend(quote! {
            let filter = source.#source_ident.as_ref().map(|s| s.as_str());
        });
    }
    if options.order_by.parse {
        let source_ident = &options.order_by.source;
        parse_impl.extend(quote! {
            let order_by = source.#source_ident.as_ref().map(|s| s.as_str());
        });
    }

    if search {
        quote! {
            #parse_impl
            let query = query_builder.build(query_string, page_size, page_token, filter, order_by)?;
        }
    } else {
        quote! {
            #parse_impl
            let query = query_builder.build(page_size, page_token, filter, order_by)?;
        }
    }
}

fn expand_parse_field_mask(
    field: &ParseField,
    field_mask: &crate::parse::options::ParseFieldMask,
) -> syn::Result<TokenStream> {
    let target_ident = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "field missing ident"))?;

    // Extract container and field from source option
    let (container_ident, field_path, field_optional) = field.options.source.as_ref().map_or_else(
        || (None, target_ident.to_string(), false),
        |source| {
            // For source like "book?.display_name?", we want container="book", field="display_name", and track if field is optional
            let parts: Vec<&str> = source.split('.').collect();
            if parts.len() >= 2 {
                let container_name = parts[0].trim_end_matches('?');
                let field_name_raw = parts[1];
                let field_optional = field_name_raw.ends_with('?');
                let field_name = field_name_raw.trim_end_matches('?');
                let container_ident = format_ident!("{}", container_name);
                let field_path = field_name.to_string();
                (Some(container_ident), field_path, field_optional)
            } else {
                (None, target_ident.to_string(), false)
            }
        },
    );

    // Determine container field for field mask
    let _container_field = field_mask.field.as_ref().map_or_else(
        || {
            container_ident
                .as_ref()
                .map_or_else(|| format_ident!("book"), Clone::clone) // Default fallback
        },
        Clone::clone,
    );

    let mask_field = &field_mask.mask;

    // Generate the field extraction logic
    let extract_logic = container_ident.as_ref().map_or_else(
        || {
            quote! {
                {
                    let target = source.#target_ident;
                    Some(target)
                }
            }
        },
        |container_ident| {
            let field_ident = format_ident!("{}", field_path);
            if field_optional {
                quote! {
                    {
                        let target = source.#container_ident.clone();
                        if let Some(target) = target {
                            target.#field_ident
                        } else {
                            None
                        }
                    }
                }
            } else {
                quote! {
                    {
                        let target = source.#container_ident.clone();
                        if let Some(target) = target {
                            Some(target.#field_ident)
                        } else {
                            None
                        }
                    }
                }
            }
        },
    );

    // Generate field error path
    let field_error_path = make_field_error_path(&field_path, None);

    // Generate the complete parsing logic with field mask check
    field.options.convert.as_ref().map_or_else(|| Ok(quote! {
        #target_ident: {
            // Check if field mask allows this field
            if let Some(mask) = source.#mask_field.as_ref() {
                if mask.paths.contains(&#field_path.to_string()) {
                    #extract_logic
                } else {
                    None
                }
            } else {
                // No field mask provided, don't parse this field
                None
            }
        },
    }), |convert| {
        let convert_impl = convert
            .parse
            .as_ref()
            .map(ToTokens::to_token_stream)
            .or_else(|| convert.module.as_ref().map(|module| quote!(#module::parse)))
            .unwrap();

        Ok(quote! {
            #target_ident: {
                // Check if field mask allows this field
                if let Some(mask) = source.#mask_field.as_ref() {
                    if mask.paths.contains(&#field_path.to_string()) {
                        let target = #extract_logic;
                        let target = if let Some(target) = target {
                            Some(#convert_impl(target).map_err(|err: RequestError| err.wrap_path(#field_error_path))?)
                        } else {
                            None
                        };
                        target
                    } else {
                        None
                    }
                } else {
                    // No field mask provided, don't parse this field
                    None
                }
            },
        })
    })
}
