//! # Wasm macro
//!
//! This macro generates TypeScript WASM bindings for Rust types.
//! Based on [1] with extra features.
//!
//! [1]: https://github.com/madonoharu/tsify

use std::collections::BTreeSet;

use bomboni_core::string::{str_to_case, Case};
use bomboni_wasm_core::{
    options::{AsStringWasm, ProxyWasm, WasmOptions},
    ts_decl::{TsDecl, TsDeclParser},
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{self, parse_quote, DeriveInput};

pub fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let options = WasmOptions::from_derive_input(&input)?;

    if let Some(proxy) = options.proxy.as_ref() {
        return Ok(derive_proxy(proxy, &options));
    }
    if let Some(as_string) = options.as_string.as_ref() {
        return Ok(derive_as_string(as_string, &options));
    }

    let mut wasm_abi = quote!();
    if options.into_wasm_abi {
        wasm_abi.extend(expand_into_wasm_abi(&options));
    }
    if options.from_wasm_abi {
        wasm_abi.extend(expand_from_wasm_abi(&options));
    }

    let ident = options.ident();
    let ts_decl = TsDeclParser::new(&options).parse();
    let ts_decl_literal = ts_decl.to_string();
    let ts_decl_name = ts_decl.name();

    let (impl_generics, type_generics, where_clause) = options.generics().split_for_impl();

    if !wasm_abi.is_empty() {
        wasm_abi.extend(quote! {
            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(typescript_type = #ts_decl_name)]
                pub type JsType;
            }

            #[automatically_derived]
            impl #impl_generics Wasm for #ident #type_generics #where_clause {
                type JsType = JsType;
            }

            #[automatically_derived]
            impl #impl_generics WasmDescribe for #ident #type_generics #where_clause {
                #[inline]
                fn describe() {
                    <Self as Wasm>::JsType::describe()
                }
            }

            #[automatically_derived]
            impl #impl_generics WasmDescribeVector for #ident #type_generics #where_clause {
                #[inline]
                fn describe_vector() {
                    <Self as Wasm>::JsType::describe_vector()
                }
            }
        });
    }

    let enum_js = if options.as_enum {
        expand_enum_js(&options, &ts_decl)?
    } else {
        quote!()
    };
    let usage = expand_usage(&options);
    Ok(quote! {
        #[automatically_derived]
        const _: () = {
            #usage

            #[wasm_bindgen(typescript_custom_section)]
            const TS_APPEND_CONTENT: &'static str = #ts_decl_literal;

            #[automatically_derived]
            impl #impl_generics #ident #type_generics #where_clause {
                const DECL: &'static str = #ts_decl_literal;
            }

            #wasm_abi
            #enum_js
        };
    })
}

