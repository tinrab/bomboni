//! # Utilities for working with API requests.
//!
//! This crate provides utilities for working API requests based on Google's AIP resource-oriented gRPC API conventions [1].
//!
//! [1]: https://google.aip.dev

#[allow(unused_extern_crates)]
extern crate regex;

pub mod error;
pub mod filter;
pub mod ordering;
pub mod parse;
pub mod query;
pub mod schema;
pub mod sql;
pub mod value;

#[cfg(feature = "testing")]
pub mod testing;

#[cfg(feature = "derive")]
pub mod derive {
    pub use bomboni_request_derive::*;
}
