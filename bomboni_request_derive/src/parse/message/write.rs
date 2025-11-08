use bomboni_core::syn::type_is_phantom;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

use crate::parse::{
    message::utility::get_field_extract,
    options::{ParseDerive, ParseField, ParseOptions, ParseQuery, ParseResource},
    write_utility::{expand_field_inject, expand_write_field_type},
};

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> syn::Result<TokenStream> {
    let mut write_fields = quote!();

    // Write derived fields
    for field in fields
        .iter()
        .filter(|field| !field.options.skip && field.options.derive.is_some())
    {
        write_fields.extend(expand_write_field(field)?);
    }

    // Write field mask fields
    for field in fields.iter().filter(|field| !field.options.skip) {
        if let Some(field_mask) = &field.options.field_mask {
            write_fields.extend(expand_write_field_mask(field, field_mask)?);
        }
    }

    for field in fields.iter().filter(|field| {
        !field.options.skip
            && !type_is_phantom(&field.ty)
            && field.options.derive.is_none()
            && field.options.field_mask.is_none()
    }) {
        if let Some(resource) = field.resource.as_ref() {
            write_fields.extend(expand_write_resource(resource, field));
        } else if let Some(query) = field.list_query.as_ref().or(field.search_query.as_ref()) {
            write_fields.extend(expand_write_query(
                query,
                field,
                field.search_query.is_some(),
            ));
        } else {
            write_fields.extend(expand_write_field(field)?);
        }
    }

    let source = &options.source;
    let ident = &options.ident;
    let (impl_generics, type_generics, where_clause) = options.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics From<#ident #type_generics> for #source #where_clause {
            #[allow(clippy::needless_update)]
            fn from(target: #ident #type_generics) -> Self {
                let mut source: #source = Default::default();
                #write_fields
                source
            }
        }
    })
}

