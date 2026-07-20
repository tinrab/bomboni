#![allow(clippy::option_if_let_else, clippy::needless_continue)]

use bomboni_core::syn::{
    meta::{
        attribute_metas, meta_bool, meta_expr, meta_expr_path, meta_ident, meta_list, meta_path,
        meta_string,
    },
    type_is_phantom,
};
use proc_macro2::Ident;
use quote::{ToTokens, format_ident};
use syn::{
    self, DeriveInput, Expr, ExprArray, ExprCall, ExprLit, ExprPath, Generics, Lit, Meta, Path,
    Type, TypePath,
};

use super::field_type_info::{FieldTypeInfo, get_field_type_info};

#[derive(Debug, Clone)]
pub enum Data<V, F> {
    Struct(StructData<F>),
    Enum(Vec<V>),
}

#[derive(Debug, Clone)]
pub struct StructData<F> {
    pub fields: Vec<F>,
}

/// Main options for the Parse derive macro.
#[derive(Debug)]
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
    pub write: bool,

    /// Implement `serde::Serialize` for the source type.
    ///
    /// When set to `true`, generates a `Serialize` implementation that serializes
    /// the source type instead of the parsed type. This is useful when you want
    /// to serialize data in the original format.
    pub serialize_as: bool,

    /// Implement `serde::Deserialize` for the source type.
    ///
    /// When set to `true`, generates a `Deserialize` implementation that deserializes
    /// directly into the source type. This is useful when you want to deserialize
    /// data into the original format.
    pub deserialize_as: bool,

    /// Implement both `serde::Serialize` and `serde::Deserialize` for the source type.
    ///
    /// When set to `true`, this is a shorthand for setting both `serialize_as` and
    /// `deserialize_as` to `true`. This generates complete serde support for the source type.
    ///
    /// This is commonly used when you want full serde compatibility with the original format.
    pub serde_as: bool,

    /// Create tagged union from a oneof field.
    ///
    /// Specifies that this struct should be treated as a tagged union, where the
    /// specific variant is determined by a oneof field. This is commonly used for
    /// protobuf messages that contain oneof fields representing different message types.
    pub tagged_union: Option<ParseTaggedUnion>,

    /// Mark this message as a request message for enhanced error handling.
    /// Errors will be wrapped within `BadRequest` with [`RequestError::bad_request`].
    pub request: Option<ParseRequest>,

    /// Custom `prost` crate path.
    pub prost_crate: Option<Path>,

    /// Custom `bomboni` crate path.
    pub bomboni_crate: Option<Path>,

    /// Custom `bomboni_proto` crate path.
    pub bomboni_proto_crate: Option<Path>,

    /// Custom `bomboni_request` crate path.
    pub bomboni_request_crate: Option<Path>,

    /// Custom `serde` crate path.
    pub serde_crate: Option<Path>,
}

/// Configuration for creating tagged unions from oneof fields.
#[derive(Debug)]
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
#[derive(Debug, Clone)]
pub struct ParseField {
    /// The identifier of the field.
    pub ident: Option<Ident>,

    /// The type of the field.
    pub ty: Type,

    /// Parsing options for this field.
    pub options: ParseFieldOptions,

    /// Parse resource fields into this field.
    pub resource: Option<ParseResource>,

    /// Parse list query fields.
    pub list_query: Option<ParseQuery>,

    /// Parse search query fields.
    pub search_query: Option<ParseQuery>,

    /// Type information for the field (internal use).
    pub type_info: Option<FieldTypeInfo>,
}

/// Represents a variant in an enum that can be parsed.
#[derive(Debug, Clone)]
pub struct ParseVariant {
    /// The identifier of the variant.
    pub ident: Ident,

    /// The fields of the variant.
    pub fields: Vec<Type>,

    /// Parsing options for this variant.
    pub options: ParseFieldOptions,

    /// True if the source is an empty unit variant.
    pub source_unit: bool,

    /// Type information for the variant (internal use).
    pub type_info: Option<FieldTypeInfo>,
}

