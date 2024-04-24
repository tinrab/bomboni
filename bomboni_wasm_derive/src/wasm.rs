//! # Wasm macro
//!
//! This macro generates TypeScript WASM bindings for Rust types.
//! Based on [1] with extra features.
//!
//! [1]: https://github.com/madonoharu/tsify

use std::collections::BTreeSet;

use bomboni_core::string::{str_to_case, Case};
use bomboni_wasm_core::{
    options::{JsValueWasm, ProxyWasm, WasmOptions},
    ts_decl::{TsDecl, TsDeclParser},
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{self, DeriveInput};

pub fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let options = WasmOptions::from_derive_input(&input)?;

    if let Some(js_value) = options.js_value.as_ref() {
        return Ok(derive_js_value(js_value, &options));
    }

    if options.enum_value {
        return derive_enum_value(&options);
    }

    if let Some(proxy) = options.proxy.as_ref() {
        return Ok(derive_proxy(proxy, &options));
    }

    Ok(derive_serde_wasm(&options))
}

fn derive_serde_wasm(options: &WasmOptions) -> TokenStream {
    let ident = options.ident();
    let (impl_generics, type_generics, where_clause) = options.generics().split_for_impl();

    let ts_decl = TsDeclParser::new(options).parse();
    let ts_decl_literal = ts_decl.to_string();
    let ts_decl_name = ts_decl.name();

    let mut impls = quote! {
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

        #[wasm_bindgen(typescript_custom_section)]
        const TS_APPEND_CONTENT: &'static str = #ts_decl_literal;

        #[automatically_derived]
        impl #impl_generics #ident #type_generics #where_clause {
            const DECL: &'static str = #ts_decl_literal;
        }
    };

    if options.from_wasm_abi {
        let handle_error = expand_wasm_error_handler(ident);

        impls.extend(quote! {
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
                        Err(err) => #handle_error,
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
            impl #impl_generics VectorFromWasmAbi for #ident #type_generics #where_clause {
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
                    let value = <JsValue as RefFromWasmAbi>::ref_from_abi(js);
                    match Self::from_js(core::mem::ManuallyDrop::into_inner(value)) {
                        Ok(value) => core::mem::ManuallyDrop::new(value),
                        Err(err) => #handle_error,
                    }
                }
            }

            impl #impl_generics LongRefFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <JsType as LongRefFromWasmAbi>::Abi;
                type Anchor = #ident #type_generics;

                #[inline]
                unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
                    let value = <JsType as LongRefFromWasmAbi>::long_ref_from_abi(js);
                    match Self::from_js(value) {
                        Ok(value) => value,
                        Err(err) => #handle_error,
                    }
                }
            }
        });
    }

    if options.into_wasm_abi {
        impls.extend(quote! {
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
            impl #impl_generics VectorIntoWasmAbi for #ident #type_generics #where_clause {
                type Abi = <_wasm_bindgen::__rt::std::boxed::Box<[JsValue]> as IntoWasmAbi>::Abi;

                #[inline]
                fn vector_into_abi(vector: _wasm_bindgen::__rt::std::boxed::Box<[#ident #type_generics]>) -> Self::Abi {
                    _wasm_bindgen::convert::js_value_vector_into_abi(vector)
                }
            }
        });
    }

    let usage = expand_usage(options);
    quote! {
        #[automatically_derived]
        const _: () = {
            #usage
            #impls
        };
    }
}

