# bomboni: `wasm_derive`

Provides derive macros for generating TypeScript WASM bindings for Rust types.

This crate offers the `Wasm` derive macro that generates code to make Rust types
usable in JavaScript/TypeScript environments through wasm-bindgen, with automatic
TypeScript type generation.

## Features

- Automatic TypeScript type generation
- wasm-bindgen integration
- Custom type conversions
- Enum value objects
- Proxy types for complex conversions
- Reference type mapping
- Field and variant renaming
- Custom crate path configuration
- `JsValue` conversion support

## Quick Start

```rust,ignore
use bomboni_wasm_derive::Wasm;

#[derive(Wasm)]
#[wasm(rename = "User", into_wasm_abi = true, from_wasm_abi = true)]
struct User {
    #[wasm(rename = "userName")]
    name: String,

    #[wasm(override_type = "Date")]
    created_at: /* DateTime<Utc> */,

    #[wasm(always_some = true)]
    optional_field: Option<String>,
}
```

## Generated Output

The macro generates:
- TypeScript type definitions
- wasm-bindgen bindings
- Conversion traits (`FromWasmAbi`, `IntoWasmAbi`, etc.)
- Error handling for WASM conversions

## Struct Options

The `#[wasm(...)]` attribute can be applied to structs to control overall behavior:

- `rename = "Name"` - Custom name in TypeScript
- `into_wasm_abi = bool` - Generate `IntoWasmAbi` implementation
- `from_wasm_abi = bool` - Generate `FromWasmAbi` implementation
- `wasm_abi = bool` - Generate both `IntoWasmAbi` and `FromWasmAbi`
- `enum_value = bool` - Generate enum value object (for enums)
- `js_value(...)` - Custom `JsValue` conversion
- `proxy = Type` - Use proxy type for conversions
- `rename_all = "rule"` - Rename rule for all fields
- `override_type = "string"` - Override generated TypeScript type
- `change_ref = "mapping"` - Reference type mapping
- `*_crate = "path"` - Custom crate paths

## Field Options

The `#[wasm(...)]` attribute can be applied to fields:

- `rename = "name"` - Custom field name in TypeScript
- `override_type = "type"` - Override field type
- `rename_wrapper = bool` - Control wrapper type renaming
- `always_some = bool` - Force field to be required in TypeScript
- `change_ref = "mapping"` - Reference type mapping for field

## Advanced Examples

### Custom `JsValue` Conversions

```rust,ignore
#[derive(Wasm)]
#[wasm(js_value(
    into = MyType::to_js_value,
    try_from = MyType::from_js_value,
))]
struct MyType {
    value: String,
}

impl MyType {
    fn to_js_value(self) -> /* JsValue */ {
        // Custom conversion logic
    }

    fn from_js_value(value: &/* JsValue */) -> Result<Self, /* JsValue */> {
        // Custom conversion logic
    }
}
```

### String Conversions

```rust,ignore
#[derive(Wasm)]
#[wasm(js_value(convert_string))]
struct UserId(String);

#[derive(Wasm)]
#[wasm(js_value(convert_string))]
struct Email(String);
```

### Proxy Types

```rust,ignore
#[derive(Wasm)]
#[wasm(proxy = UserProxy)]
struct User {
    id: u32,
    name: String,
}

// With custom conversions
#[derive(Wasm)]
#[wasm(proxy(
    source = UserProxy,
    into = User::to_proxy,
    try_from = User::from_proxy,
))]
struct User2 {
    id: u32,
    name: String,
}
```

### Enum Value Objects

```rust,ignore
#[derive(Wasm)]
#[wasm(enum_value = true)]
enum Status {
    #[wasm(rename = "active")]
    Active,

    #[wasm(rename = "inactive")]
    Inactive,
}
```

### Reference Type Mapping

```rust,ignore
#[derive(Wasm)]
#[wasm(change_ref = "&str -> string")]
struct MyStruct {
    #[wasm(change_ref = "&[u8] -> Uint8Array")]
    data: Vec<u8>,
}
```

### Custom Crate Paths

```rust,ignore
#[derive(Wasm)]
#[wasm(
    wasm_bindgen_crate = "crate::wasm",
    js_sys_crate = "crate::js",
    bomboni_crate = "crate::internal::bomboni",
)]
struct MyStruct {
    field: String,
}
```
