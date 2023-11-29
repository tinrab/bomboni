use std::collections::BTreeMap;

use crate::utility::str_to_case;
use convert_case::Case;
use proc_macro2::{Ident, Literal, TokenStream};
use prost_types::{field_descriptor_proto, DescriptorProto, OneofDescriptorProto};
use quote::{format_ident, quote};
use syn::TypePath;

use crate::context::Context;

pub fn write_message_oneofs(context: &Context, s: &mut TokenStream, message: &DescriptorProto) {
    if message.oneof_decl.is_empty() {
        return;
    }

    if context.config.api.field_names {
        for (oneof_index, oneof) in message.oneof_decl.iter().enumerate() {
            if message.field.iter().any(|field| {
                field.oneof_index == Some(oneof_index as i32) && field.proto3_optional()
            }) {
                continue;
            }

            write_name(context, s, message, oneof);
            write_variant_names(context, s, message, oneof, oneof_index);
        }
    }

    if context.config.api.oneof_utility {
        for (oneof_index, oneof) in message.oneof_decl.iter().enumerate() {
            if message.field.iter().any(|field| {
                field.oneof_index == Some(oneof_index as i32) && field.proto3_optional()
            }) {
                continue;
            }

            write_variant_from(context, s, message, oneof, oneof_index);
            write_variant_utility(context, s, message, oneof, oneof_index);
        }
    }

    if context.config.api.oneof_utility {
        write_into_owner(context, s, message);
    }

    // panic!();
}

fn write_name(
    context: &Context,
    s: &mut TokenStream,
    message: &DescriptorProto,
    oneof: &OneofDescriptorProto,
) {
    let message_ident = context.get_type_ident(message.name());
    let oneof_name_ident = format_ident!(
        "{}_ONEOF_NAME",
        str_to_case(oneof.name(), Case::ScreamingSnake)
    );
    let oneof_name_literal = Literal::string(oneof.name());
    s.extend(quote! {
        impl #message_ident {
            pub const #oneof_name_ident: &'static str = #oneof_name_literal;
        }
    });
}

fn write_variant_names(
    context: &Context,
    s: &mut TokenStream,
    message: &DescriptorProto,
    oneof: &OneofDescriptorProto,
    oneof_index: usize,
) {
    let mut variant_names = TokenStream::new();
    for field in message
        .field
        .iter()
        .filter(|field| field.oneof_index == Some(oneof_index as i32))
    {
        let variant_name_ident = format_ident!(
            "{}_VARIANT_NAME",
            str_to_case(field.name(), Case::ScreamingSnake)
        );
        let variant_name_literal = Literal::string(field.name());
        variant_names.extend(quote! {
            pub const #variant_name_ident: &'static str = #variant_name_literal;
        });
    }
    let oneof_ident = context.get_oneof_ident(message, oneof);
    s.extend(quote! {
        impl #oneof_ident {
            #variant_names
        }
    });
}

