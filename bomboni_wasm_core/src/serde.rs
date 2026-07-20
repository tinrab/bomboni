use bomboni_core::syn::meta::{
    attribute_metas, meta_flag, meta_list, meta_string, meta_string_path,
};
use syn::{
    Attribute, Data as SynData, DeriveInput, Fields, Generics, Ident, Member, Meta, Path, Type,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Struct,
    Tuple,
    Newtype,
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagType {
    External,
    Internal { tag: String },
    Adjacent { tag: String, content: String },
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameRule {
    None,
    LowerCase,
    UpperCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl RenameRule {
    pub fn parse(value: &str, span: impl quote::ToTokens) -> syn::Result<Self> {
        match value {
            "lowercase" => Ok(Self::LowerCase),
            "UPPERCASE" => Ok(Self::UpperCase),
            "PascalCase" => Ok(Self::PascalCase),
            "camelCase" => Ok(Self::CamelCase),
            "snake_case" => Ok(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
            "kebab-case" => Ok(Self::KebabCase),
            "SCREAMING-KEBAB-CASE" => Ok(Self::ScreamingKebabCase),
            _ => Err(syn::Error::new_spanned(
                span,
                format!("unknown rename rule `{value}`"),
            )),
        }
    }

    pub fn apply_to_variant(self, variant: &str) -> String {
        match self {
            Self::None | Self::PascalCase => variant.to_owned(),
            Self::LowerCase => variant.to_ascii_lowercase(),
            Self::UpperCase => variant.to_ascii_uppercase(),
            Self::CamelCase => lowercase_first(variant),
            Self::SnakeCase => variant_to_snake_case(variant),
            Self::ScreamingSnakeCase => variant_to_snake_case(variant).to_ascii_uppercase(),
            Self::KebabCase => variant_to_snake_case(variant).replace('_', "-"),
            Self::ScreamingKebabCase => variant_to_snake_case(variant)
                .to_ascii_uppercase()
                .replace('_', "-"),
        }
    }

    pub fn apply_to_field(self, field: &str) -> String {
        match self {
            Self::None | Self::LowerCase | Self::SnakeCase => field.to_owned(),
            Self::UpperCase | Self::ScreamingSnakeCase => field.to_ascii_uppercase(),
            Self::PascalCase => field_to_pascal_case(field),
            Self::CamelCase => lowercase_first(&field_to_pascal_case(field)),
            Self::KebabCase => field.replace('_', "-"),
            Self::ScreamingKebabCase => field.to_ascii_uppercase().replace('_', "-"),
        }
    }
}

fn lowercase_first(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        std::iter::once(first.to_ascii_lowercase())
            .chain(chars)
            .collect()
    })
}

fn variant_to_snake_case(value: &str) -> String {
    let mut snake = String::with_capacity(value.len());
    for (index, ch) in value.char_indices() {
        if index > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}

fn field_to_pascal_case(value: &str) -> String {
    let mut pascal = String::with_capacity(value.len());
    let mut capitalize = true;
    for ch in value.chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            pascal.push(ch.to_ascii_uppercase());
            capitalize = false;
        } else {
            pascal.push(ch);
        }
    }
    pascal
}

#[derive(Debug)]
pub struct Container<'a> {
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub attrs: ContainerAttrs,
    pub data: Data<'a>,
}

#[derive(Debug)]
pub enum Data<'a> {
    Struct(Style, Vec<Field<'a>>),
    Enum(Vec<Variant<'a>>),
}

#[derive(Debug)]
pub struct Field<'a> {
    pub member: Member,
    pub attrs: FieldAttrs,
    pub ty: &'a Type,
}

#[derive(Debug)]
pub struct Variant<'a> {
    pub ident: &'a Ident,
    pub attrs: VariantAttrs,
    pub style: Style,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug)]
pub struct ContainerAttrs {
    pub name: String,
    pub tag: TagType,
    pub transparent: bool,
    pub default: bool,
    pub custom_crate: Option<Path>,
}

