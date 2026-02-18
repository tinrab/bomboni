#![doc = include_str!("../README.md")]

#[allow(unused_extern_crates)]
extern crate regex;

/// Error handling types.
pub mod error;

/// Filter expression parsing and evaluation.
pub mod filter;

/// Query ordering parsing and validation.
pub mod ordering;

/// Parsing utilities and traits.
pub mod parse;

/// Query builders for list and search operations.
pub mod query;

/// Schema definitions for validation.
pub mod schema;

/// SQL generation utilities.
pub mod sql;

/// Value types and parsing.
pub mod value;

#[cfg(feature = "testing")]
/// Testing utilities and schemas.
pub mod testing;

#[cfg(feature = "derive")]
/// Derive macros for request parsing.
pub mod derive {
    pub use bomboni_request_derive::*;
}
