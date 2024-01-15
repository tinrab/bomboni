use std::collections::BTreeMap;

use convert_case::Boundary;
use darling::{ast::Fields, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::Ident;
use serde_derive_internals::{
    ast::{self, Container as SerdeContainer},
    attr, Ctxt,
};
use syn::{self, DeriveInput, Generics, Member, Path};

use crate::ts_type::TsType;

pub struct WasmOptions<'a> {
    pub serde_container: SerdeContainer<'a>,
    pub wasm_bindgen: Option<Path>,
    pub bomboni_wasm: Option<Path>,
    pub into_wasm_abi: bool,
    pub from_wasm_abi: bool,
    pub wasm_ref: bool,
    pub as_enum: bool,
    pub reference_change: ReferenceChangeMap,
    pub rename: Option<String>,
    pub rename_wrapper: Option<bool>,
    pub rename_all: Option<attr::RenameRule>,
    pub rename_boundary: Vec<Boundary>,
    pub fields: Vec<FieldWasm>,
    pub variants: Vec<VariantWasm>,
}

pub struct FieldWasm {
    pub member: Member,
    pub optional: bool,
    pub as_string: bool,
    pub reference_change: ReferenceChangeMap,
    pub override_type: Option<String>,
    pub rename_wrapper: Option<bool>,
    pub always_some: Option<bool>,
    pub rename: Option<String>,
}

pub struct VariantWasm {
    pub ident: Ident,
    pub as_string: bool,
    pub reference_change: ReferenceChangeMap,
    pub override_type: Option<String>,
    pub rename_wrapper: Option<bool>,
    pub fields: Vec<FieldWasm>,
    pub rename: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ReferenceChangeMap {
    pub name: Option<String>,
    pub types: BTreeMap<String, TsType>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(wasm))]
struct Attributes {
    wasm_bindgen: Option<Path>,
    bomboni_wasm: Option<Path>,
    wasm_abi: Option<bool>,
    into_wasm_abi: Option<bool>,
    from_wasm_abi: Option<bool>,
    wasm_ref: Option<bool>,
    as_enum: Option<bool>,
    rename: Option<String>,
    change_ref: Option<ReferenceChangeMap>,
    change_refs: Option<ReferenceChangeMap>,
    rename_wrapper: Option<bool>,
    rename_all: Option<String>,
    rename_boundary: Option<String>,
    data: darling::ast::Data<VariantAttributes, FieldAttributes>,
}

#[derive(Debug, FromField)]
#[darling(attributes(wasm))]
struct FieldAttributes {
    ident: Option<Ident>,
    change_ref: Option<ReferenceChangeMap>,
    change_refs: Option<ReferenceChangeMap>,
    override_type: Option<String>,
    rename_wrapper: Option<bool>,
    always_some: Option<bool>,
    rename: Option<String>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(wasm))]
struct VariantAttributes {
    ident: Ident,
    change_ref: Option<ReferenceChangeMap>,
    change_refs: Option<ReferenceChangeMap>,
    override_type: Option<String>,
    rename_wrapper: Option<bool>,
    fields: Fields<FieldAttributes>,
    rename: Option<String>,
}

impl<'a> WasmOptions<'a> {
    pub fn from_derive_input(input: &'a DeriveInput) -> syn::Result<Self> {
        let ctx = Ctxt::new();
        let serde_container = match SerdeContainer::from_ast(
            &ctx,
            input,
            serde_derive_internals::Derive::Serialize,
        ) {
            Some(container) => {
                ctx.check()?;
                container
            }
            None => {
                return Err(ctx.check().expect_err("serde_container is None"));
            }
        };
        let attributes = match Attributes::from_derive_input(input) {
            Ok(v) => v,
            Err(err) => {
                return Err(err.into());
            }
        };

        let (fields, variants) = match (&serde_container.data, attributes.data) {
            (ast::Data::Struct(_, serde_fields), darling::ast::Data::Struct(field_attributes)) => {
                let fields = get_fields(serde_fields, &field_attributes);
                (fields, Vec::new())
            }
            (ast::Data::Enum(serde_variants), darling::ast::Data::Enum(variant_attributes)) => {
                let variants = get_variants(serde_variants, &variant_attributes);
                (Vec::new(), variants)
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "invalid struct or enum for WASM",
                ));
            }
        };

        let wasm_abi = attributes.wasm_abi.unwrap_or_default();

        let rename_all = if let Some(rename_all) = attributes.rename_all {
            Some(
                attr::RenameRule::from_str(&rename_all)
                    .map_err(|err| syn::Error::new_spanned(input, err))?,
            )
        } else {
            None
        };
        let rename_boundary = if let Some(rename_boundary) = attributes.rename_boundary.as_ref() {
            Boundary::list_from(rename_boundary)
        } else {
            Vec::new()
        };

        Ok(Self {
            serde_container,
            wasm_bindgen: attributes.wasm_bindgen,
            bomboni_wasm: attributes.bomboni_wasm,
            into_wasm_abi: attributes.into_wasm_abi.unwrap_or(wasm_abi),
            from_wasm_abi: attributes.from_wasm_abi.unwrap_or(wasm_abi),
            wasm_ref: attributes.wasm_ref.unwrap_or_default(),
            as_enum: attributes.as_enum.unwrap_or_default(),
            rename: attributes.rename,
            reference_change: attributes
                .change_ref
                .as_ref()
                .or(attributes.change_refs.as_ref())
                .cloned()
                .unwrap_or_default(),
            rename_wrapper: attributes.rename_wrapper,
            rename_all,
            rename_boundary,
            fields,
            variants,
        })
    }

    pub fn ident(&self) -> &Ident {
        &self.serde_container.ident
    }

    pub fn name(&self) -> &str {
        self.rename.as_ref().map_or_else(
            || self.serde_attrs().name().serialize_name(),
            String::as_str,
        )
    }

    pub fn serde_data(&self) -> &ast::Data {
        &self.serde_container.data
    }

    pub fn generics(&self) -> &Generics {
        self.serde_container.generics
    }

    pub fn serde_attrs(&self) -> &attr::Container {
        &self.serde_container.attrs
    }
}