/// Options for controlling how individual fields are parsed.
#[derive(Debug, Clone, Default)]
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
    pub source: Option<String>,

    /// Indicates that the source field name is the same as the target field name.
    ///
    /// When set to `true`, this is a shorthand for `source = "<field_name>"`.
    /// This is useful when you want to explicitly indicate that a field should
    /// be parsed from a source field with the same name. This is commonly used
    /// with `derive` attribute.
    pub source_field: bool,

    /// Skip parsing this field entirely.
    ///
    /// When set to `true`, this field will be completely ignored during parsing.
    /// The field will not be read from the input and will not be included in the output.
    pub skip: bool,

    /// Keep the source and target fields the same without any parsing.
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
    pub keep_primitive: bool,

    /// Allow unspecified enum values and empty strings without treating them as required.
    ///
    /// When set to `true`, the field will not be treated as required and will accept
    /// unspecified enum values (typically 0) and empty strings without generating errors.
    pub unspecified: bool,

    /// Custom extraction plan for the field.
    ///
    /// Specifies a series of extraction steps to transform the field value.
    /// This provides fine-grained control over how values are extracted and processed.
    ///
    /// The extraction plan consists of multiple steps that are applied in sequence.
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
    pub wrapper: bool,

    /// Parse oneof value from a Protobuf oneof field.
    ///
    /// When set to `true`, indicates that this field should be parsed from a
    /// Protobuf oneof field. This is a special-purpose parse option for handling
    /// oneof fields in protobuf messages.
    ///
    /// The field will be extracted from the oneof and converted to the appropriate type.
    pub oneof: bool,

    /// Parse enum value from `i32`.
    ///
    /// When set to `true`, indicates that this field should be parsed as an enum
    /// from an `i32` value. This is a special-purpose parse option for enum fields
    /// that are represented as integers in the source data.
    pub enumeration: bool,

    /// Check string against a regular expression pattern.
    ///
    /// Specifies a regular expression that the field value must match.
    /// The field will be parsed only if the string matches the regex pattern.
    pub regex: Option<Expr>,

    /// Parse `google.protobuf.Timestamp` into a `OffsetDateTime`.
    ///
    /// When set to `true`, automatically converts protobuf timestamp fields
    /// into `OffsetDateTime` instances. This handles the conversion from the
    /// protobuf timestamp format to Rust's date/time representation.
    pub timestamp: bool,

    /// Convert field to a custom type using `try_from` or `try_into`.
    ///
    /// Specifies a custom type path that implements `TryFrom` or `TryInto`
    /// for converting the field value. The conversion can fail and returns a Result.
    ///
    /// This is useful for custom conversion logic that doesn't fit into other
    /// categories, such as domain-specific types, validation, or complex transformations.
    pub try_from: Option<TypePath>,

    /// Use custom conversion and writing functions.
    ///
    /// Specifies custom conversion functions for both parsing (reading) and writing.
    /// This provides maximum flexibility for complex field transformations.
    pub convert: Option<ParseConvert>,

    /// Make this field use derived parsing implementation.
    ///
    /// When set, indicates that this field should use a custom derived parsing
    /// implementation. This is useful for custom, non-opinionated parsing where
    /// you have full control over the parsing logic.
    pub derive: Option<ParseDerive>,

    /// Parse field only if field mask allows it.
    ///
    /// When set, indicates that this field should only be parsed if the specified
    /// field mask contains the field path. This is commonly used for update
    /// operations where only certain fields should be modified.
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

impl ParseField {
    fn from_field(field: &syn::Field) -> syn::Result<Self> {
        let mut options = ParseFieldOptions::default();
        let mut resource = None;
        let mut list_query = None;
        let mut search_query = None;
        for meta in attribute_metas(&field.attrs, "parse")? {
            if meta.path().is_ident("resource") {
                resource = Some(parse_resource(&meta)?);
            } else if meta.path().is_ident("list_query") {
                list_query = Some(parse_query(&meta)?);
            } else if meta.path().is_ident("search_query") {
                search_query = Some(parse_query(&meta)?);
            } else {
                options.apply_meta(&meta)?;
            }
        }
        Ok(Self {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            options,
            resource,
            list_query,
            search_query,
            type_info: None,
        })
    }
}

impl ParseVariant {
    fn from_variant(variant: &syn::Variant) -> syn::Result<Self> {
        let mut options = ParseFieldOptions::default();
        let mut source_unit = false;
        for meta in attribute_metas(&variant.attrs, "parse")? {
            if meta.path().is_ident("source_unit") {
                source_unit = meta_bool(&meta)?;
            } else {
                options.apply_meta(&meta)?;
            }
        }
        Ok(Self {
            ident: variant.ident.clone(),
            fields: variant
                .fields
                .iter()
                .map(|field| field.ty.clone())
                .collect(),
            options,
            source_unit,
            type_info: None,
        })
    }
}