fn expand_write_field(field: &ParseField) -> syn::Result<TokenStream> {
    let target_ident = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "field missing ident"))?;
    let extract = get_field_extract(field)?;

    if let Some(ParseDerive {
        write,
        module,
        target_borrow,
        ..
    }) = field.options.derive.as_ref()
    {
        let write_impl = write
            .as_ref()
            .map(ToTokens::to_token_stream)
            .or_else(|| module.as_ref().map(|module| quote!(#module::write)))
            .ok_or_else(|| {
                syn::Error::new_spanned(target_ident, "missing derive write implementation")
            })?;

        if field.options.source.is_some() || field.options.extract.is_some() {
            let inject_impl = expand_field_inject(&extract, &field.options, None);
            let source_value = if *target_borrow {
                quote!(&target.#target_ident)
            } else {
                quote!(target.#target_ident)
            };

            return Ok(quote! {{
                let source_field = #write_impl(#source_value);
                #inject_impl
            }});
        }

        return Ok(quote! {{
            #write_impl(&target, &mut source);
        }});
    }

    let field_type_info = field.type_info.as_ref().ok_or_else(|| {
        syn::Error::new(proc_macro2::Span::call_site(), "field missing type info")
    })?;
    let inject_impl = expand_field_inject(&extract, &field.options, Some(field_type_info));
    let write_field_impl = expand_write_field_type(&field.options, field_type_info, inject_impl);

    Ok(quote! {{
        let source_field = target.#target_ident;
        #write_field_impl
    }})
}

fn expand_write_resource(options: &ParseResource, field: &ParseField) -> syn::Result<TokenStream> {
    let target_ident = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "field missing ident"))?;
    let mut write_impl = quote!();

    if options.name.write {
        let source_ident = &options.name.source;
        write_impl.extend(quote! {
            source.#source_ident = target.#target_ident.name;
        });
    }
    if options.create_time.write {
        let source_ident = &options.create_time.source;
        write_impl.extend(quote! {
            source.#source_ident = target.#target_ident.create_time.map(Into::into);
        });
    }
    if options.update_time.write {
        let source_ident = &options.update_time.source;
        write_impl.extend(quote! {
            source.#source_ident = target.#target_ident.update_time.map(Into::into);
        });
    }
    if options.delete_time.write {
        let source_ident = &options.delete_time.source;
        write_impl.extend(quote! {
            source.#source_ident = target.#target_ident.delete_time.map(Into::into);
        });
    }
    if options.deleted.write {
        let source_ident = &options.deleted.source;
        write_impl.extend(quote! {
            source.#source_ident = target.#target_ident.deleted;
        });
    }
    if options.etag.write {
        let source_ident = &options.etag.source;
        write_impl.extend(quote! {
            source.#source_ident = target.#target_ident.etag;
        });
    }

    Ok(write_impl)
}

fn expand_write_field_mask(
    field: &ParseField,
    field_mask: &crate::parse::options::ParseFieldMask,
) -> syn::Result<TokenStream> {
    let target_ident = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "field missing ident"))?;
    let _field_name = target_ident.to_string();

    // Extract container and field from source option
    let (container_ident, field_path) = if let Some(source) = field.options.source.as_ref() {
        // For source like "book?.display_name", we want container="book", field="display_name"
        // For source like "book?.author?.name", we want container="book", field="name"
        let parts: Vec<&str> = source.split('.').collect();
        if parts.len() >= 2 {
            let container_name = parts[0].trim_end_matches('?');
            let field_name = parts.last().unwrap().trim_end_matches('?');
            let container_ident = format_ident!("{}", container_name);
            let field_path = field_name.to_string();
            (Some(container_ident), field_path)
        } else {
            (None, source.clone()) // Use the actual source string when no container
        }
    } else {
        (None, target_ident.to_string())
    };

    // Determine container field for field mask
    let _container_field = if let Some(field) = &field_mask.field {
        field.clone()
    } else if let Some(container_ident) = &container_ident {
        container_ident.clone()
    } else {
        format_ident!("book") // Default fallback
    };

    let mask_field = &field_mask.mask;

    // Extract the container field (e.g., book) once
    let container_access = if let Some(container_ident) = &container_ident {
        quote! { source.#container_ident }
    } else {
        quote! { source }
    };

    // Generate the nested field access (e.g., display_name)
    let nested_field = format_ident!("{}", field_path);

    // Generate the complete writing logic
    Ok(quote! {{
        if let Some(value) = target.#target_ident {
            // Update the field mask to include this field path
            let mask = source.#mask_field.get_or_insert_with(|| Default::default());
            if !mask.paths.contains(&#field_path.to_string()) {
                mask.paths.push(#field_path.to_string());
            }

            // Update the container field with the new value
            let container = #container_access.get_or_insert_with(|| Default::default());
            container.#nested_field = value;
        }
    }})
}

fn expand_write_query(
    query: &ParseQuery,
    field: &ParseField,
    search: bool,
) -> syn::Result<TokenStream> {
    let target_ident = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "field missing ident"))?;
    let mut write_impl = quote!();

    if query.query.write && search {
        let source_ident = &query.query.source;
        write_impl.extend(quote! {
            source.#source_ident = target.#target_ident.query;
        });
    }
    if query.page_size.write {
        let source_ident = &query.page_size.source;
        write_impl.extend(quote! {
            source.#source_ident = Some(target.#target_ident.page_size.try_into().unwrap());
        });
    }
    if query.page_token.write {
        let source_ident = &query.page_token.source;
        write_impl.extend(quote! {
            source.#source_ident = target.#target_ident.page_token.map(|page_token| page_token.to_string());
        });
    }
    if query.filter.write {
        let source_ident = &query.filter.source;
        write_impl.extend(quote! {
            source.#source_ident = if target.#target_ident.filter.is_empty() {
                None
            } else {
                Some(target.#target_ident.filter.to_string())
            };
        });
    }
    if query.order_by.write {
        let source_ident = &query.order_by.source;
        write_impl.extend(quote! {
            source.#source_ident = if target.#target_ident.ordering.is_empty() {
                None
            } else {
                Some(target.#target_ident.ordering.to_string())
            };
        });
    }

    Ok(write_impl)
}
