# bomboni: `prost`

Provides code generation utilities for working with Protocol Buffers and Prost.

This crate offers functionality to compile protobuf files and generate Rust code with additional helper functions and utilities.

## Features

- Protobuf compilation with custom configuration
- Helper function generation
- Enum and message processing
- Oneof handling
- Path mapping utilities
- API configuration support

## Examples

```rust
use bomboni_prost::{compile, config::CompileConfig, ApiConfig, path_map::PathMap};
let config = CompileConfig {
    file_descriptor_set_path: "descriptor.bin".into(),
    output_path: "src/generated".into(),
    format: true,
    api: ApiConfig::default(),
    external_paths: PathMap::default(),
};
// compile(config)?;  // This would compile protobuf files
# Ok::<(), Box<dyn std::error::Error>>(())
```
