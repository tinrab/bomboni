use darling::util::parse_expr;
use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::{Ident, TokenStream};
use syn::{
    self, parse_quote, DeriveInput, Expr, ExprArray, ExprPath, Generics, Meta, MetaNameValue, Path,
    Type,
};

mod message;
mod oneof;
pub mod parse_into_map;
pub mod parse_resource_name;
mod serde;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(parse))]
pub struct ParseOptions {
    pub ident: Ident,
    pub generics: Generics,
    pub data: ast::Data<ParseVariant, ParseField>,
    /// Source proto type.
    pub source: Path,
    /// Set to true to implement `From` trait for converting parsed type back into source proto type.
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
    /// Custom serde crate.
    #[darling(default)]
    pub serde_crate: Option<Path>,
    /// Used to create tagged unions.
    #[darling(default)]
    pub tagged_union: Option<ParseTaggedUnion>,
    /// Parse list query fields.
    #[darling(default)]
    pub list_query: Option<QueryOptions>,
    /// Parse search query fields.
    #[darling(default)]
    pub search_query: Option<QueryOptions>,
    /// Marks this message as a request message.
    /// Errors will be wrapped with request's name.
    #[darling(default)]
    pub request: Option<RequestOptions>,
}

#[derive(FromMeta, Debug)]
pub struct ParseTaggedUnion {
    pub oneof: Path,
    pub field: Ident,
}

#[derive(Debug)]
pub struct RequestOptions {
    pub name: Option<Expr>,
}

#[derive(Debug, FromField)]
#[darling(attributes(parse))]
pub struct ParseField {
    pub ident: Option<Ident>,
    pub ty: Type,
    /// Source proto field name used for error reporting.
    /// Defaults to the source field name.
    #[darling(with = parse_expr::parse_str_literal, map = Some)]
    pub name: Option<Expr>,
    /// Source field name.
    /// Can be a path to a nested field.
    pub source_name: Option<String>,
    /// Skip parsing field.
    #[darling(default)]
    pub skip: bool,
    /// True if the source field should unwrapped from a `Option` type.
    #[darling(default)]
    pub source_option: bool,
    /// True if the source field should be dereferenced from a `Box` type.
    #[darling(default)]
    pub source_box: bool,
    /// Type used to convert from and into the target type.
    #[darling(default)]
    pub source_try_from: Option<Ident>,
    /// Parses enum value from `i32`.
    #[darling(default)]
    pub enumeration: bool,
    /// Parses oneof value.
    #[darling(default)]
    pub oneof: bool,
    /// Parses Protobuf's well-known wrapper type.
    #[darling(default)]
    pub wrapper: bool,
    /// True if the source and target types are the same.
    #[darling(default)]
    pub keep: bool,
    /// Parse resource fields into this field.
    #[darling(default)]
    pub resource: Option<ResourceOptions>,
    /// Custom expression that returns the default value.
    #[darling(with = parse_default_expr, map = Some)]
    pub default: Option<Expr>,
    /// String value will be checked against this regex.
    #[darling(with = parse_expr::preserve_str_literal, map = Some)]
    pub regex: Option<Expr>,
    /// Custom function that parses the source proto field.
    /// The function must have the signature `fn(source: Source) -> RequestResult<Target>`.
    #[darling(with = parse_expr::parse_str_literal, map = Some)]
    pub parse_with: Option<Expr>,
    /// Custom function that writes write source proto field.
    /// The function must have the signature `fn(target: Target) -> Source`.
    #[darling(with = parse_expr::parse_str_literal, map = Some)]
    pub write_with: Option<Expr>,
    /// Module that contains both `parse_with` and `write_with` functions.
    /// The names of the functions must be `parse` and `write` respectively.
    #[darling(with = parse_expr::parse_str_literal, map = Some)]
    pub with: Option<Expr>,
    /// Make this field derived.
    pub derive: Option<DeriveOptions>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(parse))]
