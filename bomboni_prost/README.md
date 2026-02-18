# bomboni_prost

Utilities for working with prost. Part of Bomboni library.

This crate provides utilities for compiling Protocol Buffers with prost and generating additional helper functions and implementations.

## Features

- **Enhanced Code Generation**: Generates additional helper functions for protobuf messages and enums
- **Field Name Utilities**: Functions for getting field names and working with message fields
- **Oneof Utilities**: Helper functions for working with protobuf oneof fields
- **Serde Integration**: Automatic Serialize/Deserialize implementations for protobuf types
- **Path Mapping**: Support for mapping protobuf types to custom Rust types

## Examples

```rust,ignore
use bomboni_prost::{compile, config::CompileConfig, ApiConfig, path_map::PathMap};

let config = CompileConfig {
    file_descriptor_set_path: "descriptor.bin".into(),
    output_path: "src/generated".into(),
    helpers_mod: Some("helpers".into()),
    serde: true,
    format: true,
    api: ApiConfig::default(),
    external_paths: PathMap::default(),
};

compile(config)?; 
```

Generated code is in file with a `plus` suffix:

```rust,ignore
bomboni_proto::include_proto!("bookstore.v1"); // from prost-build
bomboni_proto::include_proto!("bookstore.v1.plus"); // ours
```

Here's is a stripped example of what is generated.

Source proto definitions:

```protobuf
message RetryInfo {
  google.protobuf.Duration retry_delay = 1;
}

enum Code {
  OK = 0;
  // ...
}

message Value {
  oneof kind {
    NullValue null_value = 1;
    double number_value = 2;
    string string_value = 3;
    bool bool_value = 4;
    Struct struct_value = 5;
    ListValue list_value = 6;
  }
}
```

```rust,ignore
impl RetryInfo {
    pub const RETRY_DELAY_FIELD_NAME: &'static str = "retry_delay";
}


impl Code {
    pub const NAME: &'static str = "Code";
    pub const PACKAGE: &'static str = "google.rpc";
    
    pub const OK_VALUE_NAME: &'static str = "OK";
    // ...
    pub const VALUE_NAMES: &'static [&'static str] = &[
        Self::OK_VALUE_NAME,
        // ...
    ];
}

impl ::serde::Serialize for Code {
    // ...
}
impl<'de> ::serde::Deserialize<'de> for Code {
    // ...
}

pub mod helpers {
    /// Utility for working with i32s in message fields.
    /// Usable with #[serde(with = "...")]
    pub mod code_serde {
        use ::serde::{Serialize, Deserialize};
        pub fn serialize<S>(
            value: &i32,
            serializer: S,
        ) -> Result<<S as ::serde::Serializer>::Ok, <S as ::serde::Serializer>::Error>
        where
            S: ::serde::Serializer,
        {
            // ...
        }
        pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            // ...
        }
    }
}


impl Value {
    pub const NULL_VALUE_FIELD_NAME: &'static str = "null_value";
    pub const STRING_VALUE_FIELD_NAME: &'static str = "string_value";
    // ...
    pub const KIND_ONEOF_NAME: &'static str = "kind";
}
impl value::Kind {
    pub const NULL_VALUE_VARIANT_NAME: &'static str = "null_value";
    pub const STRING_VALUE_VARIANT_NAME: &'static str = "string_value";
    // ...
}
impl value::Kind {
    pub fn get_variant_name(&self) -> &'static str {
        match self {
            Self::NullValue(_) => Self::NULL_VALUE_VARIANT_NAME,
            Self::StringValue(_) => Self::STRING_VALUE_VARIANT_NAME,
            // ...
        }
    }
}
impl From<value::Kind> for Value {
    fn from(value: value::Kind) -> Self {
        Self { kind: Some(value) }
    }
}
impl From<String> for value::Kind {
    fn from(value: String) -> Self {
        Self::StringValue(value.into())
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Self { kind: Some(value.into()) }
    }
}
// ...
```
