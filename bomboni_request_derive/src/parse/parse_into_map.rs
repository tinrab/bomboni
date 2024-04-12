use std::ops::Deref;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
    ExprClosure, GenericArgument, Pat, PatType, Path, PathArguments, ReturnType, Token, Type,
    TypePath, TypeTuple, Visibility,
};

#[derive(Debug)]
pub struct ParseIntoMap {
    vis: Visibility,
    ident: Ident,
    parse_item_closure: ExprClosure,
    write_item_closure: Option<ExprClosure>,
    map_type: Option<Type>,
}

pub fn expand(options: ParseIntoMap) -> syn::Result<TokenStream> {
    let ParseIntoMap {
        vis,
        ident,
        parse_item_closure,
        write_item_closure,
        map_type,
    } = options;

    let source_item_type = if let Some(Pat::Type(pat_type)) = parse_item_closure.inputs.first() {
        &pat_type.ty
    } else if let Some(ReturnType::Type(_, return_type)) =
        write_item_closure.as_ref().map(|c| &c.output)
    {
        return_type
    } else {
        return Err(syn::Error::new_spanned(
            &parse_item_closure.inputs,
            "cannot determine source item type from parse or write closures",
        ));
    };

    let mut is_request_result = false;
    let mut parse_return_type = None;
    if let ReturnType::Type(_, return_type) = &parse_item_closure.output {
        if let Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) = &**return_type
        {
            if let Some(syn::PathSegment {
                ident,
                arguments: PathArguments::AngleBracketed(args),
            }) = segments.first()
            {
                if let Some(GenericArgument::Type(ty)) = args.args.first() {
                    if ident == "RequestResult" {
                        is_request_result = true;
                        parse_return_type = Some(ty.clone());
                    }
                }
            }
        }
        if parse_return_type.is_none() {
            parse_return_type = Some(return_type.deref().clone());
        }
    }
    let parse_return_type = if let Some(parse_return_type) = parse_return_type {
        parse_return_type
    } else if let Some(Pat::Type(PatType { ty, .. })) =
        write_item_closure.as_ref().and_then(|c| c.inputs.first())
    {
        ty.deref().clone()
    } else {
        return Err(syn::Error::new_spanned(
            &parse_item_closure.output,
            "cannot determine parse return type from parse or write closures",
        ));
    };
    let (key_type, value_type) = if let Type::Tuple(TypeTuple { ref elems, .. }) = parse_return_type
    {
        if elems.len() != 2 {
            return Err(syn::Error::new_spanned(
                &parse_return_type,
                "expected key-value tuple type",
            ));
        }
        (&elems[0], &elems[1])
    } else {
        return Err(syn::Error::new_spanned(
            &parse_return_type,
            "expected key-value tuple type",
        ));
    };

    let map_type = if let Some(map_type) = map_type {
        map_type.into_token_stream()
    } else {
        quote! { ::std::collections::BTreeMap }
    };

    let parse_body = &parse_item_closure.body;
    let parse_expr = if is_request_result {
        quote! {
            { #parse_body }.map_err(|err: RequestError| err)?
        }
    } else {
        quote! {
            #parse_body
        }
    };

    let write_fn = if let Some(write_item_closure) = write_item_closure.as_ref() {
        let write_body = &write_item_closure.body;
        quote! {
            pub fn write<I>(values: I) -> Vec<#source_item_type>
            where
                I: ::std::iter::IntoIterator<Item = (#key_type, #value_type)>,
            {
                values
                    .into_iter()
                    .map(|item| {
                        #[allow(unused_braces)]
                        #write_body
                    })
                    .collect()
            }
        }
    } else {
        quote!()
    };

    Ok(quote! {
        #vis mod #ident {
            use super::*;

            pub fn parse<I>(values: I) -> RequestResult<#map_type<#key_type, #value_type>>
            where
                I: ::std::iter::IntoIterator<Item = #source_item_type>,
            {
                let mut m = #map_type::new();
                for item in values.into_iter() {
                    let (k, v): (#key_type, #value_type) = {
                        #[allow(unused_braces)]
                        #parse_expr
                    };
                    if m.insert(k, v).is_some() {
                        return Err(CommonError::DuplicateValue.into());
                    }
                }
                Ok(m)
            }

            #write_fn
        }
    })
}

impl Parse for ParseIntoMap {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let ident: Ident = input.parse()?;
        let _: Token![,] = input.parse()?;

        let map_type: Option<Type> = input.parse().ok();

        let _: Option<Token![,]> = input.parse()?;
        let parse_item_closure: ExprClosure = input.parse()?;
        let _: Option<Token![,]> = input.parse()?;
        let write_item_closure: Option<ExprClosure> = input.parse().ok();

        // Trailing comma
        let _: Option<Token![,]> = input.parse()?;

        Ok(Self {
            vis,
            ident,
            parse_item_closure,
            write_item_closure,
            map_type,
        })
    }
}