fn derive_js_value(js_value: &JsValueWasm, options: &WasmOptions) -> TokenStream {
    let ident = options.ident();
    let (impl_generics, type_generics, where_clause) = options.generics().split_for_impl();

    let convert_into = if let Some(path) = js_value.into.clone() {
        path.to_token_stream()
    } else {
        quote!(core::convert::Into::into)
    };

    let convert_try_from = if let Some(path) = js_value.try_from.clone() {
        path.to_token_stream()
    } else {
        quote!(core::convert::TryFrom::try_from)
    };

    let type_name = options.name();
    let type_len = type_name.len() as u32;
    let type_chars = type_name.chars().map(|c| c as u32);

    let ts_decl_literal = if let Some(override_type) = options.override_type.as_ref() {
        format!("export type {type_name} = {override_type};")
    } else if js_value.convert_string {
        format!("export type {type_name} = string;")
    } else {
        format!("export type {type_name} = any;")
    };

    let mut impls = quote! {
        #[automatically_derived]
        impl #impl_generics WasmDescribe for #ident #type_generics #where_clause {
            #[inline]
            fn describe() {
                use _wasm_bindgen::describe::*;
                inform(NAMED_EXTERNREF);
                inform(#type_len);
                #(inform(#type_chars);)*
            }
        }

        #[automatically_derived]
        impl #impl_generics WasmDescribeVector for #ident #type_generics #where_clause {
            #[inline]
            fn describe_vector() {
                use _wasm_bindgen::describe::*;
                inform(VECTOR);
                <#ident #type_generics as WasmDescribe>::describe();
            }
        }

        #[wasm_bindgen(typescript_custom_section)]
        const TS_APPEND_CONTENT: &'static str = #ts_decl_literal;

        #[automatically_derived]
        impl #impl_generics #ident #type_generics #where_clause {
            const DECL: &'static str = #ts_decl_literal;
        }
    };

    if js_value.convert_string {
        impls.extend(quote! {
            impl #impl_generics From<#ident #type_generics> for JsValue #where_clause {
                fn from(value: #ident #type_generics) -> Self {
                    JsValue::from_str(&value.to_string())
                }
            }

            impl #impl_generics TryFrom<JsValue> for #ident #type_generics #where_clause {
                type Error = JsValue;

                fn try_from(value: JsValue) -> Result<Self, Self::Error> {
                    match value.as_string().as_ref().map(|s| s.parse()) {
                        Some(Ok(value)) => Ok(value),
                        Some(Err(err)) => Err(js_sys::Error::new(&err.to_string()).into()),
                        None => Err(js_sys::Error::new("expected a string").into()),
                    }
                }
            }
        });
    }

    if options.from_wasm_abi {
        let handle_error = expand_wasm_error_handler(ident);

        impls.extend(quote! {
            #[automatically_derived]
            impl #impl_generics FromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <JsValue as FromWasmAbi>::Abi;

                #[inline]
                unsafe fn from_abi(js: Self::Abi) -> Self {
                    let value = JsValue::from_abi(js);
                    match #convert_try_from(value) {
                        Ok(value) => value,
                        Err(err) => #handle_error,
                    }
                }
            }

            #[automatically_derived]
            impl #impl_generics OptionFromWasmAbi for #ident #type_generics #where_clause {
                #[inline]
                fn is_none(js: &Self::Abi) -> bool {
                    <_js_sys::Object as OptionFromWasmAbi>::is_none(js)
                }
            }

            #[automatically_derived]
            impl #impl_generics VectorFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <_wasm_bindgen::__rt::std::boxed::Box<[JsValue]> as FromWasmAbi>::Abi;

                #[inline]
                unsafe fn vector_from_abi(js: Self::Abi) -> _wasm_bindgen::__rt::std::boxed::Box<[#ident #type_generics]> {
                    let values = <Vec<JsValue> as FromWasmAbi>::from_abi(js);
                    let mut vector = Vec::<#ident #type_generics>::with_capacity(values.len());
                    for value in values.into_iter() {
                        match #convert_try_from(value) {
                            Ok(value) => vector.push(value),
                            Err(err) => #handle_error,
                        }
                    }
                    vector.into_boxed_slice()
                }
            }

            #[automatically_derived]
            impl #impl_generics RefFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <JsValue as RefFromWasmAbi>::Abi;
                type Anchor = core::mem::ManuallyDrop<#ident #type_generics>;

                #[inline]
                unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                    let value = <JsValue as RefFromWasmAbi>::ref_from_abi(js);
                    match #convert_try_from(
                        core::mem::ManuallyDrop::into_inner(value)
                    ) {
                        Ok(value) => core::mem::ManuallyDrop::new(value),
                        Err(err) => #handle_error,
                    }
                }
            }

            impl #impl_generics LongRefFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <JsValue as LongRefFromWasmAbi>::Abi;
                type Anchor = #ident #type_generics;

                #[inline]
                unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
                    let value = <JsValue as LongRefFromWasmAbi>::long_ref_from_abi(js);
                    match #convert_try_from(value) {
                        Ok(value) => value,
                        Err(err) => #handle_error,
                    }
                }
            }
        });
    }

    if options.into_wasm_abi {
        impls.extend(quote! {
            #[automatically_derived]
            impl #impl_generics IntoWasmAbi for #ident #type_generics #where_clause {
                type Abi = <JsValue as IntoWasmAbi>::Abi;

                #[inline]
                fn into_abi(self) -> Self::Abi {
                    let value: JsValue = #convert_into(self);
                    value.into_abi()
                }
            }

            #[automatically_derived]
            impl #impl_generics OptionIntoWasmAbi for #ident #type_generics #where_clause {
                #[inline]
                fn none() -> Self::Abi {
                    <_js_sys::Object as OptionIntoWasmAbi>::none()
                }
            }

            #[automatically_derived]
            impl #impl_generics VectorIntoWasmAbi for #ident #type_generics #where_clause {
                type Abi = <_wasm_bindgen::__rt::std::boxed::Box<[JsValue]> as IntoWasmAbi>::Abi;

                #[inline]
                fn vector_into_abi(vector: _wasm_bindgen::__rt::std::boxed::Box<[#ident #type_generics]>) -> Self::Abi {
                    let js_values: Box<[JsValue]> = vector
                        .into_vec()
                        .into_iter()
                        .map(|value| #convert_into(value))
                        .collect();
                    js_values.into_abi()
                }
            }
        });
    }

    let usage = expand_usage(options);
    quote! {
        #[automatically_derived]
        const _: () = {
            #usage
            #impls
        };
    }
}

