use std::collections::BTreeMap;

use convert_case::Boundary;
use darling::{FromDeriveInput, FromField, FromMeta, FromVariant, ast::Fields};
use proc_macro2::Ident;
use serde_derive_internals::{
    Ctxt,
    ast::{self, Container as SerdeContainer},
    attr,
};
use syn::{self, DeriveInput, Generics, Member, Path};

use crate::ts_type::TsType;

/// Configuration options for the Wasm derive macro.
///
/// This struct controls how TypeScript WASM bindings are generated for Rust types.
/// The Wasm macro generates code that allows Rust types to be used in JavaScript/TypeScript
/// environments through wasm-bindgen, with automatic TypeScript type generation.
///
/// # Examples
///
/// ```rust,ignore
/// #[derive(Wasm)]
/// #[wasm(rename = "User", into_wasm_abi = true, from_wasm_abi = true)]
/// struct User {
///     #[wasm(rename = "userName")]
///     name: String,
///     
///     #[wasm(override_type = "Date")]
///     created_at: DateTime<Utc>,
/// }
/// ```
///
/// # Features
///
/// - Automatic TypeScript type generation
/// - wasm-bindgen integration
/// - Custom type conversions
/// - Enum value objects
/// - Proxy types for complex conversions
/// - Reference type mapping
/// - Field and variant renaming
pub struct WasmOptions<'a> {
    /// The serde container information from the input type.
    ///
    /// This contains information about the type's serde attributes,
    /// which are used to guide the WASM binding generation.
    pub serde_container: SerdeContainer<'a>,

    /// Custom path to the wasm-bindgen crate.
    ///
    /// If not specified, defaults to `wasm_bindgen`. This is useful
    /// in monorepos or when using a custom version of wasm-bindgen.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(wasm_bindgen_crate = "crate::wasm")]
    /// struct MyStruct { /* fields */ }
    /// ```
    pub wasm_bindgen_crate: Option<Path>,

    /// Custom path to the js-sys crate.
    ///
    /// If not specified, defaults to `js_sys`. This is useful when
    /// you need to use a custom version or path to js-sys.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(js_sys_crate = "crate::js")]
    /// struct MyStruct { /* fields */ }
    /// ```
    pub js_sys_crate: Option<Path>,

    /// Custom path to the bomboni crate.
    ///
    /// If not specified, defaults to `bomboni` or `bomboni::wasm` depending
    /// on the feature configuration. This is useful in monorepos.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(bomboni_crate = "crate::internal::bomboni")]
    /// struct MyStruct { /* fields */ }
    /// ```
    pub bomboni_crate: Option<Path>,

    /// Custom path to the `bomboni_wasm` crate.
    ///
    /// If not specified, defaults to `bomboni_wasm`. This is useful when
    /// you need to use a custom version or path.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(bomboni_wasm_crate = "crate::wasm")]
    /// struct MyStruct { /* fields */ }
    /// ```
    pub bomboni_wasm_crate: Option<Path>,

    /// Generate `IntoWasmAbi` implementation.
    ///
    /// When set to `true`, generates code to convert the Rust type into
    /// a WASM ABI representation. This is required for passing Rust values
    /// to JavaScript functions.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(into_wasm_abi = true)]
    /// struct MyStruct { /* fields */ }
    /// ```
    pub into_wasm_abi: bool,

    /// Generate `FromWasmAbi` implementation.
    ///
    /// When set to `true`, generates code to convert from a WASM ABI
    /// representation back to the Rust type. This is required for receiving
    /// values from JavaScript.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(from_wasm_abi = true)]
    /// struct MyStruct { /* fields */ }
    /// ```
    pub from_wasm_abi: bool,

    /// Generate enum value object.
    ///
    /// When set to `true`, generates a JavaScript enum object with both
    /// string and numeric values for each enum variant. This creates a
    /// more ergonomic API for working with enums in JavaScript.
    ///
    /// This option cannot be used with `js_value` or `proxy`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(enum_value = true)]
    /// enum Status {
    ///     Active,
    ///     Inactive,
    /// }
    ///
    /// // Generates:
    /// // export const Status = Object.freeze({
    /// //   ACTIVE: "Active",
    /// //   INACTIVE: "Inactive",
    /// //   Active: "ACTIVE",
    /// //   Inactive: "INACTIVE",
    /// // });
    /// ```
    pub enum_value: bool,

    /// Custom `JsValue` conversion configuration.
    ///
    /// When specified, uses custom conversion functions for converting
    /// between the Rust type and JavaScript `JsValue`. This is useful for
    /// types that don't have standard serde serialization.
    ///
    /// This option cannot be used with `enum_value` or `proxy`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(js_value(...))]
    /// struct MyType { /* fields */ }
    /// ```
    pub js_value: Option<JsValueWasm>,

    /// Proxy type configuration.
    ///
    /// When specified, uses a proxy type for WASM bindings. The proxy type
    /// handles the WASM conversion, and the original type is converted
    /// to/from the proxy. This is useful for complex types that need
    /// custom conversion logic.
    ///
    /// This option cannot be used with `enum_value` or `js_value`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(proxy = ProxyType)]
    /// struct MyComplexType { /* fields */ }
    /// ```
    pub proxy: Option<ProxyWasm>,

    /// Reference type mapping configuration.
    ///
    /// Maps Rust reference types to TypeScript types. This is used to
    /// handle references, pointers, and other non-owning types in the
    /// generated TypeScript definitions.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(change_ref = "&str -> string")]
    /// struct MyStruct { /* fields */ }
    /// ```
    pub reference_change: ReferenceChangeMap,

    /// Custom name for the type in TypeScript.
    ///
    /// If not specified, uses the Rust type name (possibly transformed
    /// by `rename_all`). This allows you to use different names in Rust
    /// and TypeScript.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(rename = "User")]
    /// struct UserStruct { /* fields */ }
    /// ```
    pub rename: Option<String>,

    /// Control wrapper type renaming.
    ///
    /// When set to `Some(true)`, wrapper types (like `Option`, `Vec`) will
    /// be renamed according to the `rename_all` rule. When `Some(false)`,
    /// wrapper types will not be renamed. `None` uses the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(rename_wrapper = true, rename_all = "camelCase")]
    /// struct MyStruct { /* fields */ }
    /// ```
    pub rename_wrapper: Option<bool>,

    /// Rename rule for all fields and variants.
    ///
    /// Applies a case transformation to all field and variant names.
    /// Common values include "camelCase", "`PascalCase`", "`snake_case`", etc.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(rename_all = "camelCase")]
    /// struct MyStruct {
    ///     user_name: String,  // Becomes "userName" in TypeScript
    ///     created_at: DateTime,  // Becomes "createdAt"
    /// }
    /// ```
    pub rename_all: Option<attr::RenameRule>,

    /// Word boundaries for renaming.
    ///
    /// Specifies how to identify word boundaries when applying rename rules.
    /// This affects how compound words are split and transformed.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(rename_boundary = "lowercase")]
    /// struct MyStruct { /* fields */ }
    /// ```
    pub rename_boundary: Vec<Boundary>,

    /// Override the generated TypeScript type.
    ///
    /// When specified, completely replaces the generated TypeScript type
    /// with the provided string. This is useful for types that have special
    /// representations in TypeScript or when you want to use existing types.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(override_type = "Date")]
    /// struct Timestamp { /* fields */ }
    ///
    /// #[wasm(override_type = "{ name: string; age: number }")]
    /// struct Person { /* fields */ }
    /// ```
    pub override_type: Option<String>,

    /// Field-specific WASM options.
    ///
    /// Contains the parsed WASM options for each field in the struct.
    /// This is populated during macro processing.
    pub fields: Vec<FieldWasm>,

    /// Variant-specific WASM options.
    ///
    /// Contains the parsed WASM options for each variant in the enum.
    /// This is populated during macro processing.
    pub variants: Vec<VariantWasm>,
}