impl FromMeta for ReferenceChangeMap {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(name),
                ..
            }) => Ok(Self {
                name: Some(name.value()),
                types: BTreeMap::default(),
            }),
            syn::Expr::Array(syn::ExprArray { elems, .. }) => {
                let mut types = BTreeMap::new();
                for elem in elems {
                    if let syn::Expr::Tuple(syn::ExprTuple { elems, .. }) = elem {
                        if elems.len() != 2 {
                            return Err(darling::Error::custom(
                                "expected tuple of length 2 containing source and target names",
                            )
                            .with_span(elem));
                        }
                        if let (
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(source),
                                ..
                            }),
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(target),
                                ..
                            }),
                        ) = (&elems[0], &elems[1])
                        {
                            types.insert(
                                source.value(),
                                TsType::Reference {
                                    name: target.value(),
                                    type_params: Vec::new(),
                                },
                            );
                        } else {
                            return Err(darling::Error::custom(
                                "expected tuple of length 2 containing source and target names",
                            ));
                        }
                    } else {
                        return Err(darling::Error::custom(
                            "expected tuple of length 2 containing source and target names",
                        )
                        .with_span(elem));
                    }
                }
                Ok(Self { name: None, types })
            }
            _ => Err(darling::Error::custom("expected string literal")),
        }
    }
}

fn get_fields(
    serde_fields: &[ast::Field],
    field_attributes: &Fields<FieldAttributes>,
) -> Vec<FieldWasm> {
    let mut fields = Vec::new();

    for serde_field in serde_fields {
        let mut optional = false;
        if let Some(expr) = serde_field.attrs.skip_serializing_if() {
            let last_step = expr.path.segments.iter().rev().nth(1);
            optional |= matches!(
                last_step,
                Some(
                    syn::PathSegment { ident, .. }
                ) if ident == "is_none"
            );
            optional |= matches!(
                last_step,
                Some(
                    syn::PathSegment { ident, .. }
                ) if ident == "is_default"
            );
        }

        let mut as_string = false;
        if let Some(with) = serde_field.attrs.serialize_with() {
            as_string |= matches!(
                with.path.segments.iter().rev().nth(1),
                Some(
                    syn::PathSegment { ident, .. }
                ) if ident == "as_string"
            );
        }

        let Some((_, field)) =
            field_attributes
                .iter()
                .enumerate()
                .find(|(i, field)| match &serde_field.member {
                    Member::Named(serde_ident) => Some(serde_ident) == field.ident.as_ref(),
                    Member::Unnamed(serde_index) => serde_index.index as usize == *i,
                })
        else {
            continue;
        };
        let reference_change = field
            .change_ref
            .as_ref()
            .or(field.change_refs.as_ref())
            .cloned()
            .unwrap_or_default();
        let rename_wrapper = field.rename_wrapper;

        fields.push(FieldWasm {
            member: serde_field.member.clone(),
            optional,
            as_string,
            reference_change,
            override_type: field.override_type.clone(),
            rename_wrapper,
            always_some: field.always_some,
            rename: field.rename.clone(),
        });
    }

    fields
}

fn get_variants(
    serde_variants: &[ast::Variant],
    variant_attributes: &[VariantAttributes],
) -> Vec<VariantWasm> {
    let mut variants = Vec::new();

    for serde_variant in serde_variants {
        let mut as_string = false;
        if let Some(with) = serde_variant.attrs.serialize_with() {
            as_string |= matches!(
                with.path.segments.iter().rev().nth(1),
                Some(
                    syn::PathSegment { ident, .. }
                ) if ident == "as_string"
            );
        }

        let Some(variant) = variant_attributes
            .iter()
            .find(|variant| variant.ident == serde_variant.ident)
        else {
            continue;
        };
        let reference_change = variant
            .change_ref
            .as_ref()
            .or(variant.change_refs.as_ref())
            .cloned()
            .unwrap_or_default();
        let rename_wrapper = variant.rename_wrapper;

        variants.push(VariantWasm {
            ident: serde_variant.ident.clone(),
            as_string,
            reference_change,
            override_type: variant.override_type.clone(),
            rename_wrapper,
            fields: get_fields(&serde_variant.fields, &variant.fields),
            rename: variant.rename.clone(),
        });
    }

    variants
}
