#![allow(clippy::option_if_let_else)]

use bomboni_core::syn::type_is_phantom;
use darling::{
    FromDeriveInput, FromField, FromMeta, FromVariant,
    ast::{Data, Fields, NestedMeta, Style},
    util::parse_expr,
};
use proc_macro2::Ident;
use quote::format_ident;
use syn::{
    self, DeriveInput, Expr, ExprArray, ExprCall, ExprPath, Generics, LitBool, LitStr, Meta,
    MetaList, MetaNameValue, Path, Type, TypePath, parse_quote,
};

use super::field_type_info::{FieldTypeInfo, get_field_type_info};

/// Main options for the Parse derive macro.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(parse), supports(struct_any, enum_any))]
pub struct ParseOptions {
    /// The identifier of the struct or enum being derived.
    pub ident: Ident,

    /// Generic parameters for the type.
    pub generics: Generics,

    /// The data (fields or variants) of the struct or enum.
    pub data: Data<ParseVariant, ParseField>,

    /// Source type to parse from.
    pub source: Path,

    /// Generate `From` trait implementation for converting back to source type.
    ///
    /// When set to `true`, generates code to convert the parsed type back into
    /// the source type. This enables bidirectional conversion between the types.
    #[darling(default)]
    pub write: bool,

    /// Implement `serde::Serialize` for the source type.
    ///
    /// When set to `true`, generates a `Serialize` implementation that serializes
    /// the source type instead of the parsed type. This is useful when you want
    /// to serialize data in the original format.
    #[darling(default)]
    pub serialize_as: bool,

    /// Implement `serde::Deserialize` for the source type.
    ///
    /// When set to `true`, generates a `Deserialize` implementation that deserializes
    /// directly into the source type. This is useful when you want to deserialize
    /// data into the original format.
    #[darling(default)]
    pub deserialize_as: bool,

    /// Implement both `serde::Serialize` and `serde::Deserialize` for the source type.
    ///
    /// When set to `true`, this is a shorthand for setting both `serialize_as` and
    /// `deserialize_as` to `true`. This generates complete serde support for the source type.
    ///
    /// This is commonly used when you want full serde compatibility with the original format.
    #[darling(default)]
    pub serde_as: bool,

    /// Create tagged union from a oneof field.
    ///
    /// Specifies that this struct should be treated as a tagged union, where the
    /// specific variant is determined by a oneof field. This is commonly used for
    /// protobuf messages that contain oneof fields representing different message types.
    #[darling(default)]
    pub tagged_union: Option<ParseTaggedUnion>,

    /// Mark this message as a request message for enhanced error handling.
    /// Errors will be wrapped within `BadRequest` with [`RequestError::bad_request`].
    #[darling(default)]
    pub request: Option<ParseRequest>,

    /// Custom `prost` crate path.
    #[darling(default)]
    pub prost_crate: Option<Path>,

    /// Custom `bomboni` crate path.
    #[darling(default)]
    pub bomboni_crate: Option<Path>,

    /// Custom `bomboni_proto` crate path.
    #[darling(default)]
    pub bomboni_proto_crate: Option<Path>,

    /// Custom `bomboni_request` crate path.
    #[darling(default)]
    pub bomboni_request_crate: Option<Path>,

    /// Custom `serde` crate path.
    #[darling(default)]
    pub serde_crate: Option<Path>,
}

/// Configuration for creating tagged unions from oneof fields.
#[derive(Debug, FromMeta)]
pub struct ParseTaggedUnion {
    /// The oneof field that contains the variant data.
    pub oneof: Path,

    /// The field that contains the tag/variant identifier.
    pub field: Ident,
}

/// Configuration for request message error handling.
///
/// When a struct is marked as a request message, parsing errors will be
/// wrapped with additional context to make debugging and error reporting easier.
#[derive(Debug)]
pub struct ParseRequest {
    /// Optional custom name for the request.
    pub name: Option<Expr>,
}

/// Represents a field in a struct that can be parsed.
#[derive(Debug, Clone, FromField)]
#[darling(attributes(parse))]
pub struct ParseField {
    /// The identifier of the field.
    pub ident: Option<Ident>,

    /// The type of the field.
    pub ty: Type,

    /// Parsing options for this field.
    #[darling(flatten)]
    pub options: ParseFieldOptions,

