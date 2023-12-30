use darling::util;
use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{self, DeriveInput, Generics, Path, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(wasm))]
pub struct WasmNewtype {
    ident: Ident,
    generics: Generics,
    data: ast::Data<util::Ignored, Field>,
    serde: Option<bool>,
    as_string: Option<bool>,
    with: Option<Path>,
    wasm_bindgen_mod: Option<Path>,
}

#[derive(Debug, FromField)]
struct Field {
    pub ty: Type,
}

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let options = match WasmNewtype::from_derive_input(&input) {
        Ok(v) => v,
        Err(err) => {
            return Err(err.into());
        }
    };
    if options.as_string.is_some() && options.with.is_some() {
        return Err(syn::Error::new_spanned(
            &options.with,
            "cannot specify both `with` and `as_string`",
        ));
    }

    let field = match options.data {
        ast::Data::Struct(mut fields) => {
            if fields.len() != 1 {
                return Err(syn::Error::new_spanned(&input, "expected a newtype struct"));
            }
            fields.fields.remove(0)
        }
        ast::Data::Enum(_) => {
            return Err(syn::Error::new_spanned(&input, "expected a newtype struct"));
        }
    };

    let field_type = &field.ty;
    let ident = &options.ident;

    let serde = if options.serde.unwrap_or_default() {
        quote! {
            impl ::serde::Serialize for #ident {
                fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    self.0.serialize(serializer)
                }
            }

            impl<'de> ::serde::Deserialize<'de> for #ident {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    Ok(Self(#field_type::deserialize(deserializer)?))
                }
            }
        }
    } else {
        quote! {}
    };

    let mut proxy = field_type.to_token_stream();
    let mut into_abi = quote! {
        self.0.into_abi()
    };
    let mut from_abi = quote! {
        Self(#field_type::from_abi(js))
    };
    if options.as_string.unwrap_or_default() {
        proxy = quote!(String);
        into_abi = quote! {
            self.0.to_string().into_abi()
        };
        from_abi = quote! {
            Self(#proxy::from_abi(js).parse().unwrap())
        };
    } else if let Some(with) = options.with.as_ref() {
        proxy = quote!(#with);
        into_abi = quote! {
            #with::from(self.0).into_abi()
        };
        from_abi = quote! {
            Self(#with::from_abi(js).into())
        };
    }

    let type_params = {
        let type_params = options.generics.type_params().map(|param| &param.ident);
        quote! {
            <#(#type_params),*>
        }
    };
    let where_clause = if let Some(where_clause) = &options.generics.where_clause {
        quote!(#where_clause)
    } else {
        quote!()
    };
    let wasm_mod = if let Some(wasm_bindgen_mod) = &options.wasm_bindgen_mod {
        quote!(#wasm_bindgen_mod)
    } else {
        quote!(::wasm_bindgen)
    };

    Ok(quote! {
        impl #type_params #wasm_mod::describe::WasmDescribe for #ident #type_params #where_clause {
            fn describe() {
                <#proxy as #wasm_mod::describe::WasmDescribe>::describe();
            }
        }

        impl #type_params #wasm_mod::convert::IntoWasmAbi for #ident #type_params #where_clause {
            type Abi = <#proxy as #wasm_mod::convert::IntoWasmAbi>::Abi;

            fn into_abi(self) -> Self::Abi {
                #into_abi
            }
        }

        impl #type_params #wasm_mod::convert::FromWasmAbi for #ident #type_params #where_clause {
            type Abi = <#proxy as #wasm_mod::convert::FromWasmAbi>::Abi;

            unsafe fn from_abi(js: Self::Abi) -> Self {
                #from_abi
            }
        }

        impl #type_params #wasm_mod::convert::OptionIntoWasmAbi for #ident #type_params #where_clause {
            #[inline]
            fn none() -> Self::Abi {
                <#proxy as #wasm_mod::convert::OptionIntoWasmAbi>::none()
            }
        }

        impl #type_params #wasm_mod::convert::OptionFromWasmAbi for #ident #type_params #where_clause {
            #[inline]
            fn is_none(abi: &Self::Abi) -> bool {
                <#proxy as #wasm_mod::convert::OptionFromWasmAbi>::is_none(abi)
            }
        }

        #serde
    })
}
