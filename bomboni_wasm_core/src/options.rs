#![allow(clippy::needless_continue)]

use std::collections::BTreeMap;

use bomboni_core::syn::meta::{
    attribute_metas, meta_bool, meta_expr, meta_list, meta_path, meta_string,
};
use convert_case::Boundary;
use proc_macro2::Ident;
use syn::{
    self, Attribute, Data, DeriveInput, Expr, ExprLit, ExprTuple, Generics, Lit, Member, Meta, Path,
};

use crate::{
    serde::{self, Container as SerdeContainer, RenameRule},
    ts_type::TsType,
};

/// Configuration options for the Wasm derive macro.
pub struct WasmOptions<'a> {
    serde_container: SerdeContainer<'a>,

    /// Custom path to the wasm-bindgen crate.
    pub wasm_bindgen_crate: Option<Path>,

    /// Custom path to the js-sys crate.
    pub js_sys_crate: Option<Path>,

    /// Custom path to the bomboni crate.
    pub bomboni_crate: Option<Path>,

    /// Custom path to the `bomboni_wasm` crate.
    pub bomboni_wasm_crate: Option<Path>,

    /// Generate `IntoWasmAbi` implementation.
    pub into_wasm_abi: bool,

    /// Generate `FromWasmAbi` implementation.
    pub from_wasm_abi: bool,

    /// Generate enum value object.
    pub enum_value: bool,

    /// Custom `JsValue` conversion configuration.
    pub js_value: Option<JsValueWasm>,

    /// Proxy type configuration.
    pub proxy: Option<ProxyWasm>,

    /// Reference type mapping configuration.
    pub reference_change: ReferenceChangeMap,

    /// Custom name for the type in TypeScript.
    pub rename: Option<String>,

    /// Control wrapper type renaming.
    pub rename_wrapper: Option<bool>,

    /// Rename rule for all fields and variants.
    pub rename_all: Option<RenameRule>,

    /// Word boundaries for renaming.
    pub rename_boundary: Vec<Boundary>,

    /// Override the generated TypeScript type.
    pub override_type: Option<String>,

    /// Field-specific WASM options.
    pub fields: Vec<FieldWasm>,

    /// Variant-specific WASM options.
    pub variants: Vec<VariantWasm>,
}

/// WASM options for a struct field.
pub struct FieldWasm {
    /// The field member (name or index).
    pub member: Member,

    /// Whether the field is optional.
    pub optional: bool,

    /// Reference type mapping for this field.
    pub reference_change: ReferenceChangeMap,

    /// Override the TypeScript type for this field.
    pub override_type: Option<String>,

    /// Control wrapper type renaming for this field.
    pub rename_wrapper: Option<bool>,

    /// Force the field to always be present in TypeScript.
    pub always_some: Option<bool>,

    /// Custom name for the field in TypeScript.
    pub rename: Option<String>,
}

/// WASM options for an enum variant.
pub struct VariantWasm {
    /// The identifier of the variant.
    pub ident: Ident,

    /// Reference type mapping for this variant.
    pub reference_change: ReferenceChangeMap,

    /// Override the TypeScript type for this variant.
    pub override_type: Option<String>,

    /// Control wrapper type renaming for this variant.
    pub rename_wrapper: Option<bool>,

    /// Field-specific WASM options for this variant's fields.
    pub fields: Vec<FieldWasm>,

    /// Custom name for the variant in TypeScript.
    pub rename: Option<String>,
}

/// Maps Rust reference types to TypeScript types.
#[derive(Debug, Clone, Default)]
pub struct ReferenceChangeMap {
    /// Simple name mapping for the reference type.
    pub name: Option<String>,

    /// Complex type mappings for multiple reference types.
    pub types: BTreeMap<String, TsType>,
}

/// Configuration for custom `JsValue` conversions.
#[derive(Debug)]
pub struct JsValueWasm {
    /// Custom conversion function from Rust type to `JsValue`.
    pub into: Option<Path>,

    /// Custom conversion function from `JsValue` to Rust type.
    pub try_from: Option<Path>,

    /// Convert the type to/from JavaScript strings.
    pub convert_string: bool,
}