/// WASM options for a struct field.
///
/// Contains configuration for how individual fields should be handled
/// in the generated WASM bindings and TypeScript definitions.
///
/// # Examples
///
/// ```rust,ignore
/// #[derive(Wasm)]
/// struct User {
///     #[wasm(rename = "userName", override_type = "string")]
///     name: String,
///     
///     #[wasm(always_some = true)]
///     optional_field: Option<String>,
/// }
/// ```
pub struct FieldWasm {
    /// The field member (name or index).
    ///
    /// For named struct fields, this contains the field name.
    /// For tuple struct fields, this contains the field index.
    pub member: Member,

    /// Whether the field is optional.
    ///
    /// This is automatically detected based on whether the field type
    /// is `Option<T>` or if the field has `skip_serializing_if` attribute.
    pub optional: bool,

    /// Reference type mapping for this field.
    ///
    /// Overrides the global reference change mapping for this specific field.
    /// This is useful when individual fields need different type mappings.
    pub reference_change: ReferenceChangeMap,

    /// Override the TypeScript type for this field.
    ///
    /// When specified, replaces the automatically generated TypeScript type
    /// with the provided string. This is useful for fields that have special
    /// representations in TypeScript.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(override_type = "Date")]
    /// created_at: DateTime<Utc>,
    ///
    /// #[wasm(override_type = "Blob")]
    /// data: Vec<u8>,
    /// ```
    pub override_type: Option<String>,