impl ParseFieldOptions {
    fn apply_meta(&mut self, meta: &Meta) -> syn::Result<()> {
        let path = meta.path();
        if path.is_ident("source") {
            self.source = Some(meta_string(meta)?);
        } else if path.is_ident("source_field") {
            self.source_field = meta_bool(meta)?;
        } else if path.is_ident("skip") {
            self.skip = meta_bool(meta)?;
        } else if path.is_ident("keep") {
            self.keep = meta_bool(meta)?;
        } else if path.is_ident("keep_primitive") {
            self.keep_primitive = meta_bool(meta)?;
        } else if path.is_ident("unspecified") {
            self.unspecified = meta_bool(meta)?;
        } else if path.is_ident("extract") {
            self.extract = Some(parse_extract(meta)?);
        } else if path.is_ident("wrapper") {
            self.wrapper = meta_bool(meta)?;
        } else if path.is_ident("oneof") {
            self.oneof = meta_bool(meta)?;
        } else if path.is_ident("enumeration") {
            self.enumeration = meta_bool(meta)?;
        } else if path.is_ident("regex") {
            self.regex = Some(meta_expr(meta)?.clone());
        } else if path.is_ident("timestamp") {
            self.timestamp = meta_bool(meta)?;
        } else if path.is_ident("try_from") {
            self.try_from = Some(parse_type_path(meta)?);
        } else if path.is_ident("convert") {
            self.convert = Some(parse_convert(meta)?);
        } else if path.is_ident("derive") {
            self.derive = Some(parse_derive(meta)?);
        } else if path.is_ident("field_mask") {
            self.field_mask = Some(parse_field_mask(meta)?);
        } else {
            return Err(syn::Error::new_spanned(meta, "unknown parse field option"));
        }
        Ok(())
    }
}

impl ParseOptions {
    fn from_input(input: &DeriveInput) -> syn::Result<Self> {
        let mut source = None;
        let mut write = false;
        let mut serialize_as = false;
        let mut deserialize_as = false;
        let mut serde_as = false;
        let mut tagged_union = None;
        let mut request = None;
        let mut prost_crate = None;
        let mut bomboni_crate = None;
        let mut bomboni_proto_crate = None;
        let mut bomboni_request_crate = None;
        let mut serde_crate = None;

        for meta in attribute_metas(&input.attrs, "parse")? {
            let path = meta.path();
            if path.is_ident("source") {
                source = Some(meta_path(&meta)?);
            } else if path.is_ident("write") {
                write = meta_bool(&meta)?;
            } else if path.is_ident("serialize_as") {
                serialize_as = meta_bool(&meta)?;
            } else if path.is_ident("deserialize_as") {
                deserialize_as = meta_bool(&meta)?;
            } else if path.is_ident("serde_as") {
                serde_as = meta_bool(&meta)?;
            } else if path.is_ident("tagged_union") {
                tagged_union = Some(parse_tagged_union(&meta)?);
            } else if path.is_ident("request") {
                request = Some(parse_request(&meta)?);
            } else if path.is_ident("prost_crate") {
                prost_crate = Some(meta_path(&meta)?);
            } else if path.is_ident("bomboni_crate") {
                bomboni_crate = Some(meta_path(&meta)?);
            } else if path.is_ident("bomboni_proto_crate") {
                bomboni_proto_crate = Some(meta_path(&meta)?);
            } else if path.is_ident("bomboni_request_crate") {
                bomboni_request_crate = Some(meta_path(&meta)?);
            } else if path.is_ident("serde_crate") {
                serde_crate = Some(meta_path(&meta)?);
            } else {
                return Err(syn::Error::new_spanned(meta, "unknown parse option"));
            }
        }

        let data = match &input.data {
            syn::Data::Struct(data) => Data::Struct(StructData {
                fields: data
                    .fields
                    .iter()
                    .map(ParseField::from_field)
                    .collect::<syn::Result<_>>()?,
            }),
            syn::Data::Enum(data) => Data::Enum(
                data.variants
                    .iter()
                    .map(ParseVariant::from_variant)
                    .collect::<syn::Result<_>>()?,
            ),
            syn::Data::Union(_) => {
                return Err(syn::Error::new_spanned(input, "unions are not supported"));
            }
        };

        Ok(Self {
            ident: input.ident.clone(),
            generics: input.generics.clone(),
            data,
            source: source.ok_or_else(|| syn::Error::new_spanned(input, "missing `source`"))?,
            write,
            serialize_as,
            deserialize_as,
            serde_as,
            tagged_union,
            request,
            prost_crate,
            bomboni_crate,
            bomboni_proto_crate,
            bomboni_request_crate,
            serde_crate,
        })
    }

