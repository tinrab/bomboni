use bomboni_core::string::{str_to_case, Case};
use proc_macro2::TokenStream;
use prost_types::DescriptorProto;
use quote::{format_ident, quote};

use crate::enums::write_enum;
use crate::{context::Context, oneofs::write_message_oneofs};

pub fn write_message(context: &Context, s: &mut TokenStream, message: &DescriptorProto) {
    if context.config.api.field_names {
        write_field_names(context, s, message);
    }
    if context.config.api.oneof_utility {
        write_message_oneofs(context, s, message);
    }

    let mut path = context.path.clone();
    path.push(message.name.clone().unwrap());
    let nested_context = Context {
        path: path.clone(),
        package_name: context.package_name.clone(),
        ..*context
    };

    for nested_enum in &message.enum_type {
        write_enum(&nested_context, s, nested_enum);
    }

    for nested_message in &message.nested_type {
        // Skip map entries
        if nested_message
            .options
            .as_ref()
            .and_then(|o| o.map_entry)
            .unwrap_or(false)
        {
            continue;
        }

        write_message(&nested_context, s, nested_message);
    }
}

fn write_field_names(context: &Context, s: &mut TokenStream, message: &DescriptorProto) {
    if message.field.is_empty() {
        return;
    }
    let mut names = TokenStream::new();
    for field in &message.field {
        let field_name_ident = format_ident!(
            "{}_FIELD_NAME",
            str_to_case(field.name(), Case::ScreamingSnake)
        );
        let field_name = field.name();
        names.extend(quote! {
            pub const #field_name_ident: &'static str = #field_name;
        });
    }
    let message_ident = context.get_type_expr_path(message.name());
    s.extend(quote! {
        impl #message_ident {
            #names
        }
    });
}
