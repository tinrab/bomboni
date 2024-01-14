use crate::options::ReferenceRenameMap;
use serde_derive_internals::ast::Style;
use serde_derive_internals::attr::TagType;
use std::collections::BTreeSet;
use std::fmt::Write;
use std::fmt::{self, Display, Formatter};
use syn::{
    Expr, ExprLit, GenericArgument, Lit, Path, PathArguments, PathSegment, ReturnType, Type,
    TypeArray, TypeBareFn, TypeGroup, TypeImplTrait, TypeParamBound, TypeParen, TypePath,
    TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TsType {
    Keyword(KeywordTsType),
    Literal(String),
    Array(Box<Self>),
    Tuple(Vec<Self>),
    Option(Box<Self>),
    Reference {
        name: String,
        type_params: Vec<Self>,
    },
    Fn {
        params: Vec<Self>,
        alias_type: Box<Self>,
    },
    TypeLiteral(TypeLiteralTsType),
    Intersection(Vec<Self>),
    Union(Vec<Self>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordTsType {
    Number,
    Bigint,
    Boolean,
    String,
    Void,
    Undefined,
    Null,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeLiteralTsType {
    pub members: Vec<TsTypeElement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsTypeElement {
    pub key: String,
    pub alias_type: TsType,
    pub optional: bool,
}

impl TsType {
    pub const NUMBER: TsType = TsType::Keyword(KeywordTsType::Number);
    pub const BIGINT: TsType = TsType::Keyword(KeywordTsType::Bigint);
    pub const BOOLEAN: TsType = TsType::Keyword(KeywordTsType::Boolean);
    pub const STRING: TsType = TsType::Keyword(KeywordTsType::String);
    pub const VOID: TsType = TsType::Keyword(KeywordTsType::Void);
    pub const UNDEFINED: TsType = TsType::Keyword(KeywordTsType::Undefined);
    pub const NULL: TsType = TsType::Keyword(KeywordTsType::Null);
    pub const NEVER: TsType = TsType::Keyword(KeywordTsType::Never);

    pub fn from_type(value: &Type) -> Self {
        match value {
            Type::Array(TypeArray { elem, len, .. }) => {
                let elem = Self::from_type(elem);
                let len = if let Expr::Lit(ExprLit {
                    lit: Lit::Int(lit_int),
                    ..
                }) = len
                {
                    lit_int.base10_parse::<usize>().ok()
                } else {
                    None
                };

                match len {
                    Some(len) if len <= 16 => Self::Tuple(vec![elem; len]),
                    _ => Self::Array(Box::new(elem)),
                }
            }
            Type::Slice(TypeSlice { elem, .. }) => Self::Array(Box::new(Self::from_type(elem))),
            Type::Reference(TypeReference { elem, .. })
            | Type::Paren(TypeParen { elem, .. })
            | Type::Group(TypeGroup { elem, .. }) => Self::from_type(elem),
            Type::BareFn(TypeBareFn { inputs, output, .. }) => {
                let params = inputs.iter().map(|arg| Self::from_type(&arg.ty)).collect();

                let alias_type = if let syn::ReturnType::Type(_, ty) = output {
                    Self::from_type(ty)
                } else {
                    Self::VOID
                };

                Self::Fn {
                    params,
                    alias_type: Box::new(alias_type),
                }
            }
            Type::Tuple(TypeTuple { elems, .. }) => {
                if elems.is_empty() {
                    Self::nullish()
                } else {
                    let elements = elems.iter().map(Self::from_type).collect();
                    Self::Tuple(elements)
                }
            }
            Type::Path(TypePath { path, .. }) => Self::from_path(path).unwrap_or(Self::NEVER),
            Type::TraitObject(TypeTraitObject { bounds, .. })
            | Type::ImplTrait(TypeImplTrait { bounds, .. }) => {
                let elements = bounds
                    .iter()
                    .filter_map(|t| match t {
                        TypeParamBound::Trait(t) => Self::from_path(&t.path),
                        _ => None,
                    })
                    .collect();

                Self::Intersection(elements)
            }
            Type::Ptr(_) | Type::Infer(_) | Type::Macro(_) | Type::Never(_) | Type::Verbatim(_) => {
                Self::NEVER
            }
            _ => Self::NEVER,
        }
    }

    pub const fn nullish() -> Self {
        if cfg!(feature = "js") {
            Self::UNDEFINED
        } else {
            Self::NULL
        }
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, Self::Reference { .. })
    }

    pub fn get_reference_names(&self) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        self.visit(&mut |ty| {
            if let Self::Reference { name, .. } = ty {
                names.insert(name.clone());
            }
        });
        names
    }

    fn visit<'a, F>(&'a self, f: &mut F)
    where
        F: FnMut(&'a Self),
    {
        f(self);
        match self {
            Self::Keyword(_) | Self::Literal(_) => (),
            Self::Array(element) => element.visit(f),
            Self::Tuple(elements) => {
                elements.iter().for_each(|t| t.visit(f));
            }
            Self::Option(t) => t.visit(f),
            Self::Reference { type_params, .. } => {
                type_params.iter().for_each(|t| t.visit(f));
            }
            Self::Fn { params, alias_type } => {
                params
                    .iter()
                    .chain(Some(alias_type.as_ref()))
                    .for_each(|t| t.visit(f));
            }
            Self::TypeLiteral(TypeLiteralTsType { members }) => {
                members.iter().for_each(|m| m.alias_type.visit(f));
            }
            Self::Intersection(types) | Self::Union(types) => {
                types.iter().for_each(|t| t.visit(f));
            }
        }
    }

    fn from_path(path: &Path) -> Option<Self> {
        path.segments.last().map(Self::from_path_segment)
    }

    fn from_path_segment(segment: &PathSegment) -> Self {
        let name = segment.ident.to_string();

        let (args, _) = match &segment.arguments {
            PathArguments::AngleBracketed(path) => {
                let args = path
                    .args
                    .iter()
                    .filter_map(|p| match p {
                        GenericArgument::Type(t) => Some(t),
                        GenericArgument::AssocType(t) => Some(&t.ty),
                        _ => None,
                    })
                    .collect();
                (args, None)
            }
            PathArguments::Parenthesized(path) => {
                let args = path.inputs.iter().collect();
                let output = match &path.output {
                    ReturnType::Default => None,
                    ReturnType::Type(_, tp) => Some(tp.as_ref()),
                };
                (args, output)
            }
            PathArguments::None => (vec![], None),
        };

        match name.as_str() {
            "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize"
            | "f64" | "f32" => Self::NUMBER,
            "u128" | "i128" => {
                if cfg!(feature = "js") {
                    Self::BIGINT
                } else {
                    Self::NUMBER
                }
            }
            "String" | "str" | "char" | "Path" | "PathBuf" => Self::STRING,
            "bool" => Self::BOOLEAN,
            "Box" | "Cow" | "Rc" | "Arc" | "Cell" | "RefCell" if args.len() == 1 => {
                Self::from_type(args[0])
            }
            "Vec" | "VecDeque" | "LinkedList" if args.len() == 1 => {
                let element = Self::from_type(args[0]);
                Self::Array(Box::new(element))
            }
            "HashMap" | "BTreeMap" if args.len() == 2 => {
                let type_params = args.iter().map(|arg| Self::from_type(arg)).collect();

                let name = if cfg!(feature = "js") {
                    "Map"
                } else {
                    "Record"
                }
                .to_string();

                Self::Reference { name, type_params }
            }
            "HashSet" | "BTreeSet" if args.len() == 1 => {
                let element = Self::from_type(args[0]);
                Self::Array(Box::new(element))
            }
            "ByteBuf" => {
                if cfg!(feature = "js") {
                    Self::Reference {
                        name: String::from("Uint8Array"),
                        type_params: vec![],
                    }
                } else {
                    Self::Array(Box::new(Self::NUMBER))
                }
            }
            "Option" if args.len() == 1 => Self::Option(Box::new(Self::from_type(args[0]))),
            _ => {
                let type_params = args.into_iter().map(Self::from_type).collect();
                Self::Reference { name, type_params }
            }
        }
    }

    #[must_use]
    pub fn with_tag_type(self, tag_type: &TagType, name: &str, style: Style) -> Self {
        match tag_type {
            TagType::External => {
                if matches!(style, Style::Unit) {
                    Self::Literal(name.into())
                } else {
                    TsTypeElement {
                        key: name.into(),
                        alias_type: self,
                        optional: false,
                    }
                    .into()
                }
            }
            TagType::Internal { tag } => {
                if self == TsType::nullish() {
                    TsTypeElement {
                        key: tag.clone(),
                        alias_type: Self::Literal(name.into()),
                        optional: false,
                    }
                    .into()
                } else {
                    TsType::from(TsTypeElement {
                        key: tag.clone(),
                        alias_type: Self::Literal(name.into()),
                        optional: false,
                    })
                    .intersection(self)
                }
            }
            TagType::Adjacent { tag, content } => {
                let tag_field = TsTypeElement {
                    key: tag.clone(),
                    alias_type: TsType::Literal(name.into()),
                    optional: false,
                };
                if matches!(style, Style::Unit) {
                    tag_field.into()
                } else {
                    TypeLiteralTsType {
                        members: vec![
                            tag_field,
                            TsTypeElement {
                                key: content.clone(),
                                alias_type: self,
                                optional: false,
                            },
                        ],
                    }
                    .into()
                }
            }
            TagType::None => self,
        }
    }

    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        match (self, other) {
            (Self::TypeLiteral(x), Self::TypeLiteral(y)) => x.intersection(y).into(),
            (Self::Intersection(x), Self::Intersection(y)) => {
                let mut vec = Vec::with_capacity(x.len() + y.len());
                vec.extend(x);
                vec.extend(y);
                TsType::Intersection(vec)
            }
            (Self::Intersection(x), y) => {
                let mut vec = Vec::with_capacity(x.len() + 1);
                vec.extend(x);
                vec.push(y);
                Self::Intersection(vec)
            }
            (x, Self::Intersection(y)) => {
                let mut vec = Vec::with_capacity(y.len() + 1);
                vec.push(x);
                vec.extend(y);
                Self::Intersection(vec)
            }
            (x, y) => Self::Intersection(vec![x, y]),
        }
    }

    #[must_use]
    pub fn rename_reference(self, rename_map: &ReferenceRenameMap) -> Self {
        if rename_map.name.is_none() && rename_map.types.is_empty() {
            return self;
        }
        match self {
            Self::Reference { name, type_params } => {
                let type_params = type_params
                    .into_iter()
                    .map(|param| param.rename_reference(rename_map))
                    .collect();
                if let Some(new_name) = rename_map.name.clone() {
                    Self::Reference {
                        name: new_name,
                        type_params,
                    }
                } else if let Some(new_type) = rename_map.types.get(&name).cloned() {
                    new_type
                } else {
                    Self::Reference { name, type_params }
                }
            }

            Self::Array(element) => Self::Array(Box::new(element.rename_reference(rename_map))),
            Self::Tuple(elements) => Self::Tuple(
                elements
                    .into_iter()
                    .map(|element| element.rename_reference(rename_map))
                    .collect(),
            ),
            Self::Option(element) => Self::Option(Box::new(element.rename_reference(rename_map))),
            Self::Fn { params, alias_type } => Self::Fn {
                params,
                alias_type: Box::new(alias_type.rename_reference(rename_map)),
            },
            Self::TypeLiteral(TypeLiteralTsType { members }) => {
                let members = members
                    .into_iter()
                    .map(|member| TsTypeElement {
                        key: member.key,
                        alias_type: member.alias_type.rename_reference(rename_map),
                        optional: member.optional,
                    })
                    .collect();
                Self::TypeLiteral(TypeLiteralTsType { members })
            }
            Self::Intersection(types) => Self::Intersection(
                types
                    .into_iter()
                    .map(|ty| ty.rename_reference(rename_map))
                    .collect(),
            ),
            Self::Union(types) => Self::Union(
                types
                    .into_iter()
                    .map(|ty| ty.rename_reference(rename_map))
                    .collect(),
            ),
            _ => self,
        }
    }

    #[must_use]
    pub fn rename_protobuf_wrapper(self) -> Self {
        let rename_map = ReferenceRenameMap {
            name: None,
            types: [
                ("DoubleValue", "number"),
                ("FloatValue", "number"),
                ("Int64Value", "number"),
                ("UInt64Value", "number"),
                ("Int32Value", "number"),
                ("UInt32Value", "number"),
                ("BoolValue", "boolean"),
                ("StringValue", "string"),
            ]
            .into_iter()
            .map(|(s, t)| {
                (
                    s.into(),
                    TsType::Reference {
                        name: t.into(),
                        type_params: Vec::new(),
                    },
                )
            })
            .chain([(
                "BytesValue".into(),
                if cfg!(feature = "js") {
                    Self::Reference {
                        name: String::from("Uint8Array"),
                        type_params: vec![],
                    }
                } else {
                    Self::Array(Box::new(Self::NUMBER))
                },
            )])
            .collect(),
        };
        self.rename_reference(&rename_map)
    }
}

impl From<&Type> for TsType {
    fn from(value: &Type) -> Self {
        Self::from_type(value)
    }
}

impl Display for TsType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyword(kind) => {
                let ty = format!("{kind:?}").to_lowercase();
                write!(f, "{ty}")
            }
            Self::Literal(literal) => {
                write!(f, "\"{literal}\"")
            }
            Self::Array(element) => match element.as_ref() {
                Self::Union(_) | Self::Intersection(_) | &Self::Option(_) => {
                    write!(f, "({element})[]")
                }
                _ => write!(f, "{element}[]"),
            },
            Self::Tuple(elements) => {
                write!(
                    f,
                    "[{}]",
                    elements
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Self::Reference { name, type_params } => {
                let params = type_params
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                if params.is_empty() {
                    write!(f, "{name}")
                } else {
                    write!(f, "{name}<{params}>")
                }
            }
            Self::Fn { params, alias_type } => {
                let params = params
                    .iter()
                    .enumerate()
                    .map(|(i, param)| format!("arg{i}: {param}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "({params}) => {alias_type}")
            }
            Self::Option(element) => {
                write!(f, "{element} | {}", Self::nullish())
            }
            Self::TypeLiteral(type_literal) => {
                write!(f, "{type_literal}")
            }
            Self::Intersection(types) => {
                if types.len() == 1 {
                    let ty = &types[0];
                    return write!(f, "{ty}");
                }

                let types = types
                    .iter()
                    .map(|ty| match ty {
                        Self::Union(_) => format!("({ty})"),
                        _ => ty.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(" & ");

                write!(f, "{types}")
            }
            Self::Union(types) => {
                if types.len() == 1 {
                    let ty = &types[0];
                    return write!(f, "{ty}");
                }

                let types = types
                    .iter()
                    .map(|ty| match ty {
                        Self::Intersection(_) => format!("({ty})"),
                        _ => ty.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");

                write!(f, "{types}")
            }
        }
    }
}

impl TypeLiteralTsType {
    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        self.members.into_iter().chain(other.members).fold(
            TypeLiteralTsType {
                members: Vec::new(),
            },
            |mut acc, member| {
                if let Some(existing) = acc.get_member_mut(&member.key) {
                    let mut prev = TsType::NULL;
                    std::mem::swap(&mut existing.alias_type, &mut prev);
                    existing.alias_type = prev.intersection(member.alias_type);
                } else {
                    acc.members.push(member);
                }
                acc
            },
        )
    }

    fn get_member_mut<K: AsRef<str>>(&mut self, key: K) -> Option<&mut TsTypeElement> {
        self.members
            .iter_mut()
            .find(|member| member.key == key.as_ref())
    }
}

impl From<TypeLiteralTsType> for TsType {
    fn from(value: TypeLiteralTsType) -> Self {
        Self::TypeLiteral(value)
    }
}

impl Display for TypeLiteralTsType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let members = self.members.iter().fold(String::new(), |mut s, member| {
            let _ = write!(s, "\n  {member};");
            s
        });
        if members.is_empty() {
            write!(f, "{{}}")
        } else {
            write!(f, "{{{members}\n}}")
        }
    }
}

impl From<TsTypeElement> for TsType {
    fn from(value: TsTypeElement) -> Self {
        Self::TypeLiteral(TypeLiteralTsType {
            members: vec![value],
        })
    }
}

impl Display for TsTypeElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let key = &self.key;
        let alias_type = &self.alias_type;
        let opt = if self.optional { "?" } else { "" };
        if key.contains('-') {
            write!(f, "\"{key}\"{opt}: {alias_type}")
        } else {
            write!(f, "{key}{opt}: {alias_type}")
        }
    }
}

impl From<KeywordTsType> for TsType {
    fn from(value: KeywordTsType) -> Self {
        Self::Keyword(value)
    }
}
