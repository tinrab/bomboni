use crate::{
    options::{FieldWasm, WasmOptions},
    ts_type::{TsType, TsTypeElement, TypeLiteralTsType},
};
use bomboni_core::{string::str_to_case, syn::type_is_phantom};
use convert_case::{Case, Casing};
use itertools::Itertools;
use serde_derive_internals::{
    ast,
    attr::{RenameRule, TagType},
};
use std::{
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub enum TsDecl {
    TypeAlias(TypeAliasTsDecl),
    Interface(InterfaceTsDecl),
    Enum(EnumTsDecl),
}

#[derive(Debug)]
pub struct TypeAliasTsDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub alias_type: TsType,
}

#[derive(Debug)]
pub struct InterfaceTsDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub extends: Vec<TsType>,
    pub body: Vec<TsTypeElement>,
}

#[derive(Debug)]
pub struct EnumTsDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub members: Vec<TypeAliasTsDecl>,
    pub external_tag: bool,
    pub as_enum: bool,
}

pub struct TsDeclParser<'a> {
    options: &'a WasmOptions<'a>,
}

impl From<TypeAliasTsDecl> for TsDecl {
    fn from(decl: TypeAliasTsDecl) -> Self {
        Self::TypeAlias(decl)
    }
}

impl Display for TypeAliasTsDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "export type {}{} = {};",
            self.name,
            if self.type_params.is_empty() {
                String::new()
            } else {
                format!("<{}>", self.type_params.join(", "))
            },
            self.alias_type
        )
    }
}

impl From<InterfaceTsDecl> for TsDecl {
    fn from(decl: InterfaceTsDecl) -> Self {
        Self::Interface(decl)
    }
}

impl Display for InterfaceTsDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "export interface {}", self.name)?;

        if !self.type_params.is_empty() {
            write!(f, "<{}>", self.type_params.join(", "))?;
        }

        if !self.extends.is_empty() {
            write!(
                f,
                " extends {}",
                self.extends.iter().map(ToString::to_string).join(", ")
            )?;
        }

        if self.body.is_empty() {
            write!(f, " {{}}")
        } else {
            write!(
                f,
                " {{{}\n}}",
                self.body
                    .iter()
                    .map(|element| format!("\n  {element};"))
                    .join("")
            )
        }
    }
}

impl From<EnumTsDecl> for TsDecl {
    fn from(decl: EnumTsDecl) -> Self {
        Self::Enum(decl)
    }
}

impl Display for EnumTsDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.as_enum {
            assert!(
                self.type_params.is_empty(),
                "enum with type params not supported"
            );
            write!(f, "export enum {} {{", self.name)?;
            for member in &self.members {
                write!(
                    f,
                    "\n  {} = {},",
                    str_to_case(&member.name, Case::Pascal),
                    &member.alias_type
                )?;
            }
            write!(f, "\n}}")
        } else {
            write!(
                f,
                "export type {}{} = {};",
                self.name,
                if self.type_params.is_empty() {
                    String::new()
                } else {
                    format!("<{}>", self.type_params.join(", "))
                },
                TsType::Union(
                    self.members
                        .iter()
                        .enumerate()
                        .map(|(i, member)| {
                            let mut member_type = member.alias_type.clone();
                            if self.external_tag {
                                // Add empty fields to externally tagged enum.
                                // This makes it possible to switch based on kind in TypeScript.
                                if let TsType::TypeLiteral(member_type) = &mut member_type {
                                    member_type.members.extend(
                                        self.members.iter().enumerate().filter_map(
                                            |(j, other_member)| {
                                                if j == i {
                                                    None
                                                } else {
                                                    Some(TsTypeElement {
                                                        key: other_member.name.clone(),
                                                        alias_type: TsType::nullish(),
                                                        optional: true,
                                                    })
                                                }
                                            },
                                        ),
                                    );
                                }
                            }
                            member_type
                        })
                        .collect(),
                )
            )
        }
    }
}

impl TsDecl {
    pub fn name(&self) -> &str {
        match self {
            Self::TypeAlias(decl) => &decl.name,
            Self::Interface(decl) => &decl.name,
            Self::Enum(decl) => &decl.name,
        }
    }
}

impl Display for TsDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeAlias(decl) => decl.fmt(f),
            Self::Interface(decl) => decl.fmt(f),
            Self::Enum(decl) => decl.fmt(f),
        }
    }
}

#[derive(Debug)]
enum ParsedFields {
    Named(Vec<TsTypeElement>, Vec<TsType>),
    Unnamed(Vec<TsType>),
    Transparent(TsType),
}

impl<'a> TsDeclParser<'a> {
    pub fn new(options: &'a WasmOptions<'a>) -> Self {
        Self { options }
    }

    pub fn parse(&self) -> TsDecl {
        match &self.options.serde_data() {
            ast::Data::Struct(style, fields) => self.parse_struct(*style, fields),
            ast::Data::Enum(variants) => self.parse_enum(variants).into(),
        }
    }

