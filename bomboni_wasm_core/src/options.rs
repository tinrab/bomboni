use std::collections::BTreeMap;

use darling::{ast::Fields, util, FromDeriveInput, FromField, FromMeta};
use itertools::Itertools;
use proc_macro2::Ident;
use serde_derive_internals::{
    ast::{self, Container as SerdeContainer},
    attr, Ctxt,
};
use syn::{self, DeriveInput, Generics, Member, Path, Visibility};

pub struct WasmOptions<'a> {
    pub serde_container: SerdeContainer<'a>,
    pub wasm_bindgen: Option<Path>,
    pub bomboni_wasm: Option<Path>,
    pub into_wasm_abi: bool,
    pub from_wasm_abi: bool,
    pub wasm_ref: bool,
    pub rename: Option<String>,
    pub interface_type: Option<bool>,
    pub fields: Vec<FieldWasm>,
}

pub struct DeclConstWasm {
    pub name: Option<Ident>,
    pub vis: Visibility,
}

pub struct FieldWasm {
    pub member: Member,
    pub optional: bool,
    pub type_rename: TypeRenameMap,
}

#[derive(Debug, Clone, Default)]
pub struct TypeRenameMap {
    pub new_name: Option<String>,
    pub name_map: BTreeMap<String, String>,
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
    rename: Option<String>,
    interface_type: Option<bool>,
    data: darling::ast::Data<util::Ignored, FieldAttributes>,
}

#[derive(Debug, FromField)]
#[darling(attributes(wasm))]
struct FieldAttributes {
    pub rename_type: Option<TypeRenameMap>,
    pub rename_types: Option<TypeRenameMap>,
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

        let mut fields = Vec::new();
        for (i, field) in serde_container.data.all_fields().enumerate() {
            let mut optional = false;
            if let Some(expr) = field.attrs.skip_serializing_if() {
                let path = expr
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .join("::");
                optional |= path == "Option::is_none";
            }

            let mut type_rename = TypeRenameMap::default();
            if let Some(Fields { fields, .. }) = attributes.data.as_ref().take_struct() {
                let Some(field) = fields.get(i) else {
                    continue;
                };
                type_rename = field
                    .rename_type
                    .as_ref()
                    .or(field.rename_types.as_ref())
                    .cloned()
                    .unwrap_or_default();
            }

            fields.push(FieldWasm {
                member: field.member.clone(),
                optional,
                type_rename,
            });
        }

        let wasm_abi = attributes.wasm_abi.unwrap_or_default();

        Ok(Self {
            serde_container,
            wasm_bindgen: attributes.wasm_bindgen,
            bomboni_wasm: attributes.bomboni_wasm,
            into_wasm_abi: attributes.into_wasm_abi.unwrap_or(wasm_abi),
            from_wasm_abi: attributes.from_wasm_abi.unwrap_or(wasm_abi),
            wasm_ref: attributes.wasm_ref.unwrap_or_default(),
            rename: attributes.rename,
            interface_type: attributes.interface_type,
            fields,
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

    pub fn get_field(&self, member: &Member) -> Option<&FieldWasm> {
        self.fields.iter().find(|field| &field.member == member)
    }
}

impl FromMeta for TypeRenameMap {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(name),
                ..
            }) => Ok(Self {
                new_name: Some(name.value()),
                name_map: BTreeMap::default(),
            }),
            syn::Expr::Array(syn::ExprArray { elems, .. }) => {
                let mut name_map = BTreeMap::new();
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
                            name_map.insert(source.value(), target.value());
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
                Ok(Self {
                    new_name: None,
                    name_map,
                })
            }
            _ => Err(darling::Error::custom("expected string literal")),
        }

        // if let syn::Expr::Lit(syn::ExprLit {
        //     lit: syn::Lit::Str(name),
        //     ..
        // }) = expr
        // {
        //     Ok(Self {
        //         new_name: Some(name.value()),
        //         name_map: Default::default(),
        //     })
        // } else {
        //     Err(darling::Error::custom("expected string literal"))
        // }
    }
}
