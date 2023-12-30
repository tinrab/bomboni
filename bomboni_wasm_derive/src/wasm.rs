//! # WasmTypescript macro
//!
//! This macro generates TypeScript WASM bindings for Rust types.
//! Based on [1] with extra features.
//!
//! [1]: https://github.com/madonoharu/tsify

use bomboni_core::string::{str_to_case, Case};
use bomboni_wasm_core::{
    options::WasmOptions,
    ts_decl::{self, TsDeclParser},
};
use darling::{FromDeriveInput, FromField};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};
use serde_derive_internals::{
    ast::{self, Container as SerdeContainer},
    attr, Ctxt,
};
use syn::{self, parse_quote, DeriveInput, Generics, Meta, MetaList, Path, Type};

pub fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let options = WasmOptions::from_derive_input(&input)?;

    let ident = options.ident();
    let (impl_generics, type_generics, where_clause) = options.generics().split_for_impl();
    let wasm_mod = options
        .wasm_bindgen
        .as_ref()
        .map(|path| path.to_token_stream())
        .unwrap_or_else(|| quote!(wasm_bindgen));

    let ts_decl = TsDeclParser::new(&options).parse();
    let ts_decl_literal = Literal::string(&ts_decl.to_string());
    let ts_decl_name = Literal::string(ts_decl.name());

    let mut wasm_abi = quote!();
    if options.into_wasm_abi {
        wasm_abi.extend(expand_into_wasm_abi(&options));
    }
    if options.from_wasm_abi {
        wasm_abi.extend(expand_from_wasm_abi(&options));
    }

    let wasm_proxy = if !wasm_abi.is_empty() {
        quote! {
            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(typescript_type = #ts_decl_name)]
                pub type JsType;
            }

            impl #impl_generics Wasm for #ident #type_generics #where_clause {
                type JsType = JsType;
                const DECL: &'static str = #ts_decl_literal;
            }
        }
    } else {
        quote!()
    };

    let wasm_describe = if options.into_wasm_abi || options.from_wasm_abi {
        quote! {
            impl #impl_generics WasmDescribe for #ident #type_generics #where_clause {
                #[inline]
                fn describe() {
                    <Self as Wasm>::JsType::describe()
                }
            }
        }
    } else {
        quote!()
    };

    let use_serde = match options.serde_attrs().custom_serde_path() {
        Some(path) => quote! {
            use #path as _serde;
        },
        None => quote! {
            extern crate serde as _serde;
        },
    };

    Ok(quote! {
        #[automatically_derived]
        const _: () = {
            use #wasm_mod::{
                prelude::*,
                convert::{IntoWasmAbi, FromWasmAbi, OptionIntoWasmAbi, OptionFromWasmAbi},
                describe::WasmDescribe,
            };
            #use_serde

            #[wasm_bindgen(typescript_custom_section)]
            const TS_APPEND_CONTENT: &'static str = #ts_decl_literal;

            #wasm_proxy
            #wasm_abi
            #wasm_describe
        };
    })
}

fn expand_into_wasm_abi(options: &WasmOptions) -> TokenStream {
    let ident = options.ident();
    let serde_path = options.serde_attrs().serde_path();

    let mut generics = options.generics().clone();
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(Self: #serde_path::Serialize));

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics IntoWasmAbi for #ident #type_generics #where_clause {
            type Abi = <JsType as IntoWasmAbi>::Abi;

            #[inline]
            fn into_abi(self) -> Self::Abi {
                self.to_js().unwrap_throw().into_abi()
            }
        }

        impl #impl_generics OptionIntoWasmAbi for #ident #type_generics #where_clause {
            #[inline]
            fn none() -> Self::Abi {
                <JsType as OptionIntoWasmAbi>::none()
            }
        }
    }
}

fn expand_from_wasm_abi(options: &WasmOptions) -> TokenStream {
    let ident = options.ident();
    let serde_path = options.serde_attrs().serde_path();

    let mut generics = options.generics().clone();
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(Self: #serde_path::de::DeserializeOwned));

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics FromWasmAbi for #ident #type_generics #where_clause {
            type Abi = <JsType as FromWasmAbi>::Abi;

            #[inline]
            unsafe fn from_abi(js: Self::Abi) -> Self {
                let result = Self::from_js(&JsType::from_abi(js));
                if let Err(err) = result {
                    wasm_bindgen::throw_str(err.to_string().as_ref());
                }
                result.unwrap_throw()
            }
        }

        impl #impl_generics OptionFromWasmAbi for #ident #type_generics #where_clause {
            #[inline]
            fn is_none(js: &Self::Abi) -> bool {
                <JsType as OptionFromWasmAbi>::is_none(js)
            }
        }
    }
}