    fn parse_struct(&self, style: ast::Style, fields: &[ast::Field]) -> TsDecl {
        match (
            self.options.serde_attrs().tag(),
            self.parse_fields(style, fields, &self.options.fields),
        ) {
            (TagType::Internal { tag, .. }, ParsedFields::Named(members, extends)) => {
                let name = self.options.name();
                let tag_field = TsTypeElement {
                    key: tag.clone(),
                    alias_type: TsType::Literal(name.into()),
                    optional: false,
                };

                let mut vec = Vec::with_capacity(members.len() + 1);
                vec.push(tag_field);
                vec.extend(members);

                self.make_named_decl(vec, extends)
            }
            (_, ParsedFields::Named(members, extends)) => self.make_named_decl(members, extends),
            (_, fields) => self.make_type_alias(fields.into()).into(),
        }
    }

    fn parse_enum(&self, variants: &[ast::Variant]) -> EnumTsDecl {
        let tag_type = self.options.serde_attrs().tag();
        let members: Vec<TypeAliasTsDecl> = variants
            .iter()
            .filter_map(|variant| {
                if variant.attrs.skip_serializing() || variant.attrs.skip_deserializing() {
                    None
                } else {
                    Some(self.parse_variant(variant, tag_type))
                }
            })
            .collect();

        EnumTsDecl {
            name: self.options.name().into(),
            type_params: self.make_ref_type_params(
                members
                    .iter()
                    .flat_map(|member| member.alias_type.get_reference_names())
                    .collect(),
            ),
            members,
            external_tag: matches!(tag_type, TagType::External),
            as_enum: self.options.as_enum,
        }
    }

    fn parse_fields(
        &self,
        style: ast::Style,
        fields: &[ast::Field],
        wasm_fields: &'a [FieldWasm],
    ) -> ParsedFields {
        match style {
            ast::Style::Newtype => {
                return ParsedFields::Transparent(self.parse_field(&fields[0], wasm_fields).1);
            }
            ast::Style::Unit => return ParsedFields::Transparent(TsType::nullish()),
            _ => {}
        }

        let fields: Vec<_> = fields
            .iter()
            .filter(|field| {
                !field.attrs.skip_serializing()
                    && !field.attrs.skip_deserializing()
                    && !type_is_phantom(field.ty)
            })
            .collect();

        if fields.len() == 1 && self.options.serde_attrs().transparent() {
            return ParsedFields::Transparent(self.parse_field(fields[0], wasm_fields).1);
        }

        match style {
            ast::Style::Struct => {
                let (flatten_fields, members): (Vec<_>, Vec<_>) =
                    fields.into_iter().partition(|field| field.attrs.flatten());

                let members = members
                    .into_iter()
                    .map(|field| {
                        let (key, field_type, wasm_field) = self.parse_field(field, wasm_fields);

                        let optional = wasm_field.optional;
                        let alias_type = if optional {
                            if let TsType::Option(t) = field_type {
                                *t
                            } else {
                                field_type
                            }
                        } else {
                            field_type
                        };

                        TsTypeElement {
                            key,
                            alias_type,
                            optional: optional
                                || !(self.options.serde_attrs().default().is_none()
                                    && field.attrs.default().is_none()),
                        }
                    })
                    .collect();

                let flatten_fields = flatten_fields
                    .into_iter()
                    .map(|field| self.parse_field(field, wasm_fields).1)
                    .collect();

                ParsedFields::Named(members, flatten_fields)
            }
            ast::Style::Tuple => ParsedFields::Unnamed(
                fields
                    .into_iter()
                    .map(|field| self.parse_field(field, wasm_fields).1)
                    .collect(),
            ),
            _ => unreachable!(),
        }
    }

    fn parse_field(
        &self,
        field: &ast::Field,
        wasm_fields: &'a [FieldWasm],
    ) -> (String, TsType, &'a FieldWasm) {
        let wasm_field = wasm_fields
            .iter()
            .find(|f| f.member == field.member)
            .unwrap();

        let name = wasm_field.rename.clone().unwrap_or_else(|| {
            let mut name = field.attrs.name().serialize_name().to_string();
            if let Some(rename_all) = self.options.rename_all {
                name = self.apply_rename(&name, rename_all);
            }
            name
        });

        let mut field_type = TsType::from_type(field.ty);
        if wasm_field.as_string {
            field_type = TsType::STRING;
        }
        if wasm_field.always_some.unwrap_or_default() {
            if let TsType::Option(some_type) = field_type {
                field_type = *some_type;
            }
        }
        if wasm_field
            .rename_wrapper
            .or(self.options.rename_wrapper)
            .unwrap_or_default()
        {
            field_type = field_type.rename_protobuf_wrapper();
        }
        field_type = field_type.rename_reference(
            if wasm_field.reference_rename.name.is_some()
                || !wasm_field.reference_rename.types.is_empty()
            {
                &wasm_field.reference_rename
            } else {
                &self.options.reference_rename
            },
        );

        (name, field_type, wasm_field)
    }

