# Bomboni: Utility Library for Rust

🚧 work in progress 🚧

A collection of Rust libraries for building robust applications.

This project includes utilities for working with WASM, extensions over protobuf/prost, tools for gRPC requests following Google AIP
designs, etc.

## Crates

This workspace contains the following crates:

- [bomboni_common](./bomboni_common/README.md): Common utilities for building distributed systems and applications, including ULID-based identifiers and UTC datetime handling
- [bomboni_core](./bomboni_core/README.md): Core utilities and abstractions used across the Bomboni project
- [bomboni_macros](./bomboni_macros/README.md): Common macros providing convenient utilities for the Bomboni library
- [bomboni_fs](./bomboni_fs/README.md): File system utilities for working with files and directories, including recursive file visiting and content reading

- [bomboni_prost](./bomboni_prost/README.md): Utilities for compiling Protocol Buffers with prost and generating additional helper functions
- [bomboni_proto](./bomboni_proto/README.md): Enhanced implementations of Google's well-known protobuf types with additional functionality beyond standard prost-types

- [bomboni_request](./bomboni_request/README.md): Comprehensive utilities for building API requests following Google AIP standards, with filtering, ordering, pagination, and SQL generation
- [bomboni_request_derive](./bomboni_request_derive/README.md): Derive macros and procedural macros for request parsing and type conversion

- [bomboni_wasm](./bomboni_wasm/README.md): WebAssembly utilities for JavaScript interoperability, console logging, and TypeScript declaration generation
- [bomboni_wasm_core](./bomboni_wasm_core/README.md): Core utilities for WebAssembly integration, including TypeScript declaration generation and type mapping
- [bomboni_wasm_derive](./bomboni_wasm_derive/README.md): Derive macros for generating TypeScript WASM bindings for Rust types

- [bomboni_template](./bomboni_template/README.md): Handlebars template utilities with custom helpers for template rendering
