#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod wasm;

/// Derive macro for generating WASM bindings.
#[proc_macro_derive(Wasm, attributes(wasm))]
pub fn derive_wasm(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    wasm::derive(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
