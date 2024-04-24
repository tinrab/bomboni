use darling::{
    ast::{Data, Fields, NestedMeta},
    util::parse_expr,
    FromDeriveInput, FromField, FromMeta, FromVariant,
};
use proc_macro2::Ident;
use quote::format_ident;
use syn::{
    self, parse_quote, Expr, ExprArray, ExprCall, ExprPath, Generics, LitBool, LitStr, Meta,
    MetaList, MetaNameValue, Path, Type,
};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(parse), supports(struct_any, enum_any))]
pub struct ParseOptions {
    pub ident: Ident,
    pub generics: Generics,
    pub data: Data<ParseVariant, ParseField>,

    /// Source type.
    pub source: Path,
    /// Set to true to implement `From` trait for converting parsed type back into source type.
    #[darling(default)]
    pub write: bool,
    /// Implement `serde::Serialize` from source type.
    #[darling(default)]
    pub serialize_as: bool,
    /// Implement `serde::Deserialize` from source type.
    #[darling(default)]
    pub deserialize_as: bool,
    /// Implement `serde::Serialize` and `serde::Deserialize` from source type.
    #[darling(default)]
    pub serde_as: bool,
    /// Used to create tagged unions.
    #[darling(default)]
    pub tagged_union: Option<ParseTaggedUnion>,
    /// Marks this message as a request message.
    /// Errors will be wrapped with request's name.
    #[darling(default)]
    pub request: Option<ParseRequest>,

    /// Custom comboni crate path.
    #[darling(default)]
    pub bomboni_crate: Option<Path>,
    /// Custom serde crate path.
    #[darling(default)]
    pub serde_crate: Option<Path>,
}

#[derive(Debug, FromMeta)]
pub struct ParseTaggedUnion {
    pub oneof: Path,
    pub field: Ident,
}

#[derive(Debug)]
pub struct ParseRequest {
    pub name: Option<Expr>,
}

#[derive(Debug, FromField)]
#[darling(attributes(parse))]
pub struct ParseField {
    pub ident: Option<Ident>,
    pub ty: Type,
    #[darling(flatten)]
    pub options: ParseFieldOptions,

    /// Parses oneof value.
    /// Special purpose parse for oneof fields.
    #[darling(default)]
    pub oneof: bool,
    /// Parse resource fields into this field.
    /// Special purpose parse for resource fields into a `ParsedResource` field.
    pub resource: Option<ParseResource>,
    /// Parse list query fields.
    #[darling(default)]
    pub list_query: Option<ParseQuery>,
    /// Parse search query fields.
    #[darling(default)]
    pub search_query: Option<ParseQuery>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(parse))]
pub struct ParseVariant {
    pub ident: Ident,
    pub fields: Fields<Type>,
    #[darling(flatten)]
    pub options: ParseFieldOptions,

    /// True if the source is an empty unit variant.
    #[darling(default)]
    pub source_unit: bool,
}

#[derive(Debug, FromMeta)]
pub struct ParseFieldOptions {
    /// Source field name.
    /// Can be a path to a nested field with conditional `?.` extraction.
    /// Example: `bio` or `address?.city`
    #[darling(default)]
    pub source: Option<String>,
    /// Skip parsing field.
    #[darling(default)]
    pub skip: bool,
    /// True to keep source and target fields the same.
    /// No parsing will be done.
    #[darling(default)]
    pub keep: bool,
    /// True to keep source and target primitive message types the same.
    /// Only surrounding container will be extracted and parsed.
    #[darling(default)]
    pub keep_primitive: bool,
    /// Extraction plan for the field.
    #[darling(default)]
    pub extract: Option<FieldExtract>,
    /// Parses Protobuf's well-known wrapper type.
    ///
    /// Types are mapped as follows:
    ///
    /// - `String` -> `StringValue`
    /// - `bool` -> `BoolValue`
    /// - `f32` -> `FloatValue`
    /// - `f64` -> `DoubleValue`
    /// - `i8`, `i16`, `i32` -> `Int32Value`
    /// - `u8`, `u16`, `u32` -> `UInt32Value`
    /// - `i64`, `isize` -> `Int64Value`
    /// - `u64`, `usize` -> `UInt64Value`
    ///
    #[darling(default)]
    pub wrapper: bool,
    /// Parses enum value from `i32`.
    /// Special purpose parse for enum fields with `i32` values.
    #[darling(default)]
    pub enumeration: bool,
    /// Check string against RegEx.
    #[darling(with = parse_expr::preserve_str_literal, map = Some)]
    pub regex: Option<Expr>,
    /// Make this field derived.
    /// Use this for custom, non-opinionated parsing.
    pub derive: Option<ParseDerive>,
}

#[derive(Debug, Clone)]
pub struct FieldExtract {
    pub steps: Vec<FieldExtractStep>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldExtractStep {
    Field(String),
    Unwrap,
    UnwrapOr(Expr),
    UnwrapOrDefault,
    Unbox,
    StringFilterEmpty,
    EnumerationFilterUnspecified,
}

#[derive(Debug)]
pub struct ParseDerive {
    pub parse: Option<ExprPath>,
    pub write: Option<ExprPath>,
    pub module: Option<ExprPath>,
    pub source_field: Option<Ident>,
    pub target_field: Option<Ident>,
    pub borrowed: bool,
}

#[derive(Debug)]
pub struct ParseResource {
    pub name: ParseResourceField,
    pub create_time: ParseResourceField,
    pub update_time: ParseResourceField,
    pub delete_time: ParseResourceField,
    pub deleted: ParseResourceField,
    pub etag: ParseResourceField,
}

#[derive(Debug)]
pub struct ParseResourceField {
    pub parse: bool,
    pub write: bool,
    pub source: Ident,
}

#[derive(Debug)]
pub struct ParseQuery {
    pub query: ParseQueryField,
    pub page_size: ParseQueryField,
    pub page_token: ParseQueryField,
    pub filter: ParseQueryField,
    pub order_by: ParseQueryField,
}

#[derive(Debug)]
pub struct ParseQueryField {
    pub parse: bool,
    pub write: bool,
    pub source: Ident,
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
            field: Option<Ident>,
            #[darling(default)]
            source_field: Option<Ident>,
            #[darling(default)]
            target_field: Option<Ident>,
            #[darling(default)]
            borrowed: bool,
        }

        let options = Options::from_list(items)?;

        if options.parse.is_none() && options.write.is_none() && options.module.is_none()
            || options.module.is_some() && (options.parse.is_some() || options.write.is_some())
            || options.field.is_some()
                && (options.source_field.is_some() || options.target_field.is_some())
        {
            return Err(darling::Error::custom("invalid options"));
        }

        Ok(Self {
            parse: options.parse,
            write: options.write,
            module: options.module,
            source_field: options.source_field.or(options.field.clone()),
            target_field: options.target_field.or(options.field),
            borrowed: options.borrowed,
        })
    }

    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(path) => Ok(Self {
                parse: None,
                write: None,
                module: Some(path.clone()),
                source_field: None,
                target_field: None,
                borrowed: false,
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