    /// Control wrapper type renaming for this field.
    ///
    /// When set to `Some(true)`, wrapper types (like `Option`, `Vec`) will
    /// be renamed according to the global `rename_all` rule. When `Some(false)`,
    /// wrapper types will not be renamed. `None` uses the global default.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(rename_wrapper = false)]
    /// user_list: Vec<User>,  // Stays as Vec<User> in TypeScript
    /// ```
    pub rename_wrapper: Option<bool>,

    /// Force the field to always be present in TypeScript.
    ///
    /// When set to `Some(true)`, the field will be treated as required
    /// in the TypeScript definition even if it's `Option<T>` in Rust.
    /// This is useful when the field is optional in Rust but always present
    /// in the JavaScript representation.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(always_some = true)]
    /// metadata: Option<HashMap<String, String>>,  // Required in TypeScript
    /// ```
    pub always_some: Option<bool>,

    /// Custom name for the field in TypeScript.
    ///
    /// If not specified, uses the Rust field name (possibly transformed
    /// by the global `rename_all` rule). This allows you to use different
    /// names in Rust and TypeScript for individual fields.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(rename = "userName")]
    /// name: String,
    ///
    /// #[wasm(rename = "createdAt")]
    /// created_at: DateTime<Utc>,
    /// ```
    pub rename: Option<String>,
}

/// WASM options for an enum variant.
///
/// Contains configuration for how individual enum variants should be handled
/// in the generated WASM bindings and TypeScript definitions.
///
/// # Examples
///
/// ```rust,ignore
/// #[derive(Wasm)]
/// enum Status {
///     #[wasm(rename = "active")]
///     Active,
///     
///     #[wasm(rename = "inactive")]
///     Inactive { reason: String },
///     
///     #[wasm(override_type = "{ code: number; message: string }")]
///     Error { code: i32, message: String },
/// }
/// ```
pub struct VariantWasm {
    /// The identifier of the variant.
    pub ident: Ident,

    /// Reference type mapping for this variant.
    ///
    /// Overrides the global reference change mapping for this specific variant.
    /// This is useful when individual variants need different type mappings.
    pub reference_change: ReferenceChangeMap,

    /// Override the TypeScript type for this variant.
    ///
    /// When specified, replaces the automatically generated TypeScript type
    /// with the provided string. This is useful for variants that have special
    /// representations in TypeScript or when you want to use existing types.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(override_type = "string")]
    /// Custom(String),
    ///
    /// #[wasm(override_type = "{ x: number; y: number }")]
    /// Point { x: f64, y: f64 },
    /// ```
    pub override_type: Option<String>,

    /// Control wrapper type renaming for this variant.
    ///
    /// When set to `Some(true)`, wrapper types in the variant's fields
    /// will be renamed according to the global `rename_all` rule.
    /// When `Some(false)`, wrapper types will not be renamed.
    /// `None` uses the global default.
    pub rename_wrapper: Option<bool>,

