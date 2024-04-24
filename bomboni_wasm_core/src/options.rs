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
    pub wasm_bindgen_crate: Option<Path>,
    pub js_sys_crate: Option<Path>,
    pub bomboni_crate: Option<Path>,
    pub into_wasm_abi: bool,
    pub from_wasm_abi: bool,
    pub enum_value: bool,
    pub js_value: Option<JsValueWasm>,
    pub proxy: Option<ProxyWasm>,
    pub reference_change: ReferenceChangeMap,
    pub rename: Option<String>,
    pub rename_wrapper: Option<bool>,
    pub rename_all: Option<attr::RenameRule>,
    pub rename_boundary: Vec<Boundary>,
    pub override_type: Option<String>,
    pub fields: Vec<FieldWasm>,
    pub variants: Vec<VariantWasm>,
}

pub struct FieldWasm {
    pub member: Member,
    pub optional: bool,
    pub reference_change: ReferenceChangeMap,
    pub override_type: Option<String>,
    pub rename_wrapper: Option<bool>,
    pub always_some: Option<bool>,
    pub rename: Option<String>,
}

pub struct VariantWasm {
    pub ident: Ident,
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

#[derive(Debug)]
pub struct JsValueWasm {
    pub into: Option<Path>,
    pub try_from: Option<Path>,
    pub convert_string: bool,
}

#[derive(Debug)]
pub struct ProxyWasm {
    pub proxy: Path,
    pub into: Option<Path>,
    pub try_from: Option<Path>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(wasm))]
struct Attributes {
    wasm_bindgen_crate: Option<Path>,
    js_sys_crate: Option<Path>,
    bomboni_crate: Option<Path>,
    wasm_abi: Option<bool>,
    into_wasm_abi: Option<bool>,
    from_wasm_abi: Option<bool>,
    enum_value: Option<bool>,
    js_value: Option<JsValueWasm>,
    proxy: Option<ProxyWasm>,
    rename: Option<String>,
    change_ref: Option<ReferenceChangeMap>,
    change_refs: Option<ReferenceChangeMap>,
    rename_wrapper: Option<bool>,
    rename_all: Option<String>,
    rename_boundary: Option<String>,
    override_type: Option<String>,
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

        if attributes.enum_value.unwrap_or_default()
            && (attributes.js_value.is_some() || attributes.proxy.is_some())
        {
            return Err(syn::Error::new_spanned(
                input,
                "`enum_value` cannot be used with `js_value` or `proxy`",
            ));
        }
        if attributes.js_value.is_some()
            && (attributes.enum_value.unwrap_or_default() || attributes.proxy.is_some())
        {
            return Err(syn::Error::new_spanned(
                input,
                "`js_value` cannot be used with `enum_value` or `proxy`",
            ));
        }
        if attributes.proxy.is_some()
            && (attributes.enum_value.unwrap_or_default() || attributes.js_value.is_some())
        {
            return Err(syn::Error::new_spanned(
                input,
                "`proxy` cannot be used with `enum_value` or `js_value`",
            ));
        }

        Ok(Self {
            serde_container,
            wasm_bindgen_crate: attributes.wasm_bindgen_crate,
            js_sys_crate: attributes.js_sys_crate,
            bomboni_crate: attributes.bomboni_crate,
            into_wasm_abi: attributes.into_wasm_abi.unwrap_or(wasm_abi),
            from_wasm_abi: attributes.from_wasm_abi.unwrap_or(wasm_abi),
            enum_value: attributes.enum_value.unwrap_or_default(),
            js_value: attributes.js_value,
            proxy: attributes.proxy,
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
            override_type: attributes.override_type,
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

impl FromMeta for ProxyWasm {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        match expr {
            syn::Expr::Path(syn::ExprPath { path, .. }) => Ok(Self {
                proxy: path.clone(),
                into: None,
                try_from: None,
            }),
            _ => Err(darling::Error::custom("expected proxy path").with_span(expr)),
        }
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let mut proxy = None;
        let mut into = None;
        let mut try_from = None;
        for item in items {
            match item {
                darling::ast::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                    path,
                    value: syn::Expr::Path(value),
                    ..
                })) => {
                    if path.is_ident("source") {
                        if proxy.is_some() {
                            return Err(darling::Error::custom("proxy `source` already specified")
                                .with_span(item));
                        }
                        proxy = Some(value.path.clone());
                    } else if path.is_ident("into") {
                        if into.is_some() {
                            return Err(
                                darling::Error::custom("`into` already specified").with_span(item)
                            );
                        }
                        into = Some(value.path.clone());
                    } else if path.is_ident("try_from") {
                        if try_from.is_some() {
                            return Err(darling::Error::custom("`try_from` already specified")
                                .with_span(item));
                        }
                        try_from = Some(value.path.clone());
                    } else {
                        return Err(darling::Error::custom("invalid option").with_span(item));
                    }
                }
                _ => {
                    return Err(darling::Error::custom("invalid options").with_span(item));
                }
            }
        }
        Ok(Self {
            proxy: proxy.ok_or_else(|| darling::Error::custom("proxy `source` not specified"))?,
            into,
            try_from,
        })
    }
}

impl FromMeta for JsValueWasm {
    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let mut into = None;
        let mut try_from = None;
        let mut convert_string = false;
        for item in items {
            match item {
                darling::ast::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                    path,
                    value: syn::Expr::Path(value),
                    ..
                })) => {
                    if path.is_ident("into") {
                        if into.is_some() {
                            return Err(
                                darling::Error::custom("`into` already specified").with_span(item)
                            );
                        }
                        into = Some(value.path.clone());
                    } else if path.is_ident("try_from") {
                        if try_from.is_some() {
                            return Err(darling::Error::custom("`try_from` already specified")
                                .with_span(item));
                        }
                        try_from = Some(value.path.clone());
                    } else {
                        return Err(
                            darling::Error::custom("expected `into` or `try_from`").with_span(item)
                        );
                    }
                }
                darling::ast::NestedMeta::Meta(syn::Meta::Path(path)) => {
                    if path.is_ident("convert_string") {
                        convert_string = true;
                    } else {
                        return Err(darling::Error::custom("invalid option").with_span(item));
                    }
                }
                _ => {
                    return Err(darling::Error::custom("invalid options").with_span(item));
                }
            }
        }
        Ok(Self {
            into,
            try_from,
            convert_string,
        })
    }

    fn from_word() -> darling::Result<Self> {
        Ok(Self {
            into: None,
            try_from: None,
            convert_string: false,
        })
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
            reference_change,
            override_type: variant.override_type.clone(),
            rename_wrapper,
            fields: get_fields(&serde_variant.fields, &variant.fields),
            rename: variant.rename.clone(),
        });
    }

    variants
}
