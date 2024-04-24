use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::parse::{field_type_info::FieldTypeInfo, options::ParseFieldOptions};

use super::options::ParseConvert;

pub fn expand_field_write_type(
    field_options: &ParseFieldOptions,
    field_type_info: &FieldTypeInfo,
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
                    let source_field = if let Some(source_field) = source_field {
                        #write_impl
                        Some(source_field)
                    } else {
                        None
                    };
                };
            }
            "Box" => {
                return quote! {
                    let source_field = *source_field;
                    #write_impl
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
                };
            }
            "HashMap" | "BTreeMap" => {
                let container_ident = if container_ident == "HashMap" {
                    quote! { HashMap }
                } else {
                    quote! { BTreeMap }
                };
                return if write_impl.is_empty() {
                    quote!()
                } else {
                    quote! {
                        let mut m = #container_ident::new();
                        for (key, source_field) in source_field.into_iter() {
                            #write_impl
                            m.insert(key, source_field);
                        }
                        let source_field = m;
                    }
                };
            }
            _ => unreachable!(),
        }
    }

    write_impl
}
