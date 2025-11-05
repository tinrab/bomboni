# Bomboni: Utility Library for Rust

Work in progress (WIP).

A collection of Rust libraries for building robust applications.

This project includes utilities for working with WASM, extensions over protobuf/prost, tools for gRPC requests following Google AIP
designs, etc.

This root crate provides a unified interface to various bomboni sub-crates, each focused on specific functionality:

- **common**: Common utilities and data structures
- **prost**: Protocol Buffers code generation utilities
- **proto**: Protocol buffer definitions and generated code
- **request**: Request parsing and validation utilities
- **template**: Template rendering engine
- **wasm**: WebAssembly bindings and utilities
- **fs**: File system utilities