fn derive_enum_value(options: &WasmOptions) -> syn::Result<TokenStream> {
    let ts_decl = TsDeclParser::new(options).parse();
    let ts_decl_name = ts_decl.name();

    let mut variants = String::new();
    if let TsDecl::Enum(ts_enum) = &ts_decl {
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
    }

    let js_literal = format!("export const {ts_decl_name} = Object.freeze({{\n  {variants}}});");
    let impls = derive_serde_wasm(options);
    Ok(quote! {
        #[wasm_bindgen(inline_js = #js_literal)]
        extern "C" {}

        #impls
    })
}

fn derive_proxy(proxy: &ProxyWasm, options: &WasmOptions) -> TokenStream {
    let ident = options.ident();
    let proxy_ident = &proxy.proxy;
    let (impl_generics, type_generics, where_clause) = options.generics().split_for_impl();

    let convert_into = if let Some(path) = proxy.into.clone() {
        path.to_token_stream()
    } else {
        quote!(core::convert::Into::into)
    };

    let convert_try_from = if let Some(path) = proxy.try_from.clone() {
        path.to_token_stream()
    } else {
        quote!(core::convert::TryFrom::try_from)
    };

    let mut impls = quote! {
        #[automatically_derived]
        impl #impl_generics WasmDescribe for #ident #type_generics #where_clause {
            #[inline]
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
    };

    if options.from_wasm_abi {
        let handle_error = expand_wasm_error_handler(ident);

        impls.extend(quote! {
            #[automatically_derived]
            impl #impl_generics TryFromJsValue for #ident #type_generics #where_clause {
                type Error = JsValue;

                #[inline]
                fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
                    Ok(#convert_try_from(#proxy_ident::from_js(value)?).unwrap_throw())
                }
            }

            #[automatically_derived]
            impl #impl_generics FromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as FromWasmAbi>::Abi;

                #[inline]
                unsafe fn from_abi(js: Self::Abi) -> Self {
                    let value = #proxy_ident::from_abi(js);
                    match #convert_try_from(value) {
                        Ok(value) => value,
                        Err(err) => #handle_error,
                    }
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
            impl #impl_generics VectorFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as VectorFromWasmAbi>::Abi;

                #[inline]
                unsafe fn vector_from_abi(js: Self::Abi) -> _wasm_bindgen::__rt::std::boxed::Box<[#ident #type_generics]> {
                    let values = <Vec<#proxy_ident> as FromWasmAbi>::from_abi(js);
                    let mut vector = Vec::<#ident #type_generics>::with_capacity(values.len());
                    for value in values.into_iter() {
                        match #convert_try_from(value) {
                            Ok(value) => vector.push(value),
                            Err(err) => #handle_error,
                        }
                    }
                    vector.into_boxed_slice()
                }
            }

            #[automatically_derived]
            impl #impl_generics RefFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as RefFromWasmAbi>::Abi;
                type Anchor = core::mem::ManuallyDrop<#ident #type_generics>;

                #[inline]
                unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                    let value = <#proxy_ident as RefFromWasmAbi>::ref_from_abi(js);
                    match #convert_try_from(
                        core::mem::ManuallyDrop::into_inner(value)
                    ) {
                        Ok(value) => core::mem::ManuallyDrop::new(value),
                        Err(err) => #handle_error,
                    }
                }
            }

            impl #impl_generics LongRefFromWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as LongRefFromWasmAbi>::Abi;
                type Anchor = #ident #type_generics;

                #[inline]
                unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
                    let value = <#proxy_ident as LongRefFromWasmAbi>::long_ref_from_abi(js);
                    match #convert_try_from(value) {
                        Ok(value) => value,
                        Err(err) => #handle_error,
                    }
                }
            }
        });
    }

    if options.into_wasm_abi {
        impls.extend(quote! {
            #[automatically_derived]
            impl #impl_generics From<#ident #type_generics> for JsValue #where_clause {
                #[inline]
                fn from(value: #ident #type_generics) -> Self {
                    let proxy: #proxy_ident = #convert_into(value);
                    proxy.to_js().unwrap_throw().into()
                }
            }

            #[automatically_derived]
            impl #impl_generics IntoWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as IntoWasmAbi>::Abi;

                fn into_abi(self) -> Self::Abi {
                    let proxy: #proxy_ident = #convert_into(self);
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
            impl #impl_generics VectorIntoWasmAbi for #ident #type_generics #where_clause {
                type Abi = <#proxy_ident as VectorIntoWasmAbi>::Abi;

                #[inline]
                fn vector_into_abi(vector: _wasm_bindgen::__rt::std::boxed::Box<[#ident #type_generics]>) -> Self::Abi {
                    let values: Box<[#proxy_ident]> = vector.into_vec()
                        .into_iter()
                        .map(|value| #convert_into(value))
                        .collect();
                    values.into_abi()
                }
            }
        });
    }

    let usage = expand_usage(options);
    quote! {
        #[automatically_derived]
        const _: () = {
            #usage
            #impls
        };
    }
}

fn expand_usage(options: &WasmOptions) -> TokenStream {
    let mut result = quote!();

    result.extend(if let Some(path) = options.wasm_bindgen_crate.as_ref() {
        quote! {
            use #path as _wasm_bindgen;
        }
    } else {
        quote! {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate wasm_bindgen as _wasm_bindgen;
        }
    });

    result.extend(if let Some(path) = options.js_sys_crate.as_ref() {
        quote! {
            use #path as _js_sys;
        }
    } else {
        quote! {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate js_sys as _js_sys;
        }
    });

    result.extend(
        if let Some(path) = options.serde_attrs().custom_serde_path() {
            quote! {
                use #path as _serde;
            }
        } else {
            quote! {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
            }
        },
    );

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

    let wasm_bindgen_mod = options
        .wasm_bindgen_crate
        .as_ref()
        .map_or_else(|| quote!(wasm_bindgen), ToTokens::to_token_stream);
    let js_sys_mod = options
        .js_sys_crate
        .as_ref()
        .map_or_else(|| quote!(js_sys), ToTokens::to_token_stream);

    quote! {
        #result
        use #wasm_bindgen_mod::{
            prelude::*,
            convert::{
                IntoWasmAbi, FromWasmAbi, OptionIntoWasmAbi, OptionFromWasmAbi, RefFromWasmAbi, LongRefFromWasmAbi,
                TryFromJsValue, VectorFromWasmAbi, VectorIntoWasmAbi,
            },
            describe::{WasmDescribe, WasmDescribeVector},
            JsObject, JsValue,
        };
        use #js_sys_mod::JsString;
        use _bomboni::wasm::Wasm;
    }
}

fn expand_wasm_error_handler(ident: &syn::Ident) -> TokenStream {
    quote! {
        {
            let err: JsValue = err.into();
            if let Some(err_str) = err.as_string() {
                _wasm_bindgen::throw_str(
                    &format!("error converting from WASM `{}`: {}",
                        stringify!(#ident),
                        err_str,
                    )
                )
            } else {
                _wasm_bindgen::throw_val(err)
            }
            // Err(err) => {
            //     _wasm_bindgen::throw_str(&err.to_string())
            //     // #[allow(unreachable_code)]
            //     // core::hint::unreachable_unchecked()
            // }
        }
    }
}
