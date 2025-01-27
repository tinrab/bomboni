use darling::ast::Data;
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
    let options = ParseOptions::parse(&input)?;

    let mut result = expand_usage(&options);

    result.extend(match &options.data {
        Data::Struct(data) => message::expand(&options, &data.fields)?,
        Data::Enum(data) => oneof::expand(&options, data)?,
    });

    result.extend(parse_serde::expand(&options)?);

    Ok(quote! {
        #[doc(hidden)]
        #[allow(unused_imports, unused_must_use, non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _ : () = {
            #result
        };
    })
}

fn expand_usage(options: &ParseOptions) -> TokenStream {
    let mut result = quote!();

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

    result.extend(if let Some(path) = options.prost_crate.as_ref() {
        quote! {
            use #path as _prost;
        }
    } else {
        quote! {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate prost as _prost;
        }
    });

    let (mut use_proto, mut use_request) = if let Some(path) = options.bomboni_crate.as_ref() {
        (quote!(#path::proto), quote!(#path::request))
    } else if cfg!(feature = "root-crate") {
        (quote!(bomboni::proto), quote!(bomboni::request))
    } else {
        (quote!(bomboni_proto), quote!(bomboni_request))
    };

    if let Some(path) = options.bomboni_proto_crate.as_ref() {
        use_proto = quote!(#path);
    }
    if let Some(path) = options.bomboni_request_crate.as_ref() {
        use_request = quote!(#path);
    }

    result.extend(quote! {
        use #use_proto::google::protobuf::{
            BoolValue, DoubleValue, FloatValue, Int32Value, Int64Value, StringValue, Timestamp,
            UInt32Value, UInt64Value,
        };
        use #use_request::{
            error::{CommonError, PathError, PathErrorStep, RequestError, RequestResult},
            filter::Filter,
            ordering::{Ordering, OrderingDirection, OrderingTerm},
            query::{
                list::{ListQuery, ListQueryBuilder, ListQueryConfig},
                page_token::{plain::PlainPageTokenBuilder, FilterPageToken, PageTokenBuilder},
                search::{SearchQuery, SearchQueryBuilder, SearchQueryConfig},
            },
            parse::{RequestParse, RequestParseInto},
        };
        use _prost::Name;
    });

    result
}