fn derive_proxy(proxy: &ProxyWasm, options: &WasmOptions) -> TokenStream {
    let ident = options.ident();
    let proxy_ident = &proxy.proxy;

    let (impl_generics, type_generics, where_clause) = options.generics().split_for_impl();

    let mut result = if options.into_wasm_abi {
        quote! {
            impl #impl_generics WasmDescribe for #ident #type_generics #where_clause {
                fn describe() {
                    <#proxy_ident as WasmDescribe>::describe()
                }
            }

            #[automatically_derived]
            impl #impl_generics WasmDescribeVector for #ident #type_generics #where_clause {
                #[inline]
                fn describe_vector() {
                    <#proxy_ident as WasmDescribeVector>::describe_vector()
                }
            }
        }
    } else {
        quote!()
    };

    let proxy_try_from = if let Some(try_from) = proxy.try_from.clone() {
        try_from
    } else {
        parse_quote!(TryFrom::try_from)
    };

    if options.into_wasm_abi {
        let proxy_into = if let Some(into) = proxy.into.clone() {
            into
        } else {
            parse_quote!(Into::into)
        };

        result.extend(quote! {
            #[automatically_derived]
            impl #impl_generics From<#ident #type_generics> for JsValue #where_clause {
                #[inline]
                fn from(value: #ident #type_generics) -> Self {
                    let proxy: #proxy_ident = #proxy_into(value);
                    proxy.to_js().unwrap_throw().into()
                }
            }

            #[automatically_derived]
            impl #impl_generics IntoWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as IntoWasmAbi>::Abi;

                fn into_abi(self) -> Self::Abi {
                    let proxy: #proxy_ident = #proxy_into(self);
                    proxy.into_abi()
                }
            }

            #[automatically_derived]
            impl #impl_generics OptionIntoWasmAbi for #ident #type_generics #where_clause {
                #[inline]
                fn none() -> Self::Abi {
                    <#proxy_ident as OptionIntoWasmAbi>::none()
                }
            }

            #[automatically_derived]
            impl VectorIntoWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as VectorIntoWasmAbi>::Abi;

                #[inline]
                fn vector_into_abi(vector: _wasm_bindgen::__rt::std::boxed::Box<[#ident #type_generics]>) -> Self::Abi {
                    let values: Box<[#proxy_ident]> = vector.into_vec()
                        .into_iter()
                        .map(|e| #proxy_into(e))
                        .collect();
                    values.into_abi()
                }
            }
        });
    }

    if options.from_wasm_abi {
        result.extend(quote! {
            #[automatically_derived]
            impl #impl_generics TryFromJsValue for #ident #type_generics #where_clause {
                type Error = JsValue;

                #[inline]
                fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
                    Ok(#proxy_try_from(#proxy_ident::from_js(value)?).unwrap_throw())
                }
            }

            #[automatically_derived]
            impl #impl_generics FromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as FromWasmAbi>::Abi;

                #[inline]
                unsafe fn from_abi(js: Self::Abi) -> Self {
                    #proxy_try_from(#proxy_ident::from_abi(js)).unwrap_throw()
                }
            }

            #[automatically_derived]
            impl #impl_generics OptionFromWasmAbi for #ident #type_generics #where_clause {
                #[inline]
                fn is_none(js: &Self::Abi) -> bool {
                    <#proxy_ident as OptionFromWasmAbi>::is_none(js)
                }
            }

            #[automatically_derived]
            impl VectorFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as VectorFromWasmAbi>::Abi;

                #[inline]
                unsafe fn vector_from_abi(js: Self::Abi) -> _wasm_bindgen::__rt::std::boxed::Box<[#ident #type_generics]> {
                    _wasm_bindgen::convert::js_value_vector_from_abi(js)
                }
            }

            #[automatically_derived]
            impl #impl_generics RefFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as RefFromWasmAbi>::Abi;
                type Anchor = core::mem::ManuallyDrop<#ident #type_generics>;

                #[inline]
                unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                    let proxy_value = <#proxy_ident as RefFromWasmAbi>::ref_from_abi(js);
                    core::mem::ManuallyDrop::new(
                        #proxy_try_from(core::mem::ManuallyDrop::into_inner(proxy_value))
                            .unwrap_throw()
                    )
                }
            }

            impl #impl_generics LongRefFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as LongRefFromWasmAbi>::Abi;
                type Anchor = #ident #type_generics;

                #[inline]
                unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
                    let proxy_value = <#proxy_ident as LongRefFromWasmAbi>::long_ref_from_abi(js);
                    #proxy_try_from(proxy_value).unwrap_throw()
                }
            }
        });
    }

    let usage = expand_usage(options);
    quote! {
        #[automatically_derived]
        const _: () = {
            #usage
            #result
        };
    }
}

