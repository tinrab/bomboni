use darling::FromMeta;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, ToTokens};

use crate::parse::{ParseOptions, ParseTaggedUnion, ParseVariant};
use crate::utility::{get_proto_type_info, ProtoTypeInfo};

mod parse;
mod write;

pub fn expand(options: &ParseOptions, variants: &[ParseVariant]) -> syn::Result<TokenStream> {
    let mut result = parse::expand(options, variants)?;
    if options.write {
        result.extend(write::expand(options, variants)?);
    }
    Ok(result)
}
