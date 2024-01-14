//! # Wasm macro
//!
//! This macro generates TypeScript WASM bindings for Rust types.
//! Based on [1] with extra features.
//!
//! [1]: https://github.com/madonoharu/tsify

use std::collections::BTreeSet;

use bomboni_core::string::{str_to_case, Case};
use bomboni_wasm_core::{
    options::WasmOptions,
    ts_decl::{TsDecl, TsDeclParser},
};
use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};
use syn::{self, parse_quote, DeriveInput};

pub fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let options = WasmOptions::from_derive_input(&input)?;

    let ident = options.ident();
    let (impl_generics, type_generics, where_clause) = options.generics().split_for_impl();

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
    if options.wasm_ref {
        wasm_abi.extend(expand_wasm_ref(&options));
    }

    if !wasm_abi.is_empty() {
        wasm_abi.extend(quote! {
            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(typescript_type = #ts_decl_name)]
                pub type JsType;
            }

            impl #impl_generics Wasm for #ident #type_generics #where_clause {
                type JsType = JsType;
            }

            impl #impl_generics WasmDescribe for #ident #type_generics #where_clause {
                #[inline]
                fn describe() {
                    <Self as Wasm>::JsType::describe()
                }
            }
        });
    }

    let enum_js = if options.as_enum {
        expand_enum_js(&options, &ts_decl)?
    } else {
        quote!()
    };

    let wasm_mod = options
        .wasm_bindgen
        .as_ref()
        .map_or_else(|| quote!(wasm_bindgen), ToTokens::to_token_stream);
    let mut usage = quote! {
        use #wasm_mod::{
            prelude::*,
            convert::{IntoWasmAbi, FromWasmAbi, OptionIntoWasmAbi, OptionFromWasmAbi, RefFromWasmAbi},
            describe::WasmDescribe,
        };
    };

    usage.extend(if let Some(path) = options.wasm_bindgen.as_ref() {
        quote! {
            use #path as _wasm_bindgen;
        }
    } else {
        quote! {
            extern crate wasm_bindgen as _wasm_bindgen;
        }
    });
    usage.extend(
        if let Some(path) = options.serde_attrs().custom_serde_path() {
            quote! {
                use #path as _serde;
            }
        } else {
            quote! {
                extern crate serde as _serde;
            }
        },
    );
    if let Some(bomboni_mod) = options.bomboni_wasm.as_ref() {
        usage.extend(quote! {
            use #bomboni_mod::Wasm;
        });
    }

    Ok(quote! {
        #[automatically_derived]
        const _: () = {
            #usage

            #[wasm_bindgen(typescript_custom_section)]
            const TS_APPEND_CONTENT: &'static str = #ts_decl_literal;

            impl #ident #type_generics #where_clause {
                const DECL: &'static str = #ts_decl_literal;
            }

            #wasm_abi
            #enum_js
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
        #[automatically_derived]
        impl #impl_generics IntoWasmAbi for #ident #type_generics #where_clause {
            type Abi = <JsType as IntoWasmAbi>::Abi;

            #[inline]
            fn into_abi(self) -> Self::Abi {
                self.to_js().unwrap_throw().into_abi()
            }
        }

        #[automatically_derived]
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
        #[automatically_derived]
        impl #impl_generics FromWasmAbi for #ident #type_generics #where_clause {
            type Abi = <JsType as FromWasmAbi>::Abi;

            #[inline]
            unsafe fn from_abi(js: Self::Abi) -> Self {
                match Self::from_js(&JsType::from_abi(js)) {
                    Ok(value) => value,
                    Err(err) => {
                        _wasm_bindgen::throw_str(&err.to_string());
                        #[allow(unreachable_code)]
                        core::hint::unreachable_unchecked()
                    }
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics OptionFromWasmAbi for #ident #type_generics #where_clause {
            #[inline]
            fn is_none(js: &Self::Abi) -> bool {
                <JsType as OptionFromWasmAbi>::is_none(js)
            }
        }
    }
}

fn expand_wasm_ref(options: &WasmOptions) -> TokenStream {
    let ident = options.ident();
    let serde_path = options.serde_attrs().serde_path();
    let mut generics = options.generics().clone();
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(Self: #serde_path::de::DeserializeOwned));

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics RefFromWasmAbi for #ident #type_generics #where_clause {
            type Abi = <JsType as FromWasmAbi>::Abi;
            type Anchor = core::mem::ManuallyDrop<#ident>;

            #[inline]
            unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                let js_value = <JsValue as RefFromWasmAbi>::ref_from_abi(js);
                match Self::from_js(core::mem::ManuallyDrop::into_inner(js_value)) {
                    Ok(value) => core::mem::ManuallyDrop::new(value),
                    Err(err) => {
                        _wasm_bindgen::throw_str(&err.to_string());
                        #[allow(unreachable_code)]
                        core::hint::unreachable_unchecked()
                    }
                }
            }
        }
    }
}

fn expand_enum_js(options: &WasmOptions, ts_decl: &TsDecl) -> syn::Result<TokenStream> {
    let mut variants = String::new();
    let ts_enum = if let TsDecl::Enum(ts_enum) = ts_decl {
        if !ts_enum.as_enum {
            return Ok(quote!());
        }
        ts_enum
    } else {
        return Ok(quote!());
    };

    let mut unique_member_names = BTreeSet::new();
    for member in &ts_enum.members {
        let member_name = str_to_case(&member.name, Case::Pascal);
        let member_type_value = member.alias_type.to_string();
        if !unique_member_names.insert(member_name.clone())
            || !unique_member_names.insert(member_type_value.clone())
        {
            return Err(syn::Error::new_spanned(
                &options.serde_container.ident,
                format!("duplicate enum member name: {member_name}"),
            ));
        }

        variants.push_str(&format!("{member_name}: {member_type_value},\n"));
        variants.push_str(&format!("{member_type_value}: \"{member_name}\",\n"));
    }

    let js_literal = Literal::string(&format!(
        "export const {} = Object.freeze({{\n  {}}});",
        ts_decl.name(),
        variants,
    ));
    Ok(quote! {
        #[wasm_bindgen(inline_js = #js_literal)]
        extern "C" {}
    })
}
