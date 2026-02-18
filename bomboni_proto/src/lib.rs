#![doc = include_str!("../README.md")]

mod protobuf;
mod rpc;

/// Serde integration for protobuf types.
pub mod serde;

/// Includes generated protobuf code.
/// Base path is specified with `OUT_DIR` environment variable.
#[macro_export]
macro_rules! include_proto {
    ($package: tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
    };
}

/// Includes generated protobuf file descriptor set.
#[macro_export]
macro_rules! include_file_descriptor_set {
    () => {
        include_file_descriptor_set!("fd");
    };
    ($name:tt) => {
        include_bytes!(concat!(env!("OUT_DIR"), concat!("/", $name, ".fd")));
    };
}

#[allow(
    unused_qualifications,
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_html_tags
)]
/// Generated Google protobuf and RPC types.
pub mod google {
    /// Generated Google protobuf message types.
    #[allow(rustdoc::broken_intra_doc_links, rustdoc::invalid_html_tags)]
    pub mod protobuf {
        crate::include_proto!("google.protobuf");
        crate::include_proto!("google.protobuf.plus");
    }
    /// Generated Google RPC status and error types.
    #[allow(rustdoc::broken_intra_doc_links, rustdoc::invalid_html_tags)]
    pub mod rpc {
        crate::include_proto!("google.rpc");
        crate::include_proto!("google.rpc.plus");
    }
}

#[cfg(test)]
mod tests {
    use google::rpc::BadRequest;
    use prost::Name;

    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(
            BadRequest::type_url(),
            "type.googleapis.com/google.rpc.BadRequest"
        );
        assert_eq!(BadRequest::FIELD_VIOLATIONS_FIELD_NAME, "field_violations");
    }
}
