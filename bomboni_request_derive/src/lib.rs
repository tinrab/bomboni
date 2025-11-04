//! # A procedural macro crate for the Bomboni library.

use parse::{
    derived_map::{self, DerivedMap},
    parse_resource_name::{self, ParseResourceName},
};
use proc_macro::TokenStream;

mod parse;

use syn::{DeriveInput, parse_macro_input};

/// A procedural macro that generates a function that parses a resource name into a tuple of typed segments.
/// Resource name format is documented in Google's AIP [1].
/// Ending segments can also be optional [2].
///
/// [1]: https://google.aip.dev/122
/// [2]: https://google.aip.dev/162
///
/// # Examples
///
/// ```
/// # use bomboni_request_derive::parse_resource_name;
/// let name = "users/42/projects/1337";
/// let parsed = parse_resource_name!({
///    "users": u32,
///    "projects": u64,
/// })(name);
/// assert_eq!(parsed, Some((42, 1337)));
/// ```
#[proc_macro]
pub fn parse_resource_name(input: TokenStream) -> TokenStream {
    let options = parse_macro_input!(input as ParseResourceName);
    parse_resource_name::expand(options)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn derived_map(input: TokenStream) -> TokenStream {
    let options = parse_macro_input!(input as DerivedMap);
    derived_map::expand(options)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Parse, attributes(parse))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    parse::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
