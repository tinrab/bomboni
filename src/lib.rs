#![doc = include_str!("../README.md")]

/// Common utilities and data structures.
pub mod common {
    pub use bomboni_common::*;
}

#[cfg(feature = "prost")]
/// Protocol Buffers utilities and extensions.
pub mod prost {
    pub use bomboni_prost::*;
}

#[cfg(feature = "proto")]
/// Generated Protocol Buffers messages.
pub mod proto {
    pub use bomboni_proto::*;
}

#[cfg(feature = "request")]
/// Request parsing and validation utilities.
pub mod request {
    pub use bomboni_request::*;
}

#[cfg(feature = "template")]
/// Template rendering and processing.
pub mod template {
    pub use bomboni_template::*;
}

#[cfg(feature = "wasm")]
/// WebAssembly support and utilities.
pub mod wasm {
    pub use bomboni_wasm::*;
}

#[cfg(feature = "fs")]
/// File system utilities.
pub mod fs {
    pub use bomboni_fs::*;
}