    pub fn parse(input: &DeriveInput) -> syn::Result<Self> {
        let mut options = Self::from_input(input)?;

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

                Data::Struct(StructData { fields })
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

                    match variant.fields.first() {
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

fn parse_tagged_union(meta: &Meta) -> syn::Result<ParseTaggedUnion> {
    let Meta::List(list) = meta else {
        return Err(syn::Error::new_spanned(
            meta,
            "expected tagged_union options",
        ));
    };
    let mut oneof = None;
    let mut field = None;
    for option in meta_list(list)? {
        if option.path().is_ident("oneof") {
            oneof = Some(meta_path(&option)?);
        } else if option.path().is_ident("field") {
            field = Some(meta_ident(&option)?);
        } else {
            return Err(syn::Error::new_spanned(
                option,
                "invalid tagged_union option",
            ));
        }
    }
    Ok(ParseTaggedUnion {
        oneof: oneof.ok_or_else(|| syn::Error::new_spanned(meta, "missing `oneof`"))?,
        field: field.ok_or_else(|| syn::Error::new_spanned(meta, "missing `field`"))?,
    })
}

fn parse_request(meta: &Meta) -> syn::Result<ParseRequest> {
    let mut name = None;
    if let Meta::List(list) = meta {
        for option in meta_list(list)? {
            if option.path().is_ident("name") {
                name = Some(meta_expr(&option)?.clone());
            } else {
                return Err(syn::Error::new_spanned(option, "invalid request option"));
            }
        }
    } else if !matches!(meta, Meta::Path(_)) {
        return Err(syn::Error::new_spanned(meta, "expected request options"));
    }
    Ok(ParseRequest { name })
}

fn parse_extract(meta: &Meta) -> syn::Result<FieldExtract> {
    let Expr::Array(ExprArray { elems, .. }) = meta_expr(meta)? else {
        return Err(syn::Error::new_spanned(
            meta,
            "expected an extraction array",
        ));
    };
    Ok(FieldExtract {
        steps: elems
            .iter()
            .map(parse_extract_step)
            .collect::<syn::Result<_>>()?,
    })
}

fn parse_extract_step(expr: &Expr) -> syn::Result<FieldExtractStep> {
    match expr {
        Expr::Path(ExprPath { path, .. }) if path.is_ident("Unwrap") => {
            Ok(FieldExtractStep::Unwrap)
        }
        Expr::Path(ExprPath { path, .. }) if path.is_ident("UnwrapOrDefault") => {
            Ok(FieldExtractStep::UnwrapOrDefault)
        }
        Expr::Path(ExprPath { path, .. }) if path.is_ident("Unbox") => Ok(FieldExtractStep::Unbox),
        Expr::Path(ExprPath { path, .. }) if path.is_ident("StringFilterEmpty") => {
            Ok(FieldExtractStep::StringFilterEmpty)
        }
        Expr::Path(ExprPath { path, .. }) if path.is_ident("EnumerationFilterUnspecified") => {
            Ok(FieldExtractStep::EnumerationFilterUnspecified)
        }
        Expr::Call(ExprCall { func, args, .. }) if matches!(&**func, Expr::Path(path) if path.path.is_ident("Field")) =>
        {
            if args.len() != 1 {
                return Err(syn::Error::new_spanned(args, "expected one argument"));
            }
            let Expr::Lit(ExprLit {
                lit: Lit::Str(value),
                ..
            }) = &args[0]
            else {
                return Err(syn::Error::new_spanned(&args[0], "expected a field name"));
            };
            let value = value.value();
            if value.contains('.') || value.contains('?') {
                return Err(syn::Error::new_spanned(&args[0], "invalid field name"));
            }
            Ok(FieldExtractStep::Field(value))
        }
        Expr::Call(ExprCall { func, args, .. }) if matches!(&**func, Expr::Path(path) if path.path.is_ident("UnwrapOr")) =>
        {
            if args.len() != 1 {
                return Err(syn::Error::new_spanned(args, "expected one argument"));
            }
            Ok(FieldExtractStep::UnwrapOr(args[0].clone()))
        }
        _ => Err(syn::Error::new_spanned(expr, "invalid extract step")),
    }
}

fn parse_derive(meta: &Meta) -> syn::Result<ParseDerive> {
    if let Meta::NameValue(_) = meta {
        return Ok(ParseDerive {
            parse: None,
            write: None,
            module: Some(meta_expr_path(meta)?),
            source_borrow: false,
            target_borrow: false,
        });
    }
    let Meta::List(list) = meta else {
        return Err(syn::Error::new_spanned(meta, "expected derive options"));
    };
    let mut value = ParseDerive {
        parse: None,
        write: None,
        module: None,
        source_borrow: false,
        target_borrow: false,
    };
    let mut borrow = false;
    for option in meta_list(list)? {
        if option.path().is_ident("parse") {
            value.parse = Some(meta_expr_path(&option)?);
        } else if option.path().is_ident("write") {
            value.write = Some(meta_expr_path(&option)?);
        } else if option.path().is_ident("module") {
            value.module = Some(meta_expr_path(&option)?);
        } else if option.path().is_ident("source_borrow") {
            value.source_borrow = meta_bool(&option)?;
        } else if option.path().is_ident("target_borrow") {
            value.target_borrow = meta_bool(&option)?;
        } else if option.path().is_ident("borrow") {
            borrow = meta_bool(&option)?;
        } else {
            return Err(syn::Error::new_spanned(option, "invalid derive option"));
        }
    }
    value.source_borrow |= borrow;
    value.target_borrow |= borrow;
    if value.parse.is_none() && value.write.is_none() && value.module.is_none()
        || value.module.is_some() && (value.parse.is_some() || value.write.is_some())
    {
        return Err(syn::Error::new_spanned(meta, "invalid derive options"));
    }
    Ok(value)
}

fn parse_convert(meta: &Meta) -> syn::Result<ParseConvert> {
    if let Meta::NameValue(_) = meta {
        return Ok(ParseConvert {
            parse: None,
            write: None,
            module: Some(meta_expr_path(meta)?),
        });
    }
    let Meta::List(list) = meta else {
        return Err(syn::Error::new_spanned(meta, "expected convert options"));
    };
    let mut value = ParseConvert {
        parse: None,
        write: None,
        module: None,
    };
    for option in meta_list(list)? {
        if option.path().is_ident("parse") {
            value.parse = Some(meta_expr_path(&option)?);
        } else if option.path().is_ident("write") {
            value.write = Some(meta_expr_path(&option)?);
        } else if option.path().is_ident("module") {
            value.module = Some(meta_expr_path(&option)?);
        } else {
            return Err(syn::Error::new_spanned(option, "invalid convert option"));
        }
    }
    if value.parse.is_none() && value.write.is_none() && value.module.is_none()
        || value.module.is_some() && (value.parse.is_some() || value.write.is_some())
    {
        return Err(syn::Error::new_spanned(meta, "invalid convert options"));
    }
    Ok(value)
}

fn parse_resource(meta: &Meta) -> syn::Result<ParseResource> {
    let mut resource = ParseResource::default();
    if matches!(meta, Meta::Path(_)) {
        return Ok(resource);
    }
    let Meta::List(list) = meta else {
        return Err(syn::Error::new_spanned(meta, "expected resource options"));
    };
    for option in meta_list(list)? {
        let Meta::List(fields) = &option else {
            return Err(syn::Error::new_spanned(option, "expected resource fields"));
        };
        if !fields.path.is_ident("fields") {
            return Err(syn::Error::new_spanned(option, "expected `fields`"));
        }
        for field in meta_list(fields)? {
            let value = parse_resource_field(&field)?;
            let path = field.path();
            if path.is_ident("name") {
                resource.name = value;
            } else if path.is_ident("create_time") {
                resource.create_time = value;
            } else if path.is_ident("update_time") {
                resource.update_time = value;
            } else if path.is_ident("delete_time") {
                resource.delete_time = value;
            } else if path.is_ident("deleted") {
                resource.deleted = value;
            } else if path.is_ident("etag") {
                resource.etag = value;
            } else {
                return Err(syn::Error::new_spanned(field, "unknown resource field"));
            }
        }
    }
    Ok(resource)
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

fn parse_resource_field(meta: &Meta) -> syn::Result<ParseResourceField> {
    let default_source = meta
        .path()
        .get_ident()
        .cloned()
        .ok_or_else(|| syn::Error::new_spanned(meta, "expected a resource field"))?;
    match meta {
        Meta::NameValue(_) => {
            let include = meta_bool(meta)?;
            Ok(ParseResourceField {
                source: default_source,
                write: include,
                parse: include,
            })
        }
        Meta::List(list) => {
            let mut source = default_source;
            let mut parse = false;
            let mut write = false;
            for option in meta_list(list)? {
                if option.path().is_ident("source") {
                    source = meta_ident(&option)?;
                } else if option.path().is_ident("parse") {
                    parse = meta_bool(&option)?;
                } else if option.path().is_ident("write") {
                    write = meta_bool(&option)?;
                } else {
                    return Err(syn::Error::new_spanned(
                        option,
                        "invalid resource field option",
                    ));
                }
            }
            Ok(ParseResourceField {
                parse,
                write,
                source,
            })
        }
        Meta::Path(_) => Ok(ParseResourceField {
            source: default_source,
            parse: true,
            write: true,
        }),
    }
}

fn parse_query(meta: &Meta) -> syn::Result<ParseQuery> {
    let mut query = ParseQuery::default();
    if matches!(meta, Meta::Path(_)) {
        return Ok(query);
    }
    let Meta::List(list) = meta else {
        return Err(syn::Error::new_spanned(meta, "expected query options"));
    };
    for field in meta_list(list)? {
        let value = parse_query_field(&field)?;
        let path = field.path();
        if path.is_ident("query") {
            query.query = value;
        } else if path.is_ident("page_size") {
            query.page_size = value;
        } else if path.is_ident("page_token") {
            query.page_token = value;
        } else if path.is_ident("filter") {
            query.filter = value;
        } else if path.is_ident("order_by") {
            query.order_by = value;
        } else {
            return Err(syn::Error::new_spanned(field, "unknown query field"));
        }
    }
    Ok(query)
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

fn parse_query_field(meta: &Meta) -> syn::Result<ParseQueryField> {
    let default_source = meta
        .path()
        .get_ident()
        .cloned()
        .ok_or_else(|| syn::Error::new_spanned(meta, "expected a query field"))?;
    match meta {
        Meta::NameValue(_) => {
            let include = meta_bool(meta)?;
            Ok(ParseQueryField {
                source: default_source,
                write: include,
                parse: include,
            })
        }
        Meta::List(list) => {
            let mut source = default_source;
            let mut parse = false;
            let mut write = false;
            for option in meta_list(list)? {
                if option.path().is_ident("source") {
                    source = meta_ident(&option)?;
                } else if option.path().is_ident("parse") {
                    parse = meta_bool(&option)?;
                } else if option.path().is_ident("write") {
                    write = meta_bool(&option)?;
                } else {
                    return Err(syn::Error::new_spanned(
                        option,
                        "invalid query field option",
                    ));
                }
            }
            Ok(ParseQueryField {
                parse,
                write,
                source,
            })
        }
        Meta::Path(_) => Ok(ParseQueryField {
            source: default_source,
            parse: true,
            write: true,
        }),
    }
}

fn parse_field_mask(meta: &Meta) -> syn::Result<ParseFieldMask> {
    let default = || ParseFieldMask {
        mask: format_ident!("update_mask"),
        field: None,
    };
    if matches!(meta, Meta::Path(_)) {
        return Ok(default());
    }
    let Meta::List(list) = meta else {
        return Err(syn::Error::new_spanned(meta, "expected field_mask options"));
    };
    let items = meta_list(list)?;
    if items.is_empty() {
        return Ok(default());
    }
    if items.len() == 1
        && let Meta::Path(path) = &items[0]
        && let Some(mask) = path.get_ident()
    {
        return Ok(ParseFieldMask {
            mask: mask.clone(),
            field: None,
        });
    }
    let mut field = None;
    let mut mask = None;
    for option in items {
        if option.path().is_ident("field") {
            field = Some(meta_ident(&option)?);
        } else if option.path().is_ident("mask") {
            mask = Some(meta_ident(&option)?);
        } else {
            return Err(syn::Error::new_spanned(option, "invalid field_mask option"));
        }
    }
    Ok(ParseFieldMask {
        field,
        mask: mask.ok_or_else(|| syn::Error::new_spanned(meta, "missing `mask`"))?,
    })
}

fn parse_type_path(meta: &Meta) -> syn::Result<TypePath> {
    syn::parse2(meta_expr(meta)?.to_token_stream())
        .map_err(|_| syn::Error::new_spanned(meta, "expected type path"))
}
