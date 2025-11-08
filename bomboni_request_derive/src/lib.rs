#![doc = include_str!("../README.md")]

use parse::{
    derived_map::{self, DerivedMap},
    parse_resource_name::{self, ParseResourceName},
};
use proc_macro::TokenStream;

mod parse;

use syn::{DeriveInput, parse_macro_input};

/// A procedural macro that generates a function that parses a resource name into a tuple of typed segments.
///
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

/// Derive macro for creating derived map types.
#[proc_macro]
pub fn derived_map(input: TokenStream) -> TokenStream {
    let options = parse_macro_input!(input as DerivedMap);
    derived_map::expand(options)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive macro for parsing request types.
///
/// This macro generates code for converting between different data representations.
/// It's commonly used for converting protobuf messages to domain models, but can
/// handle any type conversion scenario.
///
/// # Attributes
///
/// ## Struct-level attributes
///
/// - `source = "Type"` - Source type to parse from (required)
/// - `write = bool` - Generate write/conversion code back to source type
/// - `serialize_as = bool` - Generate Serialize for source type
/// - `deserialize_as = bool` - Generate Deserialize for source type
/// - `serde_as = bool` - Generate both Serialize and Deserialize for source type
/// - `request = {...}` - Mark as request message for error handling
/// - `tagged_union = {...}` - Create tagged union from oneof field
/// - `bomboni_crate = "path"` - Custom bomboni crate path
/// - `bomboni_proto_crate = "path"` - Custom `bomboni_proto` crate path
/// - `bomboni_request_crate = "path"` - Custom `bomboni_request` crate path
/// - `serde_crate = "path"` - Custom serde crate path
///
/// ## Field-level attributes
///
/// - `source = "field"` - Source field name to parse from
/// - `source_field = bool` - Source field name is same as target
/// - `skip = bool` - Skip parsing this field
/// - `keep = bool` - Keep field unchanged
/// - `keep_primitive = bool` - Keep primitive types unchanged
/// - `unspecified = bool` - Allow unspecified enum values
/// - `extract = {...}` - Custom extraction plan
/// - `wrapper = bool` - Parse protobuf wrapper types
/// - `oneof = bool` - Parse from oneof field
/// - `enumeration = bool` - Parse enum from i32
/// - `regex = "pattern"` - Validate with regex
/// - `timestamp = bool` - Parse protobuf timestamps
/// - `try_from = "path"` - Custom `TryFrom` conversion
/// - `convert = {...}` - Custom conversion functions
/// - `derive = {...}` - Use derived parsing
/// - `resource = {...}` - Parse resource fields
/// - `list_query = {...}` - Parse list query
/// - `search_query = {...}` - Parse search query
/// - `field_mask = {...}` - Parse field only if field mask allows it
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,ignore
/// #[derive(Parse)]
/// #[parse(source = "proto::User")]
/// struct User {
///     #[parse(source = "user_name")]
///     name: String,
/// }
/// ```
///
/// With bidirectional conversion:
///
/// ```rust,ignore
/// #[derive(Parse)]
/// #[parse(source = "proto::User", write = true)]
/// struct User {
///     name: String,
///     email: Option<String>,
/// }
/// ```
///
/// Complex example with multiple features:
///
/// ```rust,ignore
/// #[derive(Parse)]
/// #[parse(source = "proto::UserMessage", write = true, serde_as = true)]
/// struct User {
///     #[parse(source = "user_id", try_from = "UserId::from_str")]
///     id: UserId,
///
///     #[parse(source = "user_name")]
///     name: String,
///
///     #[parse(source = "email_address", regex = r"[^@]+@[^@]+\.[^@]+")]
///     email: String,
///
///     #[parse(source = "created_timestamp", timestamp)]
///     created_at: OffsetDateTime,
///
///     #[parse(source = "status_code", enumeration)]
///     status: UserStatus,
///
///     #[parse(source = "profile_data", keep)]
///     metadata: HashMap<String, String>,
/// }
/// ```
///
/// Field mask example for update operations:
///
/// ```rust,ignore
/// #[derive(Parse)]
/// #[parse(source = "UpdateBookRequest", write = true)]
/// struct ParsedUpdateBookRequest {
///     #[parse(source = "book?.name", convert = book_id_convert)]
///     id: BookId,
///     #[parse(source = "book?.display_name", field_mask { field = book, mask = update_mask })]
///     display_name: Option<String>,
/// }
/// ```
#[proc_macro_derive(Parse, attributes(parse))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    parse::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