pub struct ParseVariant {
    pub ident: Ident,
    pub fields: ast::Fields<Type>,
    /// Source variant name.
    pub source_name: Option<String>,
    /// Skip parsing variant.
    #[darling(default)]
    pub skip: bool,
    /// True if the source variant should unwrapped from a `Option` type.
    #[darling(default)]
    pub source_option: bool,
    /// True if the source variant should be dereferenced from a `Box` type.
    #[darling(default)]
    pub source_box: bool,
    /// True if the source is an empty unit variant.
    #[darling(default)]
    pub source_empty: bool,
    /// Type used to convert from and into the target type.
    #[darling(default)]
    pub source_try_from: Option<Ident>,
    /// Parses enum value from `i32`.
    #[darling(default)]
    pub enumeration: bool,
    /// Parses Protobuf's well-known wrapper type.
    #[darling(default)]
    pub wrapper: bool,
    /// True if the source and target use the same nested type.
    #[darling(default)]
    pub keep: bool,
    /// Custom expression that returns the default value.
    #[darling(with = parse_default_expr, map = Some)]
    pub default: Option<Expr>,
    /// String value will be checked against this regex.
    #[darling(with = parse_expr::preserve_str_literal, map = Some)]
    pub regex: Option<Expr>,
    /// Custom function that parses the source proto field.
    /// The function must have the signature `fn(source: Source) -> RequestResult<Target>`.
    #[darling(with = parse_expr::parse_str_literal, map = Some)]
    pub parse_with: Option<Expr>,
    /// Custom function that writes write source proto field.
    /// The function must have the signature `fn(target: Target) -> Source`.
    #[darling(with = parse_expr::parse_str_literal, map = Some)]
    pub write_with: Option<Expr>,
    /// Module that contains both `parse_with` and `write_with` functions.
    /// The names of the functions must be `parse` and `write` respectively.
    #[darling(with = parse_expr::parse_str_literal, map = Some)]
    pub with: Option<Expr>,
}

#[derive(Debug)]
pub struct ResourceOptions {
    pub fields: ResourceFields,
}

#[derive(Debug, Default)]
pub struct ResourceFields {
    pub name: bool,
    pub create_time: bool,
    pub update_time: bool,
    pub delete_time: bool,
    pub deleted: bool,
    pub etag: bool,
}

#[derive(Debug)]
pub struct QueryOptions {
    pub field: Ident,
    pub query: QueryFieldOptions,
    pub page_size: QueryFieldOptions,
    pub page_token: QueryFieldOptions,
    pub filter: QueryFieldOptions,
    pub ordering: QueryFieldOptions,
}

#[derive(Debug)]
pub struct QueryFieldOptions {
    pub parse: bool,
    pub source_name: Ident,
}

#[derive(Debug)]
pub struct DeriveOptions {
    /// The function must have the signature `fn(source: &Source) -> RequestResult<T>`.
    pub func: ExprPath,
    /// Optional field that will be used as the source for the function.
    /// Field name is passed in as the second argument, e.g. `fn(source: &Source, field_name: &str)`.
    pub source_field: Option<ExprPath>,
}

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let options = match ParseOptions::from_derive_input(&input) {
        Ok(v) => v,
        Err(err) => {
            return Err(err.into());
        }
    };

    let mut result = match &options.data {
        ast::Data::Struct(fields) => message::expand(&options, &fields.fields)?,
        ast::Data::Enum(variants) => oneof::expand(&options, variants)?,
    };

    result.extend(serde::expand(&options)?);

    Ok(result)
}

pub fn parse_default_expr(meta: &Meta) -> darling::Result<Expr> {
    match meta {
        Meta::Path(path) => {
            if matches!(path.get_ident(), Some(ident) if ident == "default") {
                Ok(parse_quote!(Default::default()))
            } else {
                Err(darling::Error::unsupported_format("path").with_span(meta))
            }
        }
        Meta::List(_) => Err(darling::Error::unsupported_format("list").with_span(meta)),
        Meta::NameValue(nv) => {
            if let Expr::Lit(expr_lit) = &nv.value {
                Expr::from_value(&expr_lit.lit)
            } else {
                Ok(nv.value.clone())
            }
        }
    }
}

impl FromMeta for RequestOptions {
    fn from_list(items: &[ast::NestedMeta]) -> darling::Result<Self> {
        let mut options = Self { name: None };
        for item in items {
            match item {
                ast::NestedMeta::Meta(meta) => {
                    let ident = meta.path().get_ident().unwrap();
                    match ident.to_string().as_str() {
                        "name" => {
                            if let Meta::NameValue(MetaNameValue { value, .. }) = meta {
                                options.name = Some(value.clone());
                            } else {
                                return Err(
                                    darling::Error::custom("expected name value").with_span(meta)
                                );
                            }
                        }
                        _ => {
                            return Err(
                                darling::Error::custom("unknown request option").with_span(ident)
                            );
                        }
                    }
                }
                ast::NestedMeta::Lit(lit) => {
                    return Err(darling::Error::custom("unexpected literal").with_span(lit));
                }
            }
        }
        Ok(options)
    }

    fn from_word() -> darling::Result<Self> {
        Ok(Self { name: None })
    }
}

impl FromMeta for DeriveOptions {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        Ok(match expr {
            Expr::Path(path) => Self {
                func: path.clone(),
                source_field: None,
            },
            Expr::Tuple(tuple) => {
                if tuple.elems.len() != 2 {
                    return Err(darling::Error::custom("expected tuple of size 2").with_span(tuple));
                }
                let func = match &tuple.elems[0] {
                    Expr::Path(path) => path.clone(),
                    _ => {
                        return Err(darling::Error::custom("expected function path")
                            .with_span(&tuple.elems[0]));
                    }
                };
                let source_field = match &tuple.elems[1] {
                    Expr::Path(path) => Some(path.clone()),
                    _ => {
                        return Err(darling::Error::custom("expected field ident")
                            .with_span(&tuple.elems[1]));
                    }
                };
                Self { func, source_field }
            }
            _ => {
                return Err(darling::Error::custom("expected function path").with_span(expr));
            }
        })
    }
}