    fn parse_variant(&self, variant: &ast::Variant, tag_type: &TagType) -> TypeAliasTsDecl {
        let wasm_variant = self
            .options
            .variants
            .iter()
            .find(|v| v.ident == variant.ident)
            .unwrap();

        let name = wasm_variant.rename.clone().unwrap_or_else(|| {
            let mut name = variant.attrs.name().serialize_name().to_string();
            if let Some(rename_all) = self.options.rename_all {
                name = self.apply_rename(&name, rename_all);
            }
            name
        });

        let variant_type: TsType = {
            if wasm_variant.as_string {
                TsType::STRING
            } else {
                TsType::from(self.parse_fields(
                    variant.style,
                    &variant.fields,
                    &wasm_variant.fields,
                ))
            }
        }
        .with_tag_type(tag_type, &name, variant.style);

        let mut alias_type = self.make_type_alias(variant_type);
        alias_type.name = name;

        alias_type
    }

    fn apply_rename(&self, s: &str, rename_rule: RenameRule) -> String {
        if self.options.rename_boundary.is_empty() {
            match rename_rule {
                RenameRule::None => s.to_owned(),
                RenameRule::LowerCase => str_to_case(s, Case::Lower),
                RenameRule::UpperCase => str_to_case(s, Case::Upper),
                RenameRule::PascalCase => str_to_case(s, Case::Pascal),
                RenameRule::CamelCase => str_to_case(s, Case::Camel),
                RenameRule::SnakeCase => str_to_case(s, Case::Snake),
                RenameRule::ScreamingSnakeCase => str_to_case(s, Case::ScreamingSnake),
                RenameRule::KebabCase => str_to_case(s, Case::Kebab),
                RenameRule::ScreamingKebabCase => str_to_case(s, Case::Cobol),
            }
        } else {
            match rename_rule {
                RenameRule::None => s.to_owned(),
                RenameRule::LowerCase => s
                    .with_boundaries(&self.options.rename_boundary)
                    .to_case(Case::Lower),
                RenameRule::UpperCase => s
                    .with_boundaries(&self.options.rename_boundary)
                    .to_case(Case::Upper),
                RenameRule::PascalCase => s
                    .with_boundaries(&self.options.rename_boundary)
                    .to_case(Case::Pascal),
                RenameRule::CamelCase => s
                    .with_boundaries(&self.options.rename_boundary)
                    .to_case(Case::Camel),
                RenameRule::SnakeCase => s
                    .with_boundaries(&self.options.rename_boundary)
                    .to_case(Case::Snake),
                RenameRule::ScreamingSnakeCase => s
                    .with_boundaries(&self.options.rename_boundary)
                    .to_case(Case::ScreamingSnake),
                RenameRule::KebabCase => s
                    .with_boundaries(&self.options.rename_boundary)
                    .to_case(Case::Kebab),
                RenameRule::ScreamingKebabCase => s
                    .with_boundaries(&self.options.rename_boundary)
                    .to_case(Case::Cobol),
            }
        }
    }

    fn make_named_decl(&self, members: Vec<TsTypeElement>, extends: Vec<TsType>) -> TsDecl {
        if extends.iter().all(TsType::is_reference) {
            InterfaceTsDecl {
                name: self.options.name().into(),
                type_params: self.make_ref_type_params(
                    members
                        .iter()
                        .map(|member| member.alias_type.get_reference_names())
                        .chain(extends.iter().map(TsType::get_reference_names))
                        .flatten()
                        .collect(),
                ),
                extends,
                body: members,
            }
            .into()
        } else {
            self.make_type_alias(
                TsType::from(TypeLiteralTsType { members }).intersection(TsType::Intersection(
                    extends
                        .into_iter()
                        .map(|ty| match ty {
                            TsType::Option(ty) => TsType::Union(vec![*ty, TsType::nullish()]),
                            _ => ty,
                        })
                        .collect(),
                )),
            )
            .into()
        }
    }

    fn make_type_alias(&self, alias_type: TsType) -> TypeAliasTsDecl {
        TypeAliasTsDecl {
            name: self.options.name().into(),
            type_params: self.make_ref_type_params(alias_type.get_reference_names()),
            alias_type,
        }
    }

    fn make_ref_type_params(&self, type_ref_names: BTreeSet<String>) -> Vec<String> {
        self.options
            .generics()
            .type_params()
            .map(|p| p.ident.to_string())
            .filter(|t| type_ref_names.contains(t))
            .collect()
    }
}

impl From<ParsedFields> for TsType {
    fn from(fields: ParsedFields) -> Self {
        match fields {
            ParsedFields::Named(members, extends) => {
                let ty = TsType::from(TypeLiteralTsType { members });
                if extends.is_empty() {
                    ty
                } else {
                    ty.intersection(TsType::Intersection(extends))
                }
            }
            ParsedFields::Unnamed(elements) => TsType::Tuple(elements),
            ParsedFields::Transparent(ty) => ty,
        }
    }
}
