use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::parse::{
    field_type_info::FieldTypeInfo,
    options::{FieldExtract, FieldExtractStep, ParseConvert, ParseFieldOptions},
};

pub fn expand_field_inject(
    extract: &FieldExtract,
    field_options: &ParseFieldOptions,
    field_type_info: Option<&FieldTypeInfo>,
) -> TokenStream {
    let mut inject_impl = quote!();
    let mut set_impl = quote!();

    let last_unwrap_step = extract
        .steps
        .iter()
        .rposition(|step| matches!(step, FieldExtractStep::Unwrap));
    let mut target_option = if field_type_info
        .and_then(|field_type_info| field_type_info.container_ident.as_deref())
        == Some("Option")
    {
        Some(())
    } else {
        None
    };

    let mut dereference_source = false;
    let mut inserted_field = false;

    for (i, step) in extract.steps.iter().enumerate() {
        match step {
            FieldExtractStep::Field(field_name) => {
                let field_ident = format_ident!("{}", field_name);
                set_impl.extend(quote! {
                    .#field_ident
                });
                dereference_source = false;
            }
            FieldExtractStep::Unwrap => {
                if last_unwrap_step == Some(i) && target_option.take().is_some() {
                    inject_impl = quote! {
                        let source_field = Some(source_field);
                        #inject_impl
                    };
                } else if field_options.oneof && last_unwrap_step == Some(i) {
                    set_impl.extend(quote! {
                        .insert(source_field);
                    });
                    inserted_field = true;
                } else {
                    set_impl.extend(quote! {
                        .get_or_insert_with(|| Default::default())
                    });
                    dereference_source = true;
                }
            }
            FieldExtractStep::UnwrapOr(_) | FieldExtractStep::UnwrapOrDefault => {
                inject_impl = quote! {
                    let source_field = Some(source_field);
                    #inject_impl
                };
            }
            FieldExtractStep::Unbox => {
                inject_impl = quote! {
                    let source_field = Box::new(source_field);
                    #inject_impl
                };
            }
            FieldExtractStep::StringFilterEmpty
            | FieldExtractStep::EnumerationFilterUnspecified => {}
        }
    }

    if dereference_source || !matches!(extract.steps.first(), Some(FieldExtractStep::Field(_))) {
        set_impl = quote!(*source #set_impl);
    } else {
        set_impl = quote!(source #set_impl);
    }

    if inserted_field {
        quote! {
            #inject_impl
            #set_impl
        }
    } else {
        quote! {
            #inject_impl
            #set_impl = source_field;
        }
    }
}

pub fn expand_write_field_type(
    field_options: &ParseFieldOptions,
    field_type_info: &FieldTypeInfo,
    inject_impl: TokenStream,
) -> TokenStream {
    let mut write_impl = quote!();

    if let Some(try_from) = field_options.try_from.as_ref() {
        let err_literal = format!("failed to convert to `{}`", try_from.to_token_stream());
        write_impl.extend(quote! {
            let source_field = TryInto::<#try_from>::try_into(source_field)
                .expect(#err_literal);
        });
    } else if let Some(ParseConvert { write, module, .. }) = field_options.convert.as_ref() {
        let convert_impl = write
            .as_ref()
            .map(ToTokens::to_token_stream)
            .or_else(|| module.as_ref().map(|module| quote!(#module::write)))
            .unwrap();
        write_impl.extend(quote! {
            let source_field = #convert_impl(source_field);
        });
    } else if field_options.enumeration && !field_options.keep_primitive {
        write_impl.extend(quote! {
            /// Write enumeration
            let source_field = source_field as i32;
        });
    } else if let Some(primitive_ident) = field_type_info.primitive_ident.as_ref() {
        if field_type_info.primitive_message && !field_options.keep_primitive {
            write_impl.extend(quote! {
                /// Write primitive message
                let source_field = source_field.into();
            });
        }

        if field_options.wrapper {
            let wrapper_ident = format_ident!(
                "{}",
                match primitive_ident.as_str() {
                    "String" => "StringValue",
                    "bool" => "BoolValue",
                    "f32" => "FloatValue",
                    "f64" => "DoubleValue",
                    "i8" | "i16" | "i32" => "Int32Value",
                    "u8" | "u16" | "u32" => "UInt32Value",
                    "i64" | "isize" => "Int64Value",
                    "u64" | "usize" => "UInt64Value",
                    _ => unreachable!(),
                }
            );

            write_impl.extend(quote! {
                let source_field: #wrapper_ident = source_field.into();
            });
        }
    }

    if field_type_info.generic_param.is_some() {
        write_impl.extend(quote! {
            /// Write generic
            let source_field = source_field.into();
        });
    }

    if let Some(container_ident) = field_type_info.container_ident.as_ref() {
        match container_ident.as_str() {
            "Option" => {
                return quote! {
                    if let Some(source_field) = source_field {
                        #write_impl
                        #inject_impl
                    }
                };
            }
            "Box" => {
                return quote! {
                    let source_field = *source_field;
                    #write_impl
                    #inject_impl
                };
            }
            "Vec" => {
                return quote! {
                    let mut v = Vec::new();
                    for (i, source_field) in source_field.into_iter().enumerate() {
                        #write_impl
                        v.push(source_field);
                    }
                    let source_field = v;
                    #inject_impl
                };
            }
            "HashMap" | "BTreeMap" => {
                let container_ident = if container_ident == "HashMap" {
                    quote! { HashMap }
                } else {
                    quote! { BTreeMap }
                };
                return if write_impl.is_empty() {
                    quote! {
                        #inject_impl
                    }
                } else {
                    quote! {
                        let mut m = #container_ident::new();
                        for (key, source_field) in source_field.into_iter() {
                            #write_impl
                            m.insert(key, source_field);
                        }
                        let source_field = m;
                        #inject_impl
                    }
                };
            }
            _ => unreachable!(),
        }
    }

    quote! {
        #write_impl
        #inject_impl
    }
}