    /// Parse resource fields into this field.
    pub resource: Option<ParseResource>,

    /// Parse list query fields.
    #[darling(default)]
    pub list_query: Option<ParseQuery>,

    /// Parse search query fields.
    #[darling(default)]
    pub search_query: Option<ParseQuery>,

    /// Type information for the field (internal use).
    #[darling(skip)]
    pub type_info: Option<FieldTypeInfo>,
}

/// Represents a variant in an enum that can be parsed.
#[derive(Debug, Clone, FromVariant)]
#[darling(attributes(parse))]
pub struct ParseVariant {
    /// The identifier of the variant.
    pub ident: Ident,

    /// The fields of the variant.
    pub fields: Fields<Type>,

    /// Parsing options for this variant.
    #[darling(flatten)]
    pub options: ParseFieldOptions,

    /// True if the source is an empty unit variant.
    #[darling(default)]
    pub source_unit: bool,

    /// Type information for the variant (internal use).
    #[darling(skip)]
    pub type_info: Option<FieldTypeInfo>,
}

/// Options for controlling how individual fields are parsed.
#[derive(Debug, Clone, FromMeta, Default)]
pub struct ParseFieldOptions {
    /// Source field name to parse from.
    ///
    /// Specifies the name of the field in the input data to parse from.
    /// Can be a path to a nested field with conditional `?.` extraction.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[parse(source = "bio")]
    /// biography: String,
    ///
    /// #[parse(source = "address?.city")]
    /// city: Option<String>,
    /// ```
    #[darling(default)]
    pub source: Option<String>,

    /// Indicates that the source field name is the same as the target field name.
    ///
    /// When set to `true`, this is a shorthand for `source = "<field_name>"`.
    /// This is useful when you want to explicitly indicate that a field should
    /// be parsed from a source field with the same name. This is commonly used
    /// with `derive` attribute.
    #[darling(default)]
    pub source_field: bool,

    /// Skip parsing this field entirely.
    ///
    /// When set to `true`, this field will be completely ignored during parsing.
    /// The field will not be read from the input and will not be included in the output.
    #[darling(default)]
    pub skip: bool,

    /// Keep the source and target fields the same without any parsing.
    #[darling(default)]
    pub keep: bool,

    /// Keep source and target primitive message types the same.
    ///
    /// When set to `true`, only the surrounding container will be extracted and parsed,
    /// while the primitive message types inside are kept the same.
    ///
    /// This is useful for complex nested structures where you want to parse the outer
    /// container but preserve the inner primitive types unchanged.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[parse(keep_primitive)]
    /// nested_message: Vec<InnerMessage>,  // Vec is parsed, InnerMessage is kept
    /// ```
    #[darling(default)]
    pub keep_primitive: bool,

    /// Allow unspecified enum values and empty strings without treating them as required.
    ///
    /// When set to `true`, the field will not be treated as required and will accept
    /// unspecified enum values (typically 0) and empty strings without generating errors.
    #[darling(default)]
    pub unspecified: bool,

    /// Custom extraction plan for the field.
    ///
    /// Specifies a series of extraction steps to transform the field value.
    /// This provides fine-grained control over how values are extracted and processed.
    ///
    /// The extraction plan consists of multiple steps that are applied in sequence.
    #[darling(default)]
    pub extract: Option<FieldExtract>,

    /// Parse Protobuf's well-known wrapper types.
    ///
    /// When set to `true`, automatically handles Protobuf wrapper types by extracting
    /// the inner value. This is commonly used for optional primitive fields in protobuf.
    ///
    /// Types are mapped as follows:
    ///
    /// - `String` → `StringValue`
    /// - `bool` → `BoolValue`
    /// - `f32` → `FloatValue`
    /// - `f64` → `DoubleValue`
    /// - `i8`, `i16`, `i32` → `Int32Value`
    /// - `u8`, `u16`, `u32` → `UInt32Value`
    /// - `i64`, `isize` → `Int64Value`
    /// - `u64`, `usize` → `UInt64Value`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[parse(wrapper)]
    /// optional_name: Option<String>,  // From StringValue
    ///
    /// #[parse(wrapper)]
    /// optional_count: Option<i32>,     // From Int32Value
    /// ```
    #[darling(default)]
    pub wrapper: bool,

