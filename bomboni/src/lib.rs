//! # Common utilities for Rust.
//!

//! A collection of common utilities for Rust.

#[cfg(feature = "macros")]
pub mod macros;

#[cfg(feature = "request")]
pub mod request;

#[cfg(feature = "testing")]
#[allow(clippy::all, missing_docs)]
pub mod testing;
