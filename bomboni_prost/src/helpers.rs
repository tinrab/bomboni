use convert_case::Case;
use proc_macro2::TokenStream;
use prost_types::DescriptorProto;
use prost_types::{EnumDescriptorProto, FileDescriptorProto};
use quote::{format_ident, quote};

use crate::context::Context;
use crate::utility::str_to_case;

pub fn write_helpers(context: &Context, s: &mut TokenStream, files: &[&FileDescriptorProto]) {
    let mut src = quote!();
    for file in files {
        for message in &file.message_type {
            write_message_helpers(context, &mut src, message);
        }

        for enum_type in &file.enum_type {
            write_enum_helpers(context, &mut src, enum_type);
        }
    }

    if !src.is_empty() {
        let mod_name = if let Some(mod_name) = context.config.api.helpers_mod.as_ref() {
            format_ident!("{}", mod_name)
        } else {
            return;
        };
        s.extend(quote! {
            pub mod #mod_name {
                #src
            }
        });
    }
}

fn write_message_helpers(context: &Context, s: &mut TokenStream, message: &DescriptorProto) {
    let mut path = context.path.clone();
    path.push(message.name.clone().unwrap());
    let nested_context = Context {
        path: path.clone(),
        package_name: context.package_name.clone(),
        ..*context
    };

    let mut src = quote!();
    for nested_enum in &message.enum_type {
        write_enum_helpers(&nested_context, &mut src, nested_enum);
    }
    for nested_message in &message.nested_type {
        write_message_helpers(&nested_context, &mut src, nested_message);
    }

    if !src.is_empty() {
        let mod_name = format_ident!("{}", str_to_case(message.name(), Case::Snake));
        s.extend(quote! {
            pub mod #mod_name {
                #src
            }
        });
    }
}

fn write_enum_helpers(context: &Context, s: &mut TokenStream, enum_type: &EnumDescriptorProto) {
    let enum_ident = context.get_type_expr_relative_path(enum_type.name(), 2);

    if context.config.api.serde {
        let mod_ident = format_ident!("{}_serde", str_to_case(enum_type.name(), Case::Snake));
        s.extend(quote! {
            /// Utility for working with i32s in message fields.
            /// Usable with #[serde(with = "...")]
            pub mod #mod_ident {
                use ::serde::{Serialize, Deserialize};

                pub fn serialize<S>(
                    value: &i32,
                    serializer: S,
                ) -> Result<<S as ::serde::Serializer>::Ok, <S as ::serde::Serializer>::Error>
                where
                    S: ::serde::Serializer,
                {
                    let value = #enum_ident::try_from(*value).unwrap();
                    value.serialize(serializer)
                }

                pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    let value = #enum_ident::deserialize(deserializer)?;
                    Ok(value as i32)
                }
            }
        });
    }
}