#[derive(Debug)]
pub struct FieldAttrs {
    pub name: String,
    pub skip_serializing: bool,
    pub skip_deserializing: bool,
    pub flatten: bool,
    pub default: bool,
    pub skip_serializing_if: Option<Path>,
}

#[derive(Debug)]
pub struct VariantAttrs {
    pub name: String,
    pub skip_serializing: bool,
    pub skip_deserializing: bool,
}

#[derive(Default)]
struct RawContainerAttrs {
    rename: Option<String>,
    rename_all: Option<RenameRule>,
    rename_all_fields: Option<RenameRule>,
    tag: Option<String>,
    content: Option<String>,
    untagged: bool,
    transparent: bool,
    default: bool,
    custom_crate: Option<Path>,
}

#[derive(Default)]
struct RawVariantAttrs {
    rename: Option<String>,
    rename_all: Option<RenameRule>,
    skip_serializing: bool,
    skip_deserializing: bool,
}

#[derive(Default)]
struct RawFieldAttrs {
    rename: Option<String>,
    skip_serializing: bool,
    skip_deserializing: bool,
    flatten: bool,
    default: bool,
    skip_serializing_if: Option<Path>,
}

impl<'a> Container<'a> {
    pub fn from_ast(input: &'a DeriveInput) -> syn::Result<Self> {
        let raw = RawContainerAttrs::parse(&input.attrs)?;
        let tag = raw.tag_type(input)?;
        let name = raw
            .rename
            .clone()
            .unwrap_or_else(|| ident_name(&input.ident));

        let data = match &input.data {
            SynData::Struct(data) => {
                let (style, fields) = parse_fields(&data.fields, raw.rename_all)?;
                Data::Struct(style, fields)
            }
            SynData::Enum(data) => {
                let variants = data
                    .variants
                    .iter()
                    .map(|variant| {
                        let attrs = RawVariantAttrs::parse(&variant.attrs)?;
                        let variant_name = attrs.rename.clone().unwrap_or_else(|| {
                            raw.rename_all.map_or_else(
                                || ident_name(&variant.ident),
                                |rule| rule.apply_to_variant(&ident_name(&variant.ident)),
                            )
                        });
                        let field_rule = attrs.rename_all.or(raw.rename_all_fields);
                        let (style, fields) = parse_fields(&variant.fields, field_rule)?;
                        Ok(Variant {
                            ident: &variant.ident,
                            attrs: VariantAttrs {
                                name: variant_name,
                                skip_serializing: attrs.skip_serializing,
                                skip_deserializing: attrs.skip_deserializing,
                            },
                            style,
                            fields,
                        })
                    })
                    .collect::<syn::Result<Vec<_>>>()?;
                Data::Enum(variants)
            }
            SynData::Union(_) => {
                return Err(syn::Error::new_spanned(
                    input,
                    "WASM cannot be derived for a union",
                ));
            }
        };

        if raw.transparent && !matches!(data, Data::Struct(_, _)) {
            return Err(syn::Error::new_spanned(
                input,
                "#[serde(transparent)] is only supported on structs",
            ));
        }

        Ok(Self {
            ident: &input.ident,
            generics: &input.generics,
            attrs: ContainerAttrs {
                name,
                tag,
                transparent: raw.transparent,
                default: raw.default,
                custom_crate: raw.custom_crate,
            },
            data,
        })
    }
}