impl FromMeta for ResourceOptions {
    fn from_list(items: &[ast::NestedMeta]) -> darling::Result<Self> {
        let mut fields = ResourceFields::default();
        for item in items {
            match item {
                ast::NestedMeta::Meta(meta) => {
                    let ident = meta.path().get_ident().unwrap();
                    match ident.to_string().as_str() {
                        "fields" => {
                            fields = ResourceFields::from_meta(meta)?;
                        }
                        _ => {
                            return Err(
                                darling::Error::custom("unknown resource option").with_span(ident)
                            );
                        }
                    }
                }
                ast::NestedMeta::Lit(lit) => {
                    return Err(darling::Error::custom("unexpected literal").with_span(lit));
                }
            }
        }
        Ok(Self { fields })
    }

    fn from_word() -> darling::Result<Self> {
        Ok(Self {
            fields: ResourceFields {
                name: true,
                create_time: true,
                update_time: true,
                delete_time: true,
                deleted: true,
                etag: true,
            },
        })
    }
}

impl FromMeta for ResourceFields {
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        let mut fields = ResourceFields::default();
        if let Meta::NameValue(MetaNameValue {
            value: Expr::Array(ExprArray { elems, .. }),
            ..
        }) = item
        {
            for e in elems {
                let ident = Ident::from_expr(e)?.to_string();
                match ident.as_str() {
                    "name" => fields.name = true,
                    "create_time" => fields.create_time = true,
                    "update_time" => fields.update_time = true,
                    "delete_time" => fields.delete_time = true,
                    "deleted" => fields.deleted = true,
                    "etag" => fields.etag = true,
                    _ => {
                        return Err(
                            darling::Error::custom("unknown resource field").with_span(item)
                        );
                    }
                }
            }
        } else {
            return Err(darling::Error::custom("expected array of idents").with_span(item));
        }
        Ok(fields)
    }
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            field: Ident::from_string("query").unwrap(),
            query: QueryFieldOptions {
                parse: true,
                source_name: Ident::from_string("query").unwrap(),
            },
            page_size: QueryFieldOptions {
                parse: true,
                source_name: Ident::from_string("page_size").unwrap(),
            },
            page_token: QueryFieldOptions {
                parse: true,
                source_name: Ident::from_string("page_token").unwrap(),
            },
            filter: QueryFieldOptions {
                parse: true,
                source_name: Ident::from_string("filter").unwrap(),
            },
            ordering: QueryFieldOptions {
                parse: true,
                source_name: Ident::from_string("order_by").unwrap(),
            },
        }
    }
}

impl FromMeta for QueryOptions {
    fn from_list(items: &[ast::NestedMeta]) -> darling::Result<Self> {
        let mut options = Self::default();

        macro_rules! impl_field_option {
            ($ident:ident, $meta:ident) => {
                if let Ok(parse) = bool::from_meta($meta) {
                    options.$ident.parse = parse;
                } else if let Ok(source_name) = Ident::from_meta($meta) {
                    options.$ident.source_name = source_name;
                } else {
                    return Err(darling::Error::custom(format!(
                        "invalid query `{}` option value",
                        stringify!($ident)
                    ))
                    .with_span($meta));
                }
            };
        }

        for item in items {
            match item {
                ast::NestedMeta::Meta(meta) => {
                    let ident = meta.path().get_ident().unwrap();
                    match ident.to_string().as_str() {
                        "field" => {
                            options.field = Ident::from_meta(meta)?;
                        }
                        "query" => {
                            if let Ok(source_name) = Ident::from_meta(meta) {
                                options.query.source_name = source_name;
                            } else {
                                return Err(darling::Error::custom(
                                    "invalid query `query` option value",
                                )
                                .with_span(meta));
                            }
                        }
                        "page_size" => impl_field_option!(page_size, meta),
                        "page_token" => impl_field_option!(page_token, meta),
                        "filter" => impl_field_option!(filter, meta),
                        "ordering" => impl_field_option!(ordering, meta),
                        _ => {
                            return Err(
                                darling::Error::custom("unknown query option").with_span(ident)
                            );
                        }
                    }
                }
                ast::NestedMeta::Lit(lit) => {
                    return Err(darling::Error::custom("unexpected literal").with_span(lit));
                }
            }
        }

        Ok(options)
    }

    fn from_word() -> darling::Result<Self> {
        Ok(Self::default())
    }
}