/// Configuration for proxy type WASM bindings.
#[derive(Debug)]
pub struct ProxyWasm {
    /// The proxy type to use for WASM bindings.
    pub proxy: Path,

    /// Custom conversion function from original type to proxy type.
    pub into: Option<Path>,

    /// Custom conversion function from proxy type to original type.
    pub try_from: Option<Path>,
}

#[derive(Debug, Default)]
struct Attributes {
    wasm_bindgen_crate: Option<Path>,
    js_sys_crate: Option<Path>,
    bomboni_crate: Option<Path>,
    bomboni_wasm_crate: Option<Path>,
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
}

#[derive(Debug, Default)]
struct FieldAttributes {
    ident: Option<Ident>,
    change_ref: Option<ReferenceChangeMap>,
    change_refs: Option<ReferenceChangeMap>,
    override_type: Option<String>,
    rename_wrapper: Option<bool>,
    always_some: Option<bool>,
    rename: Option<String>,
}

#[derive(Debug)]
struct VariantAttributes {
    ident: Ident,
    change_ref: Option<ReferenceChangeMap>,
    change_refs: Option<ReferenceChangeMap>,
    override_type: Option<String>,
    rename_wrapper: Option<bool>,
    fields: Vec<FieldAttributes>,
    rename: Option<String>,
}

impl Attributes {
    fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut options = Self::default();
        for meta in attribute_metas(attrs, "wasm")? {
            let path = meta.path();
            if path.is_ident("wasm_bindgen_crate") {
                options.wasm_bindgen_crate = Some(meta_path(&meta)?);
            } else if path.is_ident("js_sys_crate") {
                options.js_sys_crate = Some(meta_path(&meta)?);
            } else if path.is_ident("bomboni_crate") {
                options.bomboni_crate = Some(meta_path(&meta)?);
            } else if path.is_ident("bomboni_wasm_crate") {
                options.bomboni_wasm_crate = Some(meta_path(&meta)?);
            } else if path.is_ident("wasm_abi") {
                options.wasm_abi = Some(meta_bool(&meta)?);
            } else if path.is_ident("into_wasm_abi") {
                options.into_wasm_abi = Some(meta_bool(&meta)?);
            } else if path.is_ident("from_wasm_abi") {
                options.from_wasm_abi = Some(meta_bool(&meta)?);
            } else if path.is_ident("enum_value") {
                options.enum_value = Some(meta_bool(&meta)?);
            } else if path.is_ident("js_value") {
                options.js_value = Some(parse_js_value(&meta)?);
            } else if path.is_ident("proxy") {
                options.proxy = Some(parse_proxy(&meta)?);
            } else if path.is_ident("rename") {
                options.rename = Some(meta_string(&meta)?);
            } else if path.is_ident("change_ref") {
                options.change_ref = Some(parse_reference_change(&meta)?);
            } else if path.is_ident("change_refs") {
                options.change_refs = Some(parse_reference_change(&meta)?);
            } else if path.is_ident("rename_wrapper") {
                options.rename_wrapper = Some(meta_bool(&meta)?);
            } else if path.is_ident("rename_all") {
                options.rename_all = Some(meta_string(&meta)?);
            } else if path.is_ident("rename_boundary") {
                options.rename_boundary = Some(meta_string(&meta)?);
            } else if path.is_ident("override_type") {
                options.override_type = Some(meta_string(&meta)?);
            } else {
                return Err(syn::Error::new_spanned(meta, "unknown WASM option"));
            }
        }
        Ok(options)
    }
}

impl FieldAttributes {
    fn parse(field: &syn::Field) -> syn::Result<Self> {
        let mut options = Self {
            ident: field.ident.clone(),
            ..Self::default()
        };
        for meta in attribute_metas(&field.attrs, "wasm")? {
            let path = meta.path();
            if path.is_ident("change_ref") {
                options.change_ref = Some(parse_reference_change(&meta)?);
            } else if path.is_ident("change_refs") {
                options.change_refs = Some(parse_reference_change(&meta)?);
            } else if path.is_ident("override_type") {
                options.override_type = Some(meta_string(&meta)?);
            } else if path.is_ident("rename_wrapper") {
                options.rename_wrapper = Some(meta_bool(&meta)?);
            } else if path.is_ident("always_some") {
                options.always_some = Some(meta_bool(&meta)?);
            } else if path.is_ident("rename") {
                options.rename = Some(meta_string(&meta)?);
            } else {
                return Err(syn::Error::new_spanned(meta, "unknown WASM field option"));
            }
        }
        Ok(options)
    }
}

