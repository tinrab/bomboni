use darling::FromMeta;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, ToTokens};

use crate::parse::{ParseOptions, ParseTaggedUnion, ParseVariant};
use crate::utility::{get_proto_type_info, ProtoTypeInfo};

use super::ParseField;

mod parse;
mod write;

pub fn expand(options: &ParseOptions, fields: &[ParseField]) -> syn::Result<TokenStream> {
    let mut result = parse::expand(options, fields)?;
    if options.write {
        result.extend(write::expand(options, fields));
    }
    Ok(result)
}