    /// Parse oneof value from a Protobuf oneof field.
    ///
    /// When set to `true`, indicates that this field should be parsed from a
    /// Protobuf oneof field. This is a special-purpose parse option for handling
    /// oneof fields in protobuf messages.
    ///
    /// The field will be extracted from the oneof and converted to the appropriate type.
    #[darling(default)]
    pub oneof: bool,

    /// Parse enum value from `i32`.
    ///
    /// When set to `true`, indicates that this field should be parsed as an enum
    /// from an `i32` value. This is a special-purpose parse option for enum fields
    /// that are represented as integers in the source data.
    #[darling(default)]
    pub enumeration: bool,

    /// Check string against a regular expression pattern.
    ///
    /// Specifies a regular expression that the field value must match.
    /// The field will be parsed only if the string matches the regex pattern.
    #[darling(with = parse_expr::preserve_str_literal, map = Some)]
    pub regex: Option<Expr>,

    /// Parse `google.protobuf.Timestamp` into a `OffsetDateTime`.
    ///
    /// When set to `true`, automatically converts protobuf timestamp fields
    /// into `OffsetDateTime` instances. This handles the conversion from the
    /// protobuf timestamp format to Rust's date/time representation.
    #[darling(default)]
    pub timestamp: bool,

    /// Convert field to a custom type using `try_from` or `try_into`.
    ///
    /// Specifies a custom type path that implements `TryFrom` or `TryInto`
    /// for converting the field value. The conversion can fail and returns a Result.
    ///
    /// This is useful for custom conversion logic that doesn't fit into other
    /// categories, such as domain-specific types, validation, or complex transformations.
    #[darling(with = parse_type_path, map = Some)]
    pub try_from: Option<TypePath>,

    /// Use custom conversion and writing functions.
    ///
    /// Specifies custom conversion functions for both parsing (reading) and writing.
    /// This provides maximum flexibility for complex field transformations.
    #[darling(default)]
    pub convert: Option<ParseConvert>,

    /// Make this field use derived parsing implementation.
    ///
    /// When set, indicates that this field should use a custom derived parsing
    /// implementation. This is useful for custom, non-opinionated parsing where
    /// you have full control over the parsing logic.
    #[darling(default)]
    pub derive: Option<ParseDerive>,

    /// Parse field only if field mask allows it.
    ///
    /// When set, indicates that this field should only be parsed if the specified
    /// field mask contains the field path. This is commonly used for update
    /// operations where only certain fields should be modified.
    #[darling(default)]
    pub field_mask: Option<ParseFieldMask>,
}

#[derive(Debug, Clone)]
pub struct FieldExtract {
    pub steps: Vec<FieldExtractStep>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FieldExtractStep {
    Field(String),
    Unwrap,
    UnwrapOr(Expr),
    UnwrapOrDefault,
    Unbox,
    StringFilterEmpty,
    EnumerationFilterUnspecified,
}

#[derive(Debug, Clone)]
pub struct ParseDerive {
    pub parse: Option<ExprPath>,
    pub write: Option<ExprPath>,
    pub module: Option<ExprPath>,
    pub source_borrow: bool,
    pub target_borrow: bool,
}

#[derive(Debug, Clone)]
pub struct ParseConvert {
    pub parse: Option<ExprPath>,
    pub write: Option<ExprPath>,
    pub module: Option<ExprPath>,
}

#[derive(Debug, Clone)]
pub struct ParseResource {
    pub name: ParseResourceField,
    pub create_time: ParseResourceField,
    pub update_time: ParseResourceField,
    pub delete_time: ParseResourceField,
    pub deleted: ParseResourceField,
    pub etag: ParseResourceField,
}

#[derive(Debug, Clone)]
pub struct ParseResourceField {
    pub parse: bool,
    pub write: bool,
    pub source: Ident,
}

#[derive(Debug, Clone)]
pub struct ParseQuery {
    pub query: ParseQueryField,
    pub page_size: ParseQueryField,
    pub page_token: ParseQueryField,
    pub filter: ParseQueryField,
    pub order_by: ParseQueryField,
}

#[derive(Debug, Clone)]
pub struct ParseQueryField {
    pub parse: bool,
    pub write: bool,
    pub source: Ident,
}

/// Configuration for parsing fields with field mask validation.
#[derive(Debug, Clone)]
pub struct ParseFieldMask {
    /// The field containing the field mask.
    pub mask: Ident,