impl VariantAttributes {
    fn parse(variant: &syn::Variant) -> syn::Result<Self> {
        let mut options = Self {
            ident: variant.ident.clone(),
            change_ref: None,
            change_refs: None,
            override_type: None,
            rename_wrapper: None,
            fields: variant
                .fields
                .iter()
                .map(FieldAttributes::parse)
                .collect::<syn::Result<_>>()?,
            rename: None,
        };
        for meta in attribute_metas(&variant.attrs, "wasm")? {
            let path = meta.path();
            if path.is_ident("change_ref") {
                options.change_ref = Some(parse_reference_change(&meta)?);
            } else if path.is_ident("change_refs") {
                options.change_refs = Some(parse_reference_change(&meta)?);
            } else if path.is_ident("override_type") {
                options.override_type = Some(meta_string(&meta)?);
            } else if path.is_ident("rename_wrapper") {
                options.rename_wrapper = Some(meta_bool(&meta)?);
            } else if path.is_ident("rename") {
                options.rename = Some(meta_string(&meta)?);
            } else {
                return Err(syn::Error::new_spanned(meta, "unknown WASM variant option"));
            }
        }
        Ok(options)
    }
}

impl<'a> WasmOptions<'a> {
    /// Creates `WasmOptions` from a `DeriveInput`.
    ///
    /// # Errors
    ///
    /// Will return an error if the input is not a valid struct or enum for WASM,
    /// if serde attributes are invalid, or if incompatible attribute combinations are used.
    pub fn from_derive_input(input: &'a DeriveInput) -> syn::Result<Self> {
        let serde_container = SerdeContainer::from_ast(input)?;
        let attributes = Attributes::parse(&input.attrs)?;

        let (fields, variants) = match (&serde_container.data, &input.data) {
            (serde::Data::Struct(_, serde_fields), Data::Struct(data)) => {
                let field_attributes = data
                    .fields
                    .iter()
                    .map(FieldAttributes::parse)
                    .collect::<syn::Result<Vec<_>>>()?;
                let fields = get_fields(serde_fields, &field_attributes);
                (fields, Vec::new())
            }
            (serde::Data::Enum(serde_variants), Data::Enum(data)) => {
                let variant_attributes = data
                    .variants
                    .iter()
                    .map(VariantAttributes::parse)
                    .collect::<syn::Result<Vec<_>>>()?;
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
            Some(RenameRule::parse(&rename_all, input)?)
        } else {
            None
        };
        let rename_boundary = attributes
            .rename_boundary
            .as_ref()
            .map_or_else(Vec::new, |rename_boundary| {
                Boundary::defaults_from(rename_boundary)
            });

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
            bomboni_wasm_crate: attributes.bomboni_wasm_crate,
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

    /// Gets the identifier of the type.
    pub const fn ident(&self) -> &Ident {
        self.serde_container.ident
    }

    /// Gets the name of the type.
    pub fn name(&self) -> &str {
        self.rename
            .as_ref()
            .map_or_else(|| self.serde_attrs().name.as_str(), String::as_str)
    }

    /// Gets the serde data for the type.
    pub(crate) const fn serde_data(&self) -> &serde::Data<'_> {
        &self.serde_container.data
    }

    /// Gets the generic parameters for the type.
    pub const fn generics(&self) -> &Generics {
        self.serde_container.generics
    }

    /// Gets the serde attributes for the type.
    pub(crate) const fn serde_attrs(&self) -> &serde::ContainerAttrs {
        &self.serde_container.attrs
    }

    /// Gets the custom Serde crate path, if configured with `#[serde(crate = "...")]`.
    pub const fn serde_crate(&self) -> Option<&Path> {
        self.serde_container.attrs.custom_crate.as_ref()
    }
}

fn parse_reference_change(meta: &Meta) -> syn::Result<ReferenceChangeMap> {
    match meta_expr(meta)? {
        Expr::Lit(ExprLit {
            lit: Lit::Str(name),
            ..
        }) => Ok(ReferenceChangeMap {
            name: Some(name.value()),
            types: BTreeMap::new(),
        }),
        Expr::Array(array) => {
            let mut types = BTreeMap::new();
            for elem in &array.elems {
                let Expr::Tuple(ExprTuple { elems, .. }) = elem else {
                    return Err(syn::Error::new_spanned(
                        elem,
                        "expected a source-target tuple",
                    ));
                };
                let [source, target] = elems.iter().collect::<Vec<_>>()[..] else {
                    return Err(syn::Error::new_spanned(
                        elem,
                        "expected a source-target tuple",
                    ));
                };
                let (
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(source),
                        ..
                    }),
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(target),
                        ..
                    }),
                ) = (source, target)
                else {
                    return Err(syn::Error::new_spanned(elem, "expected string literals"));
                };
                types.insert(
                    source.value(),
                    TsType::Reference {
                        name: target.value(),
                        type_params: Vec::new(),
                    },
                );
            }
            Ok(ReferenceChangeMap { name: None, types })
        }
        expr => Err(syn::Error::new_spanned(
            expr,
            "expected a string or mapping array",
        )),
    }
}