impl RawContainerAttrs {
    fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = Self::default();
        for meta in attribute_metas(attrs, "serde")? {
            let path = meta.path();
            if path.is_ident("rename") {
                result.rename = parse_serialize_string(&meta)?;
            } else if path.is_ident("rename_all") {
                result.rename_all = parse_serialize_rename_rule(&meta)?;
            } else if path.is_ident("rename_all_fields") {
                result.rename_all_fields = parse_serialize_rename_rule(&meta)?;
            } else if path.is_ident("tag") {
                result.tag = Some(meta_string(&meta)?);
            } else if path.is_ident("content") {
                result.content = Some(meta_string(&meta)?);
            } else if path.is_ident("untagged") {
                result.untagged = meta_flag(&meta)?;
            } else if path.is_ident("transparent") {
                result.transparent = meta_flag(&meta)?;
            } else if path.is_ident("default") {
                result.default = true;
            } else if path.is_ident("crate") {
                result.custom_crate = Some(meta_string_path(&meta)?);
            }
        }
        Ok(result)
    }

    fn tag_type(&self, input: &DeriveInput) -> syn::Result<TagType> {
        if self.untagged {
            if self.tag.is_some() || self.content.is_some() {
                return Err(syn::Error::new_spanned(
                    input,
                    "untagged enum cannot also specify tag or content",
                ));
            }
            return Ok(TagType::None);
        }
        match (&self.tag, &self.content) {
            (None, None) => Ok(TagType::External),
            (Some(tag), None) => Ok(TagType::Internal { tag: tag.clone() }),
            (Some(tag), Some(content)) => Ok(TagType::Adjacent {
                tag: tag.clone(),
                content: content.clone(),
            }),
            (None, Some(_)) => Err(syn::Error::new_spanned(
                input,
                "#[serde(content = ...)] requires #[serde(tag = ...)]",
            )),
        }
    }
}

impl RawVariantAttrs {
    fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = Self::default();
        for meta in attribute_metas(attrs, "serde")? {
            let path = meta.path();
            if path.is_ident("rename") {
                result.rename = parse_serialize_string(&meta)?;
            } else if path.is_ident("rename_all") {
                result.rename_all = parse_serialize_rename_rule(&meta)?;
            } else if path.is_ident("skip") {
                meta_flag(&meta)?;
                result.skip_serializing = true;
                result.skip_deserializing = true;
            } else if path.is_ident("skip_serializing") {
                result.skip_serializing = meta_flag(&meta)?;
            } else if path.is_ident("skip_deserializing") {
                result.skip_deserializing = meta_flag(&meta)?;
            }
        }
        Ok(result)
    }
}

impl RawFieldAttrs {
    fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = Self::default();
        for meta in attribute_metas(attrs, "serde")? {
            let path = meta.path();
            if path.is_ident("rename") {
                result.rename = parse_serialize_string(&meta)?;
            } else if path.is_ident("skip") {
                meta_flag(&meta)?;
                result.skip_serializing = true;
                result.skip_deserializing = true;
            } else if path.is_ident("skip_serializing") {
                result.skip_serializing = meta_flag(&meta)?;
            } else if path.is_ident("skip_deserializing") {
                result.skip_deserializing = meta_flag(&meta)?;
            } else if path.is_ident("flatten") {
                result.flatten = meta_flag(&meta)?;
            } else if path.is_ident("default") {
                result.default = true;
            } else if path.is_ident("skip_serializing_if") {
                result.skip_serializing_if = Some(meta_string_path(&meta)?);
            }
        }
        Ok(result)
    }
}

fn parse_fields(
    fields: &Fields,
    rename_all: Option<RenameRule>,
) -> syn::Result<(Style, Vec<Field<'_>>)> {
    let style = match fields {
        Fields::Named(_) => Style::Struct,
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Style::Newtype,
        Fields::Unnamed(_) => Style::Tuple,
        Fields::Unit => Style::Unit,
    };
    let parsed = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let raw = RawFieldAttrs::parse(&field.attrs)?;
            let member = field.ident.as_ref().map_or_else(
                || Member::Unnamed(index.into()),
                |ident| Member::Named(ident.clone()),
            );
            let rust_name = field
                .ident
                .as_ref()
                .map_or_else(|| index.to_string(), ident_name);
            let name = raw.rename.unwrap_or_else(|| {
                rename_all.map_or_else(|| rust_name.clone(), |rule| rule.apply_to_field(&rust_name))
            });
            Ok(Field {
                member,
                attrs: FieldAttrs {
                    name,
                    skip_serializing: raw.skip_serializing,
                    skip_deserializing: raw.skip_deserializing,
                    flatten: raw.flatten,
                    default: raw.default,
                    skip_serializing_if: raw.skip_serializing_if,
                },
                ty: &field.ty,
            })
        })
        .collect::<syn::Result<_>>()?;
    Ok((style, parsed))
}

