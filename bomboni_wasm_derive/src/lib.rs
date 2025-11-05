#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod wasm;

/// Derive macro for generating WASM bindings.
///
/// This macro generates TypeScript WASM bindings for Rust types, enabling them
/// to be used in JavaScript/TypeScript environments through wasm-bindgen.
///
/// # Attributes
///
/// ## Struct-level attributes
///
/// - `rename = "Name"` - Custom name in TypeScript
/// - `into_wasm_abi = bool` - Generate `IntoWasmAbi` implementation
/// - `from_wasm_abi = bool` - Generate `FromWasmAbi` implementation
/// - `wasm_abi = bool` - Generate both `IntoWasmAbi` and `FromWasmAbi`
/// - `enum_value = bool` - Generate enum value object (for enums)
/// - `js_value(...)` - Custom `JsValue` conversion configuration
/// - `proxy = Type` - Use proxy type for conversions
/// - `rename_all = "rule"` - Rename rule for all fields/variants
/// - `rename_boundary = "boundary"` - Word boundaries for renaming
/// - `override_type = "string"` - Override generated TypeScript type
/// - `change_ref = "mapping"` - Reference type mapping
/// - `rename_wrapper = bool` - Control wrapper type renaming
/// - `wasm_bindgen_crate = "path"` - Custom wasm-bindgen crate path
/// - `js_sys_crate = "path"` - Custom js-sys crate path
/// - `bomboni_crate = "path"` - Custom bomboni crate path
/// - `bomboni_wasm_crate = "path"` - Custom `bomboni_wasm` crate path
///
/// ## Field-level attributes
///
/// - `rename = "name"` - Custom field name in TypeScript
/// - `override_type = "type"` - Override field TypeScript type
/// - `rename_wrapper = bool` - Control wrapper type renaming for field
/// - `always_some = bool` - Force field to be required in TypeScript
/// - `change_ref = "mapping"` - Reference type mapping for field
///
/// ## Variant-level attributes (for enums)
///
/// - `rename = "name"` - Custom variant name in TypeScript
/// - `override_type = "type"` - Override variant TypeScript type
/// - `rename_wrapper = bool` - Control wrapper type renaming for variant
/// - `change_ref = "mapping"` - Reference type mapping for variant
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,ignore
/// #[derive(Wasm)]
/// struct User {
///     name: String,
///     age: u32,
/// }
/// ```
///
/// With custom naming:
///
/// ```rust,ignore
/// #[derive(Wasm)]
/// #[wasm(rename = "UserProfile", rename_all = "camelCase")]
/// struct User {
///     user_name: String,  // Becomes "userName"
///     created_at: /* DateTime */,  // Becomes "createdAt"
/// }
/// ```
///
/// With custom conversions:
///
/// ```rust,ignore
/// #[derive(Wasm)]
/// #[wasm(js_value(convert_string))]
/// struct UserId(String);
///
/// #[derive(Wasm)]
/// #[wasm(proxy = ComplexTypeProxy)]
/// struct ComplexType {
///     data: Vec<u8>,
/// }
/// ```
#[proc_macro_derive(Wasm, attributes(wasm))]
pub fn derive_wasm(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    wasm::derive(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