fn parse_proxy(meta: &Meta) -> syn::Result<ProxyWasm> {
    if let Meta::NameValue(_) = meta {
        return Ok(ProxyWasm {
            proxy: meta_path(meta)?,
            into: None,
            try_from: None,
        });
    }
    let Meta::List(list) = meta else {
        return Err(syn::Error::new_spanned(meta, "expected proxy options"));
    };
    let mut proxy = None;
    let mut into = None;
    let mut try_from = None;
    for option in meta_list(list)? {
        if option.path().is_ident("source") {
            proxy = Some(meta_path(&option)?);
        } else if option.path().is_ident("into") {
            into = Some(meta_path(&option)?);
        } else if option.path().is_ident("try_from") {
            try_from = Some(meta_path(&option)?);
        } else {
            return Err(syn::Error::new_spanned(option, "invalid proxy option"));
        }
    }
    Ok(ProxyWasm {
        proxy: proxy
            .ok_or_else(|| syn::Error::new_spanned(meta, "proxy `source` not specified"))?,
        into,
        try_from,
    })
}

fn parse_js_value(meta: &Meta) -> syn::Result<JsValueWasm> {
    let mut value = JsValueWasm {
        into: None,
        try_from: None,
        convert_string: false,
    };
    let Meta::List(list) = meta else {
        return if matches!(meta, Meta::Path(_)) {
            Ok(value)
        } else {
            Err(syn::Error::new_spanned(meta, "expected js_value options"))
        };
    };
    for option in meta_list(list)? {
        if option.path().is_ident("into") {
            value.into = Some(meta_path(&option)?);
        } else if option.path().is_ident("try_from") {
            value.try_from = Some(meta_path(&option)?);
        } else if option.path().is_ident("convert_string") {
            value.convert_string = meta_bool(&option)?;
        } else {
            return Err(syn::Error::new_spanned(option, "invalid js_value option"));
        }
    }
    Ok(value)
}

fn get_fields(
    serde_fields: &[serde::Field<'_>],
    field_attributes: &[FieldAttributes],
) -> Vec<FieldWasm> {
    let mut fields = Vec::new();

    for serde_field in serde_fields {
        let mut optional = false;
        if let Some(path) = &serde_field.attrs.skip_serializing_if {
            let predicate = path.segments.last().map(|segment| &segment.ident);
            optional |= predicate.is_some_and(|ident| ident == "is_none" || ident == "is_default");
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
    serde_variants: &[serde::Variant<'_>],
    variant_attributes: &[VariantAttributes],
) -> Vec<VariantWasm> {
    let mut variants = Vec::new();

    for serde_variant in serde_variants {
        let Some(variant) = variant_attributes
            .iter()
            .find(|variant| variant.ident == *serde_variant.ident)
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
