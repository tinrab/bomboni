use darling::{ast::Data, FromDeriveInput};
use options::ParseOptions;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

mod field_type_info;
mod message;
mod oneof;
mod parse_serde;
mod parse_utility;
mod write_utility;

pub mod derived_map;
pub mod options;
pub mod parse_resource_name;

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let options = ParseOptions::from_derive_input(&input)?;

    let mut result = expand_usage(&options);

    result.extend(match &options.data {
        Data::Struct(data) => message::expand(&options, &data.fields)?,
        Data::Enum(data) => oneof::expand(&options, data)?,
    });

    result.extend(parse_serde::expand(&options)?);

    Ok(quote! {
        #[doc(hidden)]
        #[allow(unused_imports, non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _ : () = {
            #result
        };
    })
}

fn expand_usage(options: &ParseOptions) -> TokenStream {
    let mut result = quote!();

    result.extend(if let Some(path) = options.bomboni_crate.as_ref() {
        quote! {
            use #path as _bomboni;
        }
    } else {
        quote! {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate bomboni as _bomboni;
        }
    });

    result.extend(quote! {
        use _bomboni::{
            proto::google::protobuf::{
                BoolValue, DoubleValue, FloatValue, Int32Value, Int64Value, StringValue, Timestamp,
                UInt32Value, UInt64Value,
            },
            request::{
                error::{CommonError, PathError, PathErrorStep, RequestError, RequestResult},
                filter::Filter,
                ordering::{Ordering, OrderingDirection, OrderingTerm},
                query::{
                    list::{ListQuery, ListQueryBuilder, ListQueryConfig},
                    page_token::{plain::PlainPageTokenBuilder, FilterPageToken, PageTokenBuilder},
                    search::{SearchQuery, SearchQueryBuilder, SearchQueryConfig},
                },
            },
        };
    });

    result.extend(if let Some(path) = options.serde_crate.as_ref() {
        quote! {
            use #path as _serde;
        }
    } else {
        quote! {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
        }
    });

    result
}