    /// Field-specific WASM options for this variant's fields.
    ///
    /// Contains the parsed WASM options for each field in struct variants.
    /// For tuple variants, this contains options for each tuple element.
    /// For unit variants, this is empty.
    pub fields: Vec<FieldWasm>,

    /// Custom name for the variant in TypeScript.
    ///
    /// If not specified, uses the Rust variant name (possibly transformed
    /// by the global `rename_all` rule). This allows you to use different
    /// names in Rust and TypeScript for individual variants.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(rename = "UserActive")]
    /// Active,
    ///
    /// #[wasm(rename = "UserInactive")]
    /// Inactive,
    /// ```
    pub rename: Option<String>,
}

/// Maps Rust reference types to TypeScript types.
///
/// This structure defines how Rust reference types (like `&str`, `&[T]`, etc.)
/// should be represented in the generated TypeScript definitions.
///
/// This is particularly useful for handling non-owning types that need
/// special representation in JavaScript/TypeScript where references work
/// differently than in Rust.
///
/// # Examples
///
/// ```rust,ignore
/// // Simple name mapping
/// ReferenceChangeMap {
///     name: Some("string".to_string()),
///     types: BTreeMap::new(),
/// }
///
/// // Complex type mapping
/// ReferenceChangeMap {
///     name: None,
///     types: {
///         let mut map = BTreeMap::new();
///         map.insert("&str".to_string(), TsType::Reference {
///             name: "string".to_string(),
///             type_params: Vec::new(),
///         });
///         map.insert("&[u8]".to_string(), TsType::Reference {
///             name: "Uint8Array".to_string(),
///             type_params: Vec::new(),
///         });
///         map
///     },
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ReferenceChangeMap {
    /// Simple name mapping for the reference type.
    ///
    /// When specified, provides a simple string replacement for the reference type.
    /// This is useful for straightforward mappings like `&str` → `string`.
    pub name: Option<String>,

    /// Complex type mappings for multiple reference types.
    ///
    /// Maps Rust type strings to TypeScript type definitions.
    /// This allows for more complex mappings with type parameters and nested types.
    ///
    /// The key is the Rust type string (like `"&str"`, `"&[T]"`), and the value
    /// is the corresponding TypeScript type definition.
    pub types: BTreeMap<String, TsType>,
}

/// Configuration for custom `JsValue` conversions.
///
/// This structure allows you to specify custom conversion functions for
/// converting between Rust types and JavaScript `JsValue`. This is useful
/// for types that don't have standard serde serialization or need special
/// handling in the JavaScript environment.
///
/// # Examples
///
/// ```rust,ignore
/// #[wasm(js_value(
///     into = MyType::to_js_value,
///     try_from = MyType::from_js_value,
/// ))]
/// struct MyType { /* fields */ }
///
/// #[wasm(js_value(convert_string))]
/// struct MyString(String);
/// ```
#[derive(Debug)]
pub struct JsValueWasm {
    /// Custom conversion function from Rust type to `JsValue`.
    ///
    /// Specifies a function that converts the Rust type into a `JsValue`.
    /// The function should take `self` and return a `JsValue`.
    ///
    /// If not specified, defaults to `Into::into`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// impl MyType {
    ///     fn to_js_value(self) -> JsValue {
    ///         // Custom conversion logic
    ///     }
    /// }
    ///
    /// #[wasm(js_value(into = MyType::to_js_value))]
    /// struct MyType { /* fields */ }
    /// ```
    pub into: Option<Path>,

    /// Custom conversion function from `JsValue` to Rust type.
    ///
    /// Specifies a function that attempts to convert a `JsValue` into the Rust type.
    /// The function should take a `&JsValue` and return a `Result<Self, JsValue>`.
    ///
    /// If not specified, defaults to `TryFrom::try_from`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// impl MyType {
    ///     fn from_js_value(value: &JsValue) -> Result<Self, JsValue> {
    ///         // Custom conversion logic
    ///     }
    /// }
    ///
    /// #[wasm(js_value(try_from = MyType::from_js_value))]
    /// struct MyType { /* fields */ }
    /// ```
    pub try_from: Option<Path>,