    /// The field containing the data to parse (optional, inferred from source if not provided).
    pub field: Option<Ident>,
}

impl ParseOptions {
    pub fn parse(input: &DeriveInput) -> syn::Result<Self> {
        let mut options = Self::from_derive_input(input)?;

        options.data = match &options.data {
            Data::Struct(data) => {
                let mut fields = Vec::new();
                let mut contains_query = false;

                for mut field in data.fields.iter().cloned() {
                    if field.list_query.is_some() || field.search_query.is_some() {
                        if contains_query {
                            return Err(syn::Error::new_spanned(
                                &field.ident,
                                "can only have one list or search query field",
                            ));
                        }
                        contains_query = true;
                    }
                    if field.list_query.is_some() && field.search_query.is_some() {
                        return Err(syn::Error::new_spanned(
                            &field.ident,
                            "list and search query cannot be used together",
                        ));
                    }
                    if (field.list_query.is_some() || field.search_query.is_some())
                        && (field.options.keep
                            || field.options.keep_primitive
                            || field.options.derive.is_some()
                            || field.options.oneof
                            || field.options.enumeration
                            || field.resource.is_some())
                    {
                        return Err(syn::Error::new_spanned(
                            &field.ident,
                            "query fields cannot be used with these options",
                        ));
                    }

                    if field.options.extract.is_some() && field.options.source.is_some() {
                        return Err(syn::Error::new_spanned(
                            &field.ident,
                            "`extract` and `source` cannot be used together",
                        ));
                    }

                    if (field.options.oneof || field.options.enumeration)
                        && (field.options.keep
                            || field.options.keep_primitive
                            || field.options.try_from.is_some()
                            || field.options.derive.is_some()
                            || field.options.convert.is_some()
                            || field.resource.is_some()
                            || field.list_query.is_some()
                            || field.search_query.is_some())
                    {
                        return Err(syn::Error::new_spanned(
                            &field.ident,
                            "`oneof` and `enumeration` cannot be used with these options",
                        ));
                    }

                    if field.options.wrapper
                        && (field.options.keep
                            || field.options.keep_primitive
                            || field.options.derive.is_some()
                            || field.options.oneof
                            || field.options.enumeration
                            || field.options.try_from.is_some()
                            || field.resource.is_some()
                            || field.list_query.is_some()
                            || field.search_query.is_some())
                    {
                        return Err(syn::Error::new_spanned(
                            &field.ident,
                            "`wrapper` cannot be used with these options`",
                        ));
                    }

                    if field.options.try_from.is_some()
                        && (field.options.keep
                            || field.options.keep_primitive
                            || field.options.derive.is_some()
                            || field.options.oneof
                            || field.options.enumeration
                            || field.options.convert.is_some()
                            || field.resource.is_some()
                            || field.list_query.is_some()
                            || field.search_query.is_some()
                            || field.options.field_mask.is_some())
                    {
                        return Err(syn::Error::new_spanned(
                            &field.ident,
                            "`try_from` cannot be used with these options",
                        ));
                    }

                    if field.options.source_field {
                        field.options.source = Some(
                            field
                                .ident
                                .as_ref()
                                .ok_or_else(|| {
                                    syn::Error::new(
                                        proc_macro2::Span::call_site(),
                                        "field missing ident",
                                    )
                                })?
                                .to_string(),
                        );
                    }

                    if field.options.skip
                        || field.options.derive.is_some()
                        || field.list_query.is_some()
                        || field.search_query.is_some()
                        || field.resource.is_some()
                        || type_is_phantom(&field.ty)
                    {
                        fields.push(field);
                        continue;
                    }

                    let field_type_info = get_field_type_info(&options, &field.options, &field.ty)?;
                    fields.push(ParseField {
                        type_info: Some(field_type_info),
                        ..field
                    });
                }

                Data::Struct(Fields::new(Style::Struct, fields))
            }
            Data::Enum(data) => {
                let mut variants = Vec::new();
                for mut variant in data.iter().cloned() {
                    if variant.options.extract.is_some() && variant.options.source.is_some() {
                        return Err(syn::Error::new_spanned(
                            &variant.ident,
                            "`extract` and `source` cannot be used together",
                        ));
                    }

                    if variant.options.wrapper
                        && (variant.options.keep
                            || variant.options.keep_primitive
                            || variant.options.try_from.is_some()
                            || variant.options.derive.is_some()
                            || variant.options.enumeration)
                    {
                        return Err(syn::Error::new_spanned(
                            &variant.ident,
                            "`wrapper` cannot be used with these options`",
                        ));
                    }

                    if variant.options.try_from.is_some()
                        && (variant.options.keep
                            || variant.options.keep_primitive
                            || variant.options.derive.is_some()
                            || variant.options.enumeration
                            || variant.options.convert.is_some())
                    {
                        return Err(syn::Error::new_spanned(
                            &variant.ident,
                            "`try_from` cannot be used with these options",
                        ));
                    }

                    if variant.options.source_field {
                        variant.options.source = Some(variant.ident.to_string());
                    }

                    if variant.options.skip
                        || variant.options.derive.is_some()
                        || variant.source_unit
                    {
                        variants.push(variant);
                        continue;
                    }

                    match variant.fields.iter().next() {
                        Some(variant_type) if !type_is_phantom(variant_type) => {
                            let field_type_info =
                                get_field_type_info(&options, &variant.options, variant_type)?;
                            variants.push(ParseVariant {
                                type_info: Some(field_type_info),
                                ..variant
                            });
                        }
                        _ => {
                            variants.push(variant);
                        }
                    }
                }
                Data::Enum(variants)
            }
        };

        Ok(options)
    }
}

impl FromMeta for ParseRequest {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let mut name = None;
        for item in items {
            match item {
                NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, value, .. })) => {
                    if path.is_ident("name") {
                        name = Some(value.clone());
                    } else {
                        return Err(
                            darling::Error::custom("invalid request option").with_span(item)
                        );
                    }
                }
                _ => {
                    return Err(darling::Error::custom("invalid request option").with_span(item));
                }
            }
        }
        Ok(Self { name })
    }

    fn from_word() -> darling::Result<Self> {
        Ok(Self { name: None })
    }
}