fn derive_as_string(as_string: &AsStringWasm, options: &WasmOptions) -> TokenStream {
    let ident = options.ident();
    let (impl_generics, type_generics, where_clause) = options.generics().split_for_impl();

    let try_from = if let Some(try_from) = as_string.try_from.clone() {
        try_from
    } else {
        parse_quote!(FromStr::from_str)
    };
    let into = if let Some(into) = as_string.into.clone() {
        into.to_token_stream()
    } else {
        quote!((|item: #ident #type_generics| { item.to_string() }))
    };

    let type_name = options.name();
    let type_len = type_name.len() as u32;
    let type_chars = type_name.chars().map(|c| c as u32);

    let type_decl_literal = format!("export type {type_name} = string;");
    let unexpected_error = format!("expected `{type_name}`");

    let usage = expand_usage(options);

    quote! {
        #[automatically_derived]
        const _: () = {
            #usage

            #[automatically_derived]
            impl #impl_generics WasmDescribe for #ident #type_generics #where_clause {
                #[inline]
                fn describe() {
                    use wasm_bindgen::describe::*;
                    inform(NAMED_EXTERNREF);
                    inform(#type_len);
                    #(inform(#type_chars);)*
                }
            }

            #[automatically_derived]
            impl #impl_generics WasmDescribeVector for #ident #type_generics #where_clause {
                #[inline]
                fn describe_vector() {
                    use wasm_bindgen::describe::*;
                    inform(VECTOR);
                    <#ident #type_generics as WasmDescribe>::describe();
                }
            }

            #[automatically_derived]
            impl #impl_generics From<#ident #type_generics> for js_sys::JsString {
                #[inline]
                fn from(value: #ident #type_generics) -> Self {
                    #into(value).into()
                }
            }

            #[automatically_derived]
            impl #impl_generics From<js_sys::JsString> for #ident #type_generics #where_clause {
                #[inline]
                fn from(value: js_sys::JsString) -> Self {
                    #try_from(&value.as_string().unwrap()).unwrap_throw()
                }
            }

            #[automatically_derived]
            impl #impl_generics From<&js_sys::JsString> for #ident #type_generics #where_clause {
                #[inline]
                fn from(value: &js_sys::JsString) -> Self {
                    #try_from(&value.as_string().unwrap()).unwrap_throw()
                }
            }

            #[automatically_derived]
            impl #impl_generics From<#ident #type_generics> for JsValue #where_clause {
                #[inline]
                fn from(value: #ident #type_generics) -> Self {
                    #into(value).into()
                }
            }

            #[automatically_derived]
            impl #impl_generics TryFromJsValue for #ident #type_generics #where_clause {
                type Error = JsValue;

                #[inline]
                fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
                    let s = <String as TryFromJsValue>::try_from_js_value(value)?;
                    Ok(#try_from(&s).unwrap_throw())
                }
            }

            #[automatically_derived]
            impl #impl_generics IntoWasmAbi for #ident #type_generics #where_clause {
                type Abi = <js_sys::JsString as IntoWasmAbi>::Abi;

                #[inline]
                fn into_abi(self) -> Self::Abi {
                    js_sys::JsString::from(#into(self)).into_abi()
                }
            }

            #[automatically_derived]
            impl VectorIntoWasmAbi for #ident #type_generics #where_clause {
                type Abi = <_wasm_bindgen::__rt::std::boxed::Box<[js_sys::JsString]> as IntoWasmAbi>::Abi;

                #[inline]
                fn vector_into_abi(vector: _wasm_bindgen::__rt::std::boxed::Box<[#ident #type_generics]>) -> Self::Abi {
                    _wasm_bindgen::convert::js_value_vector_into_abi(vector)
                }
            }

            #[automatically_derived]
            impl #impl_generics OptionIntoWasmAbi for #ident #type_generics #where_clause {
                #[inline]
                fn none() -> Self::Abi {
                    <js_sys::JsString as OptionIntoWasmAbi>::none()
                }
            }

            #[automatically_derived]
            impl #impl_generics FromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <js_sys::JsString as FromWasmAbi>::Abi;

                #[inline]
                unsafe fn from_abi(js: Self::Abi) -> Self {
                    match js_sys::JsString::from_abi(js)
                        .as_string()
                        .as_ref()
                        .map(|s| #try_from(s))
                    {
                        Some(result) => result.unwrap_throw(),
                        None => {
                            wasm_bindgen::throw_str(#unexpected_error);
                        }
                    }
                }
            }

            #[automatically_derived]
            impl VectorFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <_wasm_bindgen::__rt::std::boxed::Box<[JsValue]> as FromWasmAbi>::Abi;

                #[inline]
                unsafe fn vector_from_abi(js: Self::Abi) -> _wasm_bindgen::__rt::std::boxed::Box<[#ident #type_generics]> {
                    _wasm_bindgen::convert::js_value_vector_from_abi(js)
                }
            }

            #[automatically_derived]
            impl #impl_generics OptionFromWasmAbi for #ident #type_generics #where_clause {
                #[inline]
                fn is_none(js: &Self::Abi) -> bool {
                    js_sys::JsString::is_none(js)
                }
            }

            #[automatically_derived]
            impl #impl_generics RefFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <JsValue as RefFromWasmAbi>::Abi;
                type Anchor = core::mem::ManuallyDrop<#ident #type_generics>;

                #[inline]
                unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                    let js_value = <JsValue as RefFromWasmAbi>::ref_from_abi(js);
                    core::mem::ManuallyDrop::new(
                        #try_from(&js_value.as_string().unwrap_throw()).unwrap_throw()
                    )
                }
            }

            impl #impl_generics LongRefFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <JsValue as LongRefFromWasmAbi>::Abi;
                type Anchor = #ident #type_generics;

                #[inline]
                unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
                    let js_value = <JsValue as LongRefFromWasmAbi>::long_ref_from_abi(js);
                    #try_from(&js_value.as_string().unwrap_throw()).unwrap_throw()
                }
            }

            #[wasm_bindgen(typescript_custom_section)]
            const TS_APPEND_CONTENT: &'static str = #type_decl_literal;
        };
    }
}

