use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::{Ident, Literal, TokenStream};
use prost_types::{field_descriptor_proto, DescriptorProto, OneofDescriptorProto};
use quote::{format_ident, quote};
use syn::TypePath;

use crate::context::Context;

pub fn write_message_oneofs(context: &Context, s: &mut TokenStream, message: &DescriptorProto) {
    if message.oneof_decl.is_empty() {
        return;
    }

    for (oneof_index, oneof) in message.oneof_decl.iter().enumerate() {
        write_oneof_name(context, s, message, oneof);
        write_oneof_variant_names(context, s, message, oneof, oneof_index);
        write_oneof_variant_from(context, s, message, oneof, oneof_index);
        write_oneof_variant_utility(context, s, message, oneof, oneof_index);
    }

    write_oneof_into_owner(context, s, message);
}

fn write_oneof_name(
    context: &Context,
    s: &mut TokenStream,
    message: &DescriptorProto,
    oneof: &OneofDescriptorProto,
) {
    let message_ident = context.get_type_ident(message.name());
    let oneof_name_ident =
        format_ident!("{}_ONEOF_NAME", oneof.name().to_case(Case::ScreamingSnake));
    let oneof_name_literal = Literal::string(oneof.name());
    s.extend(quote! {
        impl #message_ident {
            pub const #oneof_name_ident: &'static str = #oneof_name_literal;
        }
    });
}

fn write_oneof_variant_names(
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
            field.name().to_case(Case::ScreamingSnake)
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

fn write_oneof_variant_from(
    context: &Context,
    s: &mut TokenStream,
    message: &DescriptorProto,
    oneof: &OneofDescriptorProto,
    oneof_index: usize,
) {
    let mut from_map = BTreeMap::<String, Vec<Ident>>::new();

    for field in message
        .field
        .iter()
        .filter(|field| field.oneof_index == Some(oneof_index as i32))
    {
        let source_type = match field.r#type() {
            field_descriptor_proto::Type::Message => {
                // Create field type path based on current context's path, package, and field type name.
                let mut field_type_name = field
                    .type_name
                    .as_ref()
                    .unwrap()
                    .trim_start_matches('.')
                    .split('.')
                    .peekable();
                let mut field_type_path = String::new();
                while let Some(part) = field_type_name.next() {
                    field_type_path.push_str("::");
                    if field_type_name.peek().is_none() {
                        field_type_path.push_str(&part.to_case(Case::Pascal));
                        break;
                    }
                    field_type_path.push_str(&part.to_case(Case::Snake));
                }
                let field_type_path = format!(
                    "{}{}",
                    context.package_name.split('.').map(|_| "super").join("::"),
                    field_type_path
                );
                let field_type_ident = syn::parse_str::<TypePath>(&field_type_path).unwrap();
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
            field_descriptor_proto::Type::Int32 => quote! { i32 },
            field_descriptor_proto::Type::Int64 => quote! { i64 },
            field_descriptor_proto::Type::Uint32 => quote! { u32 },
            field_descriptor_proto::Type::Uint64 => quote! { u64 },
            field_descriptor_proto::Type::Sint32 => quote! { i32 },
            field_descriptor_proto::Type::Sint64 => quote! { i64 },
            field_descriptor_proto::Type::Fixed32 => quote! { u32 },
            field_descriptor_proto::Type::Fixed64 => quote! { u64 },
            field_descriptor_proto::Type::Sfixed32 => quote! { i32 },
            field_descriptor_proto::Type::Sfixed64 => quote! { i64 },
            field_descriptor_proto::Type::Group => {
                panic!("groups are not supported")
            }
        }
        .to_string();

        let variant_ident = format_ident!("{}", field.name().to_case(Case::Pascal));
        from_map.entry(source_type).or_default().push(variant_ident);
    }

    // Only implement from if there is a single variant for the given type.
    let oneof_ident = context.get_oneof_ident(message, oneof);
    for (source_type, variant_ident) in from_map.into_iter() {
        if variant_ident.len() != 1 {
            continue;
        }
        let variant_ident = variant_ident.into_iter().next().unwrap();
        let source_type = syn::parse_str::<TypePath>(&source_type).unwrap();
        s.extend(quote! {
            impl From<#source_type> for #oneof_ident {
                fn from(value: #source_type) -> Self {
                    Self::#variant_ident(value)
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
            let variant_ident = format_ident!("{}", oneof.name().to_case(Case::Snake));
            s.extend(quote! {
                /// From source variant type to owner message type.
                impl From<#source_type> for #message_ident {
                    fn from(value: #source_type) -> Self {
                        Self {
                            #variant_ident: Some(value.into()),
                        }
                    }
                }
            })
        }
    }
}

fn write_oneof_into_owner(context: &Context, s: &mut TokenStream, message: &DescriptorProto) {
    if message.oneof_decl.len() != 1
        || !message
            .field
            .iter()
            .all(|field| field.oneof_index.is_some())
    {
        return;
    }
    let message_ident = context.get_type_ident(message.name());
    let oneof = message.oneof_decl.first().unwrap();
    let oneof_ident = context.get_oneof_ident(message, oneof);
    let variant_ident = format_ident!("{}", oneof.name().to_case(Case::Snake));

    s.extend(quote! {
        /// From oneof type to owner message type.
        impl From<#oneof_ident> for #message_ident {
            fn from(value: #oneof_ident) -> Self {
                Self {
                    #variant_ident: Some(value),
                }
            }
        }
    })
}

fn write_oneof_variant_utility(
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
            field.name().to_case(Case::ScreamingSnake)
        );
        let oneof_field_ident = format_ident!("{}", field.name().to_case(Case::Pascal));
        variant_cases.extend(quote! {
            Self::#oneof_field_ident(_) => Self::#variant_name_ident,
        });
    }

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
