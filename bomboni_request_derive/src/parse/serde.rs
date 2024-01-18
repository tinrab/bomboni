use crate::parse::ParseOptions;
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand(options: &ParseOptions) -> syn::Result<TokenStream> {
    if !options.serde_as && !options.serialize_as && !options.deserialize_as {
        return Ok(quote!());
    }

    let mut result = if let Some(path) = options.serde_crate.as_ref() {
        quote! {
            use #path as _serde;
        }
    } else {
        quote! {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
        }
    };

    if options.serde_as || options.serialize_as {
        if !options.write {
            return Err(syn::Error::new_spanned(
                &options.ident,
                "Cannot use `serde_as` or `serialize_as` without `write`",
            ));
        }

        let source = &options.source;
        let ident = &options.ident;
        let (impl_generics, type_generics, where_clause) = options.generics.split_for_impl();

        result.extend(quote! {
            #[automatically_derived]
            impl #impl_generics _serde::Serialize for #ident #type_generics #where_clause {
                fn serialize<__S>(&self, serializer: __S) -> _serde::__private::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    #source::serialize(&self.clone().into(), serializer)
                }
            }
        });
    }

    if options.serde_as || options.deserialize_as {
        let source = &options.source;
        let ident = &options.ident;
        let (impl_generics, type_generics, where_clause) = options.generics.split_for_impl();

        result.extend(quote! {
            #[automatically_derived]
            impl<'de> #impl_generics _serde::Deserialize<'de> for #ident #type_generics #where_clause {
                fn deserialize<__D>(deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #source::deserialize(deserializer)?
                       .parse_into()
                        .map_err(_serde::de::Error::custom)
                }
            }
        });
    }

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _ : () = {
            #result
        };
    })
}
