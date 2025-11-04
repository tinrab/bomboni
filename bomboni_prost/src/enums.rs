use bomboni_core::string::{Case, str_to_case};
use proc_macro2::TokenStream;
use prost_types::EnumDescriptorProto;
use quote::{format_ident, quote};

use crate::context::Context;

pub fn write_enum(context: &Context, s: &mut TokenStream, enum_type: &EnumDescriptorProto) {
    if context.config.api.names {
        write_name(context, s, enum_type);
    }
    if context.config.api.field_names {
        write_value_names(context, s, enum_type);
    }
    if context.config.api.serde {
        write_serde(context, s, enum_type);
    }
}

fn write_name(context: &Context, s: &mut TokenStream, enum_type: &EnumDescriptorProto) {
    let enum_ident = context.get_type_expr_path(enum_type.name());
    let enum_proto_name = context.get_proto_type_name(enum_type.name());
    let package_proto_name = &context.package_name;

    s.extend(quote! {
    impl #enum_ident {
        pub const NAME: &'static str = #enum_proto_name;
        pub const PACKAGE: &'static str = #package_proto_name;
    }});
}

fn write_value_names(context: &Context, s: &mut TokenStream, enum_type: &EnumDescriptorProto) {
    let enum_ident = context.get_type_expr_path(enum_type.name());

    let mut value_names = TokenStream::new();
    let mut value_names_array = TokenStream::new();

    for value in &enum_type.value {
        let value_name_ident =
            format_ident!("{}_VALUE_NAME", str_to_case(value.name(), Case::Constant));
        let value_name = value.name();

        value_names.extend(quote! {
            pub const #value_name_ident: &'static str = #value_name;
        });

        value_names_array.extend(quote! {
            Self::#value_name_ident,
        });
    }

    s.extend(quote! {
        impl #enum_ident {
            #value_names

            pub const VALUE_NAMES: &'static [&'static str] = &[#value_names_array];
        }
    });
}

fn write_serde(context: &Context, s: &mut TokenStream, enum_type: &EnumDescriptorProto) {
    let enum_ident = context.get_type_expr_path(enum_type.name());

    // Serialize as string
    s.extend(quote! {
        impl ::serde::Serialize for #enum_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(self.as_str_name())
            }
        }
    });

    // Deserialize from string
    s.extend(quote! {
        impl<'de> ::serde::Deserialize<'de> for #enum_ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = #enum_ident;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", #enum_ident::VALUE_NAMES)
                    }

                    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        i32::try_from(v)
                            .ok()
                            .and_then(|v| #enum_ident::try_from(v).ok())
                            .ok_or_else(|| {
                                ::serde::de::Error::invalid_value(::serde::de::Unexpected::Signed(v), &self)
                            })
                    }

                    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        i32::try_from(v)
                            .ok()
                            .and_then(|v| #enum_ident::try_from(v).ok())
                            .ok_or_else(|| {
                                ::serde::de::Error::invalid_value(::serde::de::Unexpected::Unsigned(v), &self)
                            })
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        #enum_ident::from_str_name(value)
                            .ok_or_else(|| ::serde::de::Error::unknown_variant(value, #enum_ident::VALUE_NAMES))
                    }
                }
                deserializer.deserialize_any(Visitor)
            }
        }
    });
}