impl FromMeta for FieldExtract {
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        let mut steps = Vec::new();
        match item {
            Meta::NameValue(MetaNameValue {
                path,
                value: Expr::Array(ExprArray { elems, .. }),
                ..
            }) if path.is_ident("extract") => {
                for elem in elems {
                    steps.push(FieldExtractStep::from_expr(elem)?);
                }
            }
            _ => {
                return Err(darling::Error::custom("invalid extract").with_span(item));
            }
        }
        Ok(Self { steps })
    }
}

impl FromMeta for FieldExtractStep {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(ExprPath { path, .. }) => {
                if path.is_ident("Unwrap") {
                    Ok(Self::Unwrap)
                } else if path.is_ident("UnwrapOrDefault") {
                    Ok(Self::UnwrapOrDefault)
                } else if path.is_ident("Unbox") {
                    Ok(Self::Unbox)
                } else if path.is_ident("StringFilterEmpty") {
                    Ok(Self::StringFilterEmpty)
                } else if path.is_ident("EnumerationFilterUnspecified") {
                    Ok(Self::EnumerationFilterUnspecified)
                } else {
                    Err(darling::Error::custom("unknown extract step").with_span(path))
                }
            }
            Expr::Call(ExprCall { func, args, .. }) => {
                let func_ident: Ident = parse_quote!(#func);
                if func_ident == "Field" {
                    if args.len() != 1 {
                        return Err(darling::Error::custom("expected one argument").with_span(args));
                    }
                    let value: LitStr = parse_quote!(#args);
                    let value = value.value();
                    if value.contains('.') || value.contains('?') {
                        return Err(darling::Error::custom("invalid field name").with_span(&value));
                    }
                    Ok(Self::Field(value))
                } else if func_ident == "UnwrapOr" {
                    if args.len() != 1 {
                        return Err(darling::Error::custom("expected one argument").with_span(args));
                    }
                    Ok(Self::UnwrapOr(args[0].clone()))
                } else {
                    Err(darling::Error::custom("unknown extract step").with_span(func))
                }
            }
            _ => Err(darling::Error::custom("invalid extract step").with_span(expr)),
        }
    }
}

impl FromMeta for ParseDerive {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        #[derive(FromMeta)]
        struct Options {
            #[darling(default)]
            parse: Option<ExprPath>,
            #[darling(default)]
            write: Option<ExprPath>,
            #[darling(default)]
            module: Option<ExprPath>,
            #[darling(default)]
            source_borrow: bool,
            #[darling(default)]
            target_borrow: bool,
            #[darling(default)]
            borrow: bool,
        }