fn ident_name(ident: &Ident) -> String {
    let name = ident.to_string();
    name.strip_prefix("r#").unwrap_or(&name).to_owned()
}

fn parse_serialize_string(meta: &Meta) -> syn::Result<Option<String>> {
    match meta {
        Meta::NameValue(_) => Ok(Some(meta_string(meta)?)),
        Meta::List(list) => {
            let mut serialize = None;
            for nested in meta_list(list)? {
                if nested.path().is_ident("serialize") {
                    serialize = Some(meta_string(&nested)?);
                }
            }
            Ok(serialize)
        }
        Meta::Path(_) => Err(syn::Error::new_spanned(meta, "expected rename value")),
    }
}

fn parse_serialize_rename_rule(meta: &Meta) -> syn::Result<Option<RenameRule>> {
    match meta {
        Meta::NameValue(_) => {
            let value = meta_string(meta)?;
            Ok(Some(RenameRule::parse(&value, meta)?))
        }
        Meta::List(list) => {
            for nested in meta_list(list)? {
                if nested.path().is_ident("serialize") {
                    let value = meta_string(&nested)?;
                    return Ok(Some(RenameRule::parse(&value, &nested)?));
                }
            }
            Ok(None)
        }
        Meta::Path(_) => Err(syn::Error::new_spanned(meta, "expected rename rule")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn parses_serialized_shape() {
        let input: DeriveInput = parse_quote! {
            #[serde(rename = "ApiItem", rename_all = "camelCase", default)]
            struct Item {
                first_name: String,
                #[serde(rename(serialize = "id", deserialize = "identifier"))]
                item_id: u64,
                #[serde(default, flatten, skip_serializing_if = "Option::is_none")]
                metadata: Option<Metadata>,
            }
        };
        let container = Container::from_ast(&input).unwrap();
        assert_eq!(container.attrs.name, "ApiItem");
        assert!(container.attrs.default);
        let Data::Struct(Style::Struct, fields) = container.data else {
            panic!("expected struct");
        };
        assert_eq!(fields[0].attrs.name, "firstName");
        assert_eq!(fields[1].attrs.name, "id");
        assert!(fields[2].attrs.default);
        assert!(fields[2].attrs.flatten);
        assert_eq!(
            fields[2]
                .attrs
                .skip_serializing_if
                .as_ref()
                .unwrap()
                .segments
                .last()
                .unwrap()
                .ident,
            "is_none"
        );
    }

    #[test]
    fn parses_enum_tagging_and_renames() {
        let input: DeriveInput = parse_quote! {
            #[serde(tag = "kind", content = "value", rename_all = "snake_case")]
            enum Event {
                HttpRequest { response_code: u16 },
                #[serde(rename = "done")]
                Complete,
                #[serde(skip)]
                Hidden,
            }
        };
        let container = Container::from_ast(&input).unwrap();
        assert_eq!(
            container.attrs.tag,
            TagType::Adjacent {
                tag: "kind".into(),
                content: "value".into()
            }
        );
        let Data::Enum(variants) = container.data else {
            panic!("expected enum");
        };
        assert_eq!(variants[0].attrs.name, "http_request");
        assert_eq!(variants[1].attrs.name, "done");
        assert!(variants[2].attrs.skip_serializing);
        assert!(variants[2].attrs.skip_deserializing);
    }

    #[test]
    fn matches_serde_case_rules_and_unraws_identifiers() {
        assert_eq!(
            RenameRule::SnakeCase.apply_to_variant("HTTPRequest"),
            "h_t_t_p_request"
        );
        assert_eq!(
            RenameRule::CamelCase.apply_to_field("http_request"),
            "httpRequest"
        );

        let input: DeriveInput = parse_quote! {
            #[serde(rename_all = "camelCase")]
            struct RawIdentifiers {
                r#type: String,
            }
        };
        let container = Container::from_ast(&input).unwrap();
        let Data::Struct(_, fields) = container.data else {
            panic!("expected struct");
        };
        assert_eq!(fields[0].attrs.name, "type");
    }
}
