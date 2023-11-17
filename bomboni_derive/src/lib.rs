//! # A procedural macro crate for the Bomboni library.

use proc_macro::TokenStream;
use proc_macro2::{Literal, Span};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::{token, Token, Type};

use crate::utility::is_option_type;

mod utility;

#[derive(Debug)]
struct Resource {
    _bracket_token: token::Bracket,
    segments: Punctuated<Segment, Token![,]>,
}

#[derive(Debug)]
struct Segment {
    span: Span,
    name: Literal,
    _arrow_token: Token![=>],
    ty: Type,
}

/// A procedural macro that generates a function that parses a resource name into a tuple of typed segments.
/// Resource name format is documented in Google's AIP [1].
/// Ending segments can also be optional [2].
///
/// [1]: https://google.aip.dev/122
/// [2]: https://google.aip.dev/162
///
/// # Examples
///
/// ```
/// # use bomboni_derive::parse_resource_name;
/// let name = "users/42/projects/1337";
/// let parsed = parse_resource_name!([
///    "users" => u32,
///    "projects" => u64,
/// ])(name);
/// assert_eq!(parsed, Some((42, 1337)));
/// ```
#[proc_macro]
pub fn parse_resource_name(input: TokenStream) -> TokenStream {
    let resource = parse_macro_input!(input as Resource);

    let mut parse_segments = quote!();
    let mut had_optional = false;
    for segment in &resource.segments {
        let name = &segment.name;
        let ty = &segment.ty;
        if is_option_type(ty) {
            had_optional = true;
            parse_segments.extend(quote! {{
                if segments_iter.peek() == Some(&#name) {
                    segments_iter.next()?;
                    segments_iter.next().map(|e| e.parse().ok()).flatten()
                } else {
                    None
                }
            }});
        } else {
            if had_optional {
                return syn::Error::new(segment.span, "only ending segments can be optional")
                    .to_compile_error()
                    .into();
            }
            parse_segments.extend(quote! {{
                if segments_iter.next()? != #name {
                    return None;
                }
                segments_iter.next()?.parse::<#ty>().ok()?
            }});
        }
        parse_segments.extend(quote! {,});
    }

    quote! {
        |name: &str| {
            let segments = name.trim().split('/').collect::<Vec<_>>();
            let mut segments_iter = segments.into_iter().peekable();
            let result = (#parse_segments);
            // No extra segments allowed.
            if segments_iter.next().is_some() {
                return None;
            }
            Some(result)
        }
    }
    .into()
}

impl Parse for Resource {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            _bracket_token: syn::bracketed!(content in input),
            segments: content.parse_terminated(Segment::parse, Token![,])?,
        })
    }
}

impl Parse for Segment {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            span: input.span(),
            name: input.parse()?,
            _arrow_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}