        let options = Options::from_list(items)?;

        if options.parse.is_none() && options.write.is_none() && options.module.is_none()
            || options.module.is_some() && (options.parse.is_some() || options.write.is_some())
        {
            return Err(darling::Error::custom("invalid options"));
        }

        Ok(Self {
            parse: options.parse,
            write: options.write,
            module: options.module,
            source_borrow: options.source_borrow || options.borrow,
            target_borrow: options.target_borrow || options.borrow,
        })
    }

    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(path) => Ok(Self {
                parse: None,
                write: None,
                module: Some(path.clone()),
                source_borrow: false,
                target_borrow: false,
            }),
            _ => Err(darling::Error::custom("expected path").with_span(expr)),
        }
    }
}

impl FromMeta for ParseConvert {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        #[derive(FromMeta)]
        struct Options {
            #[darling(default)]
            parse: Option<ExprPath>,
            #[darling(default)]
            write: Option<ExprPath>,
            #[darling(default)]
            module: Option<ExprPath>,
        }

        let options = Options::from_list(items)?;

        if options.parse.is_none() && options.write.is_none() && options.module.is_none()
            || options.module.is_some() && (options.parse.is_some() || options.write.is_some())
        {
            return Err(darling::Error::custom("invalid options"));
        }

        Ok(Self {
            parse: options.parse,
            write: options.write,
            module: options.module,
        })
    }

    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(path) => Ok(Self {
                parse: None,
                write: None,
                module: Some(path.clone()),
            }),
            _ => Err(darling::Error::custom("expected path").with_span(expr)),
        }
    }
}

impl FromMeta for ParseResource {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        #[derive(FromMeta)]
        struct MetaOptions {
            fields: Option<FieldsMetaOptions>,
        }

        #[derive(FromMeta)]
        struct FieldsMetaOptions {
            name: Option<ParseResourceField>,
            create_time: Option<ParseResourceField>,
            update_time: Option<ParseResourceField>,
            delete_time: Option<ParseResourceField>,
            deleted: Option<ParseResourceField>,
            etag: Option<ParseResourceField>,
        }

        let options = MetaOptions::from_list(items)?;

        let mut resource = Self::default();
        if let Some(fields) = options.fields {
            if let Some(field) = fields.name {
                resource.name = field;
            }
            if let Some(field) = fields.create_time {
                resource.create_time = field;
            }
            if let Some(field) = fields.update_time {
                resource.update_time = field;
            }
            if let Some(field) = fields.delete_time {
                resource.delete_time = field;
            }
            if let Some(field) = fields.deleted {
                resource.deleted = field;
            }
            if let Some(field) = fields.etag {
                resource.etag = field;
            }
        }

        Ok(resource)
    }

    fn from_word() -> darling::Result<Self> {
        Ok(Self::default())
    }
}

impl Default for ParseResource {
    fn default() -> Self {
        Self {
            name: ParseResourceField {
                parse: true,
                write: true,
                source: format_ident!("name"),
            },
            create_time: ParseResourceField {
                parse: true,
                write: true,
                source: format_ident!("create_time"),
            },
            update_time: ParseResourceField {
                parse: true,
                write: true,
                source: format_ident!("update_time"),
            },
            delete_time: ParseResourceField {
                parse: true,
                write: true,
                source: format_ident!("delete_time"),
            },
            deleted: ParseResourceField {
                parse: true,
                write: true,
                source: format_ident!("deleted"),
            },
            etag: ParseResourceField {
                parse: true,
                write: true,
                source: format_ident!("etag"),
            },
        }
    }
}

impl FromMeta for ParseResourceField {
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        match item {
            Meta::NameValue(MetaNameValue { path, value, .. }) => {
                let include = LitBool::from_expr(value)?.value;
                Ok(Self {
                    source: path.require_ident()?.clone(),
                    write: include,
                    parse: include,
                })
            }
            meta @ Meta::List(MetaList { path, .. }) => {
                #[derive(FromMeta)]
                struct MetaOptions {
                    #[darling(default)]
                    source: Option<Ident>,
                    #[darling(default)]
                    parse: bool,
                    #[darling(default)]
                    write: bool,
                }

                let options = MetaOptions::from_meta(meta)?;
                Ok(Self {
                    source: if let Some(source) = options.source {
                        source
                    } else {
                        path.require_ident()?.clone()
                    },
                    write: options.write,
                    parse: options.parse,
                })
            }
            _ => Err(darling::Error::custom("invalid resource field").with_span(item)),
        }
    }
}

