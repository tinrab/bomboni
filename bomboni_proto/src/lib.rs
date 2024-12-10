mod protobuf;
pub mod rpc;
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

#[allow(unused_qualifications, clippy::all, clippy::pedantic)]
pub mod google {
    pub mod protobuf {
        pub use super::super::protobuf::*;
        crate::include_proto!("google.protobuf");
        crate::include_proto!("google.protobuf.plus");
    }
    pub mod rpc {
        pub use super::super::rpc::*;
        crate::include_proto!("google.rpc");
        crate::include_proto!("google.rpc.plus");
    }
}

#[cfg(test)]
mod tests {
    use google::protobuf::Timestamp;
    use prost::Name;

    use super::*;

    #[test]
    fn it_works() {
        println!("{}", Timestamp::type_url());
    }
}
