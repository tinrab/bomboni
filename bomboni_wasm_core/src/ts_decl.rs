use crate::{
    options::{FieldWasm, WasmOptions},
    ts_type::{TsType, TsTypeElement, TypeLiteralTsType},
};
use bomboni_core::syn::type_is_phantom;
use itertools::Itertools;
use serde_derive_internals::{ast, attr::TagType};
use std::{
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
};

pub enum TsDecl {
    TypeAlias(TypeAliasTsDecl),
    Interface(InterfaceTsDecl),
    Enum(EnumTsDecl),
}

pub struct TypeAliasTsDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub alias_type: TsType,
}

pub struct InterfaceTsDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub extends: Vec<TsType>,
    pub body: Vec<TsTypeElement>,
}

pub struct EnumTsDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub members: Vec<TypeAliasTsDecl>,
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
        TypeAliasTsDecl {
            name: self.name.clone(),
            type_params: self.type_params.clone(),
            alias_type: TsType::Union(
                self.members
                    .iter()
                    .map(|member| member.alias_type.clone())
                    .collect(),
            ),
        }
        .fmt(f)
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
            self.parse_fields(style, fields),
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
        let members: Vec<_> = variants
            .iter()
            .filter(|variant| {
                !variant.attrs.skip_serializing() && !variant.attrs.skip_deserializing()
            })
            .map(|variant| {
                let tag_type = self.options.serde_attrs().tag();
                let name = variant.attrs.name().serialize_name().to_string();
                let variant_type: TsType = TsType::from(
                    self.parse_fields(variant.style, &variant.fields),
                )
                .with_tag_type(tag_type, &name, variant.style);
                let mut alias_type = self.make_type_alias(variant_type);
                alias_type.name = name;
                alias_type
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
        }
    }

    fn parse_fields(&self, style: ast::Style, fields: &[ast::Field]) -> ParsedFields {
        match style {
            ast::Style::Newtype => {
                return ParsedFields::Transparent(TsType::from_type(fields[0].ty))
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
            return ParsedFields::Transparent(self.parse_field(fields[0]).0);
        }

        match style {
            ast::Style::Struct => {
                let (flatten_fields, members): (Vec<_>, Vec<_>) =
                    fields.into_iter().partition(|field| field.attrs.flatten());

                let members = members
                    .into_iter()
                    .map(|field| {
                        let key = field.attrs.name().serialize_name().to_string();
                        let (field_type, field_options) = self.parse_field(field);

                        let optional = field_options.map_or(false, |options| options.optional);

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
                    .map(|field| self.parse_field(field).0)
                    .collect();

                ParsedFields::Named(members, flatten_fields)
            }
            ast::Style::Tuple => ParsedFields::Unnamed(
                fields
                    .into_iter()
                    .map(|field| self.parse_field(field).0)
                    .collect(),
            ),
            _ => unreachable!(),
        }
    }

    fn parse_field(&self, field: &ast::Field) -> (TsType, Option<&FieldWasm>) {
        let field_type = TsType::from_type(field.ty);
        let field_options = self.options.get_field(&field.member);
        (field_type, field_options)
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