impl FromMeta for ParseQuery {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        #[derive(FromMeta)]
        struct MetaOptions {
            query: Option<ParseQueryField>,
            page_size: Option<ParseQueryField>,
            page_token: Option<ParseQueryField>,
            filter: Option<ParseQueryField>,
            order_by: Option<ParseQueryField>,
        }

        let options = MetaOptions::from_list(items)?;

        let mut query = Self::default();
        if let Some(field) = options.query {
            query.query = field;
        }
        if let Some(field) = options.page_size {
            query.page_size = field;
        }
        if let Some(field) = options.page_token {
            query.page_token = field;
        }
        if let Some(field) = options.filter {
            query.filter = field;
        }
        if let Some(field) = options.order_by {
            query.order_by = field;
        }

        Ok(query)
    }

    fn from_word() -> darling::Result<Self> {
        Ok(Self::default())
    }
}

impl Default for ParseQuery {
    fn default() -> Self {
        Self {
            query: ParseQueryField {
                parse: true,
                write: true,
                source: format_ident!("query"),
            },
            page_size: ParseQueryField {
                parse: true,
                write: true,
                source: format_ident!("page_size"),
            },
            page_token: ParseQueryField {
                parse: true,
                write: true,
                source: format_ident!("page_token"),
            },
            filter: ParseQueryField {
                parse: true,
                write: true,
                source: format_ident!("filter"),
            },
            order_by: ParseQueryField {
                parse: true,
                write: true,
                source: format_ident!("order_by"),
            },
        }
    }
}

impl FromMeta for ParseQueryField {
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        match item {
            Meta::NameValue(MetaNameValue { path, value, .. }) => {
                let include = LitBool::from_expr(value)?.value;
                Ok(Self {
                    source: path.require_ident()?.clone(),
                    write: include,
                    parse: include,
                })
            }
            meta @ Meta::List(MetaList { path, .. }) => {
                #[derive(FromMeta)]
                struct MetaOptions {
                    #[darling(default)]
                    source: Option<Ident>,
                    #[darling(default)]
                    parse: bool,
                    #[darling(default)]
                    write: bool,
                }

                let options = MetaOptions::from_meta(meta)?;
                Ok(Self {
                    source: if let Some(source) = options.source {
                        source
                    } else {
                        path.require_ident()?.clone()
                    },
                    write: options.write,
                    parse: options.parse,
                })
            }
            _ => Err(darling::Error::custom("invalid query field").with_span(item)),
        }
    }
}

impl FromMeta for ParseFieldMask {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        // Handle field_mask with single parameter: field_mask(mask_field_name)
        // Handle field_mask with parameters: field_mask { field = container, mask = mask_field }
        #[derive(FromMeta)]
        struct Options {
            field: Option<Ident>,
            mask: Ident,
        }

        // Handle empty field_mask: field_mask()
        if items.is_empty() {
            return Ok(Self {
                mask: format_ident!("update_mask"), // Default to update_mask
                field: None,                        // Will be inferred from source
            });
        }

        if items.len() == 1
            && let NestedMeta::Meta(Meta::Path(path)) = &items[0]
            && path.segments.len() == 1
        {
            let mask_ident = &path.segments[0].ident;
            return Ok(Self {
                mask: mask_ident.clone(),
                field: None, // Will be inferred from source
            });
        }

        let options = Options::from_list(items)?;
        Ok(Self {
            field: options.field,
            mask: options.mask,
        })
    }

    fn from_word() -> darling::Result<Self> {
        Ok(Self {
            mask: format_ident!("update_mask"), // Default to update_mask
            field: None,                        // Will be inferred from source
        })
    }
}

fn parse_type_path(meta: &Meta) -> darling::Result<TypePath> {
    match meta {
        Meta::NameValue(MetaNameValue { value, .. }) => match value {
            Expr::Path(path) => Ok(TypePath {
                qself: None,
                path: path.path.clone(),
            }),
            expr => TypePath::from_expr(expr),
        },
        _ => Err(darling::Error::custom("expected type path").with_span(meta)),
    }
}
