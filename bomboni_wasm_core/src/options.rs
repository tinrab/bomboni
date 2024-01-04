use darling::FromDeriveInput;
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
    pub decl_type: bool,
    pub into_wasm_abi: bool,
    pub from_wasm_abi: bool,
    pub wasm_ref: bool,
    pub fields: Vec<FieldWasm>,
}

pub struct DeclConstWasm {
    pub name: Option<Ident>,
    pub vis: Visibility,
}

pub struct FieldWasm {
    pub member: Member,
    pub optional: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(wasm))]
struct Attributes {
    wasm_bindgen: Option<Path>,
    #[darling(default)]
    decl_type: Option<bool>,
    #[darling(default)]
    wasm_abi: Option<bool>,
    #[darling(default)]
    into_wasm_abi: Option<bool>,
    #[darling(default)]
    from_wasm_abi: Option<bool>,
    #[darling(default)]
    wasm_ref: Option<bool>,
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
        for field in serde_container.data.all_fields() {
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
            fields.push(FieldWasm {
                member: field.member.clone(),
                optional,
            });
        }

        let wasm_abi = attributes.wasm_abi.unwrap_or_default();

        Ok(Self {
            serde_container,
            wasm_bindgen: attributes.wasm_bindgen,
            decl_type: attributes.decl_type.unwrap_or_default(),
            into_wasm_abi: attributes.into_wasm_abi.unwrap_or(wasm_abi),
            from_wasm_abi: attributes.from_wasm_abi.unwrap_or(wasm_abi),
            wasm_ref: attributes.wasm_ref.unwrap_or_default(),
            fields,
        })
    }

    pub fn ident(&self) -> &Ident {
        &self.serde_container.ident
    }

    pub fn name(&self) -> &str {
        self.serde_attrs().name().serialize_name()
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
