use darling::FromMeta;
use itertools::Itertools;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, ToTokens};

use crate::parse::{DeriveOptions, ParseField, ParseOptions};
use crate::utility::{get_proto_type_info, ProtoTypeInfo};

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> TokenStream {
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

    let field_type = &field.ty;
    let ProtoTypeInfo {
        is_option,
        is_nested,
        is_vec,
        is_box,
        map_ident,
        ..
    } = get_proto_type_info(field_type);

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

    if field.wrapper {
        write.extend(quote! {
            let source: #field_type = source;
            let source = source.into();
        });
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
