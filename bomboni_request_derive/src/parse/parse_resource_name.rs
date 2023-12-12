use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Token, Type};

use crate::utility::is_option_type;

#[derive(Debug)]
pub struct ParseResourceName {
    segments: Vec<Segment>,
}

#[derive(Debug)]
struct Segment {
    name: Literal,
    ty: Type,
}

pub fn expand(options: ParseResourceName) -> syn::Result<TokenStream> {
    let mut parse_segments = quote!();
    let mut had_optional = false;
    for segment in &options.segments {
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
                return Err(syn::Error::new_spanned(
                    &segment.name,
                    "only ending segments can be optional",
                ));
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

    Ok(quote! {
        |name: &str| {
            if name.is_empty() {
                return None;
            }
            let segments = name.trim().split('/').collect::<Vec<_>>();
            let mut segments_iter = segments.into_iter().peekable();
            let result = (#parse_segments);
            // No extra segments allowed.
            if segments_iter.next().is_some() {
                return None;
            }
            Some(result)
        }
    })
}

impl Parse for ParseResourceName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::braced!(content in input);
        let segments: Punctuated<Segment, Token![,]> =
            content.parse_terminated(Segment::parse, Token![,])?;
        Ok(Self {
            segments: segments.into_iter().collect(),
        })
    }
}

impl Parse for Segment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let _: Token![:] = input.parse()?;
        let ty = input.parse()?;
        Ok(Self { name, ty })
    }
}