fn expand_into_wasm_abi(options: &WasmOptions) -> TokenStream {
    let ident = options.ident();
    let (impl_generics, type_generics, where_clause) = options.generics().split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics From<#ident #type_generics> for JsValue #where_clause {
            #[inline]
            fn from(value: #ident #type_generics) -> Self {
                value.to_js().unwrap().into()
            }
        }

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

        #[automatically_derived]
        impl VectorIntoWasmAbi for #ident #type_generics #where_clause {
            type Abi = <_wasm_bindgen::__rt::std::boxed::Box<[JsValue]> as IntoWasmAbi>::Abi;

            #[inline]
            fn vector_into_abi(vector: _wasm_bindgen::__rt::std::boxed::Box<[#ident #type_generics]>) -> Self::Abi {
                _wasm_bindgen::convert::js_value_vector_into_abi(vector)
            }
        }
    }
}

fn expand_from_wasm_abi(options: &WasmOptions) -> TokenStream {
    let ident = options.ident();
    let (impl_generics, type_generics, where_clause) = options.generics().split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics TryFromJsValue for #ident #type_generics #where_clause {
            type Error = JsValue;

            #[inline]
            fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
                Ok(Self::from_js(value)?)
            }
        }

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

        #[automatically_derived]
        impl VectorFromWasmAbi for #ident #type_generics #where_clause {
            type Abi = <_wasm_bindgen::__rt::std::boxed::Box<[JsValue]> as FromWasmAbi>::Abi;

            #[inline]
            unsafe fn vector_from_abi(js: Self::Abi) -> _wasm_bindgen::__rt::std::boxed::Box<[#ident #type_generics]> {
                _wasm_bindgen::convert::js_value_vector_from_abi(js)
            }
        }

        #[automatically_derived]
        impl #impl_generics RefFromWasmAbi for #ident #type_generics #where_clause {
            type Abi = <JsType as FromWasmAbi>::Abi;
            type Anchor = core::mem::ManuallyDrop<#ident #type_generics>;

            #[inline]
            unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                let js_value = <JsValue as RefFromWasmAbi>::ref_from_abi(js);
                core::mem::ManuallyDrop::new(
                    Self::from_js(core::mem::ManuallyDrop::into_inner(js_value))
                        .unwrap_throw()
                )
            }
        }

        impl #impl_generics LongRefFromWasmAbi for #ident #type_generics #where_clause {
            type Abi = <JsType as LongRefFromWasmAbi>::Abi;
            type Anchor = #ident #type_generics;

            #[inline]
            unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
                let js_value = <JsType as LongRefFromWasmAbi>::long_ref_from_abi(js);
                Self::from_js(js_value).unwrap_throw()
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

    let js_literal = format!(
        "export const {} = Object.freeze({{\n  {}}});",
        ts_decl.name(),
        variants,
    );
    Ok(quote! {
        #[wasm_bindgen(inline_js = #js_literal)]
        extern "C" {}
    })
}

fn expand_usage(options: &WasmOptions) -> TokenStream {
    let wasm_mod = options
        .wasm_bindgen
        .as_ref()
        .map_or_else(|| quote!(wasm_bindgen), ToTokens::to_token_stream);
    let mut result = quote! {
        use #wasm_mod::{
            prelude::*,
            convert::{
                IntoWasmAbi, FromWasmAbi, OptionIntoWasmAbi, OptionFromWasmAbi, RefFromWasmAbi, LongRefFromWasmAbi,
                TryFromJsValue, VectorFromWasmAbi, VectorIntoWasmAbi,
            },
            describe::{WasmDescribe, WasmDescribeVector},
            JsObject,
        };
    };

    result.extend(if let Some(path) = options.wasm_bindgen.as_ref() {
        quote! {
            use #path as _wasm_bindgen;
        }
    } else {
        quote! {
            extern crate wasm_bindgen as _wasm_bindgen;
        }
    });

    result.extend(
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
        result.extend(quote! {
            use #bomboni_mod::Wasm;
        });
    }

    result
}
