//! Bookstore API implementation.
//!
//! This crate provides a complete implementation of a bookstore service with both
//! client and server components. It includes:
//!
//! - Client implementations for remote and in-memory services
//! - Model definitions for authors and books
//! - Error handling and parsing utilities
//! - Service request/response structures
//!
//! # Features
//!
//! - `client`: Enables client functionality for connecting to remote services

#[cfg(feature = "client")]
pub mod client;
pub mod model;

#[allow(
    unused_qualifications,
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::clone_on_ref_ptr,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_html_tags
)]
pub mod v1 {
    bomboni::proto::include_proto!("bookstore.v1");
    bomboni::proto::include_proto!("bookstore.v1.plus");

    pub use bomboni::proto::google::protobuf::{FieldMask, Timestamp};

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        bomboni::proto::include_file_descriptor_set!("bookstore_v1");

    pub mod errors {
        bomboni::proto::include_proto!("bookstore.v1.errors");
        bomboni::proto::include_proto!("bookstore.v1.errors.plus");
    }
}
