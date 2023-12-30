use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod wasm;

#[proc_macro_derive(Wasm, attributes(wasm))]
pub fn derive_wasm(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    wasm::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