fn write_variant_from(
    context: &Context,
    s: &mut TokenStream,
    message: &DescriptorProto,
    oneof: &OneofDescriptorProto,
    oneof_index: usize,
) {
    let message_full_type_name = context.get_proto_full_type_name(message.name());
    let message_type_name_ref = format!(".{message_full_type_name}");

    let mut from_map = BTreeMap::<String, Vec<Ident>>::new();

    for field in message
        .field
        .iter()
        .filter(|field| field.oneof_index == Some(oneof_index as i32))
    {
        let source_type = match field.r#type() {
            field_descriptor_proto::Type::Message => {
                // Skip if it references itself
                if field.type_name.as_ref().unwrap() == &message_type_name_ref {
                    continue;
                }
                let field_type_ident =
                    context.get_ident_from_type_name_reference(field.type_name.as_ref().unwrap());
                quote! { #field_type_ident }
            }
            field_descriptor_proto::Type::Enum => {
                quote! { todo!() }
            }
            field_descriptor_proto::Type::String => quote! { String },
            field_descriptor_proto::Type::Bytes => quote! { Vec<u8> },
            field_descriptor_proto::Type::Bool => quote! { bool },
            field_descriptor_proto::Type::Double => quote! { f64 },
            field_descriptor_proto::Type::Float => quote! { f32 },
            field_descriptor_proto::Type::Int32
            | field_descriptor_proto::Type::Sint32
            | field_descriptor_proto::Type::Sfixed32 => quote! { i32 },
            field_descriptor_proto::Type::Int64
            | field_descriptor_proto::Type::Sint64
            | field_descriptor_proto::Type::Sfixed64 => quote! { i64 },
            field_descriptor_proto::Type::Uint32 | field_descriptor_proto::Type::Fixed32 => {
                quote! { u32 }
            }
            field_descriptor_proto::Type::Uint64 | field_descriptor_proto::Type::Fixed64 => {
                quote! { u64 }
            }
            field_descriptor_proto::Type::Group => {
                panic!("groups are not supported")
            }
        }
        .to_string();

        let variant_ident = format_ident!("{}", str_to_case(field.name(), Case::Pascal));
        from_map.entry(source_type).or_default().push(variant_ident);
    }

    // Only implement from if there is a single variant for the given type.
    let oneof_ident = context.get_oneof_ident(message, oneof);
    for (source_type, variant_ident) in from_map {
        if variant_ident.len() != 1 {
            continue;
        }
        let variant_ident = variant_ident.into_iter().next().unwrap();
        let source_type = syn::parse_str::<TypePath>(&source_type).unwrap();
        s.extend(quote! {
            impl From<#source_type> for #oneof_ident {
                fn from(value: #source_type) -> Self {
                    Self::#variant_ident(value.into())
                }
            }
        });

        // Maybe implement into owner trait
        if message.oneof_decl.len() == 1
            && message
                .field
                .iter()
                .all(|field| field.oneof_index.is_some())
        {
            let message_ident = context.get_type_ident(message.name());
            let variant_ident = format_ident!("{}", str_to_case(oneof.name(), Case::Snake));
            s.extend(quote! {
                /// From source variant type to owner message type.
                impl From<#source_type> for #message_ident {
                    fn from(value: #source_type) -> Self {
                        Self {
                            #variant_ident: Some(value.into()),
                        }
                    }
                }
            });
        }
    }
}

fn write_variant_utility(
    context: &Context,
    s: &mut TokenStream,
    message: &DescriptorProto,
    oneof: &OneofDescriptorProto,
    oneof_index: usize,
) {
    let mut variant_cases = TokenStream::new();
    for field in message
        .field
        .iter()
        .filter(|field| field.oneof_index == Some(oneof_index as i32))
    {
        let variant_name_ident = format_ident!(
            "{}_VARIANT_NAME",
            str_to_case(field.name(), Case::ScreamingSnake)
        );
        let oneof_field_ident = format_ident!("{}", str_to_case(field.name(), Case::Pascal));
        variant_cases.extend(quote! {
            Self::#oneof_field_ident(_) => Self::#variant_name_ident,
        });
    }

    if !variant_cases.is_empty() {
        let oneof_ident = context.get_oneof_ident(message, oneof);
        s.extend(quote! {
            impl #oneof_ident {
                pub fn get_variant_name(&self) -> &'static str {
                    match self {
                        #variant_cases
                    }
                }
            }
        });
    }
}

fn write_into_owner(context: &Context, s: &mut TokenStream, message: &DescriptorProto) {
    if message.oneof_decl.len() != 1
        || !message
            .field
            .iter()
            .all(|field| field.oneof_index.is_some() && !field.proto3_optional())
    {
        return;
    }
    let message_ident = context.get_type_ident(message.name());
    let oneof = message.oneof_decl.first().unwrap();
    let oneof_ident = context.get_oneof_ident(message, oneof);
    let variant_ident = format_ident!("{}", str_to_case(oneof.name(), Case::Snake));

    s.extend(quote! {
        impl From<#oneof_ident> for #message_ident {
            fn from(value: #oneof_ident) -> Self {
                Self {
                    #variant_ident: Some(value),
                }
            }
        }
    });
}