    /// Convert the type to/from JavaScript strings.
    ///
    /// When set to `true`, the type will be converted to and from JavaScript
    /// strings using the type's `Display` and `FromStr` implementations.
    /// This is useful for simple wrapper types around strings or types that
    /// have a natural string representation.
    ///
    /// This option cannot be used together with `into` or `try_from`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(js_value(convert_string))]
    /// struct UserId(String);  // Converts to/from string in JS
    ///
    /// #[wasm(js_value(convert_string))]
    /// struct Email(String);    // Converts to/from string in JS
    /// ```
    pub convert_string: bool,
}

/// Configuration for proxy type WASM bindings.
///
/// This structure allows you to use a proxy type for WASM bindings.
/// The proxy type handles the WASM conversion, and the original type
/// is converted to/from the proxy. This is useful for complex types
/// that need custom conversion logic or when you want to separate the
/// WASM representation from the domain model.
///
/// # Examples
///
/// ```rust,ignore
/// // Simple proxy
/// #[wasm(proxy = UserProxy)]
/// struct User { /* fields */ }
///
/// // Proxy with custom conversions
/// #[wasm(proxy(
///     source = UserProxy,
///     into = User::from_proxy,
///     try_from = User::to_proxy,
/// ))]
/// struct User { /* fields */ }
/// ```
#[derive(Debug)]
pub struct ProxyWasm {
    /// The proxy type to use for WASM bindings.
    ///
    /// This type must have its own WASM bindings and should be convertible
    /// to/from the original type. The proxy type handles all the WASM
    /// conversion logic.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[wasm(proxy = UserProxy)]
    /// struct User { /* fields */ }
    /// ```
    pub proxy: Path,

    /// Custom conversion function from original type to proxy type.
    ///
    /// Specifies a function that converts the original type into the proxy type.
    /// The function should take `self` and return the proxy type.
    ///
    /// If not specified, defaults to `Into::into`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// impl User {
    ///     fn to_proxy(self) -> UserProxy {
    ///         // Custom conversion logic
    ///     }
    /// }
    ///
    /// #[wasm(proxy(
    ///     source = UserProxy,
    ///     into = User::to_proxy,
    /// ))]
    /// struct User { /* fields */ }
    /// ```
    pub into: Option<Path>,

    /// Custom conversion function from proxy type to original type.
    ///
    /// Specifies a function that attempts to convert the proxy type into the original type.
    /// The function should take the proxy type and return a `Result<Self, ProxyType>`.
    ///
    /// If not specified, defaults to `TryFrom::try_from`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// impl User {
    ///     fn from_proxy(proxy: UserProxy) -> Result<Self, UserProxy> {
    ///         // Custom conversion logic
    ///     }
    /// }
    ///
    /// #[wasm(proxy(
    ///     source = UserProxy,
    ///     try_from = User::from_proxy,
    /// ))]
    /// struct User { /* fields */ }
    /// ```
    pub try_from: Option<Path>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(wasm))]
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
    /// Creates `WasmOptions` from a `DeriveInput`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input cannot be parsed as a valid struct or enum
    /// - Invalid attribute combinations are provided
    /// - Required attributes are missing
    /// - Attribute values are invalid
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
    ///
    /// Returns the Rust identifier for the type being processed.
    pub const fn ident(&self) -> &Ident {
        &self.serde_container.ident
    }

    /// Gets the name of the type.
    ///
    /// Returns the name to use for the type in TypeScript, taking into
    /// account any custom rename configuration.
    pub fn name(&self) -> &str {
        self.rename.as_ref().map_or_else(
            || self.serde_attrs().name().serialize_name(),
            String::as_str,
        )
    }

    /// Gets the serde data for the type.
    ///
    /// Returns the AST data representing the structure of the type.
    pub const fn serde_data(&self) -> &ast::Data<'_> {
        &self.serde_container.data
    }

    /// Gets the generic parameters for the type.
    ///
    /// Returns the generic parameters defined on the type.
    pub const fn generics(&self) -> &Generics {
        self.serde_container.generics
    }

    /// Gets the serde attributes for the type.
    ///
    /// Returns the serde container attributes that influence code generation.
    pub const fn serde_attrs(&self) -> &attr::Container {
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
