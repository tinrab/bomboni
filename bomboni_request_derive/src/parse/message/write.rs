use bomboni_core::syn::type_is_phantom;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::parse::{
    field_type_info::get_field_type_info,
    message::utility::get_field_extract,
    options::{FieldExtractStep, ParseDerive, ParseField, ParseOptions, ParseQuery, ParseResource},
    write_utility::expand_field_write_type,
};

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> syn::Result<TokenStream> {
    let mut write_fields = quote!();

    // Write derived fields
    for field in fields
        .iter()
        .filter(|field| !field.options.skip && field.options.derive.is_some())
    {
        write_fields.extend(expand_write_field(options, field)?);
    }

    for field in fields.iter().filter(|field| {
        !field.options.skip && !type_is_phantom(&field.ty) && field.options.derive.is_none()
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
            write_fields.extend(expand_write_field(options, field)?);
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

fn expand_write_field(options: &ParseOptions, field: &ParseField) -> syn::Result<TokenStream> {
    let target_ident = field.ident.as_ref().unwrap();

    let extract = get_field_extract(field)?;
    let mut first_field = true;
    let last_item_unwrap = matches!(
        extract.steps.last(),
        Some(
            FieldExtractStep::Unwrap
                | FieldExtractStep::UnwrapOr(_)
                | FieldExtractStep::UnwrapOrDefault
        )
    );
    let mut inject_impl = quote!();
    let mut set_impl = quote!();
    for step in &extract.steps {
        match step {
            FieldExtractStep::Field(field_name) => {
                let field_ident = format_ident!("{}", field_name);
                if first_field {
                    first_field = false;
                    if last_item_unwrap {
                        inject_impl.extend(quote!(*));
                    }
                    inject_impl.extend(quote! {
                        source.#field_ident
                    });
                } else {
                    inject_impl.extend(quote! {
                        .#field_ident
                    });
                }
            }
            FieldExtractStep::Unwrap
            | FieldExtractStep::UnwrapOr(_)
            | FieldExtractStep::UnwrapOrDefault => {
                inject_impl.extend(quote! {
                    .get_or_insert_with(|| Default::default())
                });
            }
            FieldExtractStep::Unbox => {
                set_impl.extend(quote! {
                    let source_field = Box::new(source_field);
                });
            }
            FieldExtractStep::StringFilterEmpty => {
                set_impl.extend(quote! {
                    let source_field = source_field
                        .filter(|s| !s.is_empty())
                        .unwrap_or_default();
                });
            }
            FieldExtractStep::EnumerationFilterUnspecified => {
                set_impl.extend(quote! {
                    let source_field = source_field
                        .filter(|s| *s != 0)
                        .unwrap_or(0);
                });
            }
        }
    }

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
            let source_value = if *target_borrow {
                quote!(&target.#target_ident)
            } else {
                quote!(target.#target_ident)
            };

            return Ok(quote! {
                let source_field = #write_impl(#source_value);
                #set_impl
                #inject_impl = source_field;
            });
        }

        return Ok(quote! {
            #write_impl(&target, &mut source);
        });
    }

    let field_type_info = get_field_type_info(options, &field.options, &field.ty)?;

    let write_inner_impl = expand_field_write_type(&field.options, &field_type_info);

    Ok(quote! {
        let source_field = target.#target_ident;
        #write_inner_impl
        #set_impl
        #inject_impl = source_field;
    })
}

fn expand_write_resource(options: &ParseResource, field: &ParseField) -> TokenStream {
    let target_ident = field.ident.as_ref().unwrap();
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

    write_impl
}

fn expand_write_query(query: &ParseQuery, field: &ParseField, search: bool) -> TokenStream {
    let target_ident = field.ident.as_ref().unwrap();
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

    write_impl
}
