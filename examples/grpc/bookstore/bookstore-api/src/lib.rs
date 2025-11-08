#[cfg(feature = "client")]
pub mod client;
pub mod model;

#[allow(unused_qualifications)]
pub mod v1 {
    bomboni_proto::include_proto!("bookstore.v1");
    bomboni_proto::include_proto!("bookstore.v1.plus");

    pub use bomboni_proto::google::protobuf::Timestamp;

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        bomboni_proto::include_file_descriptor_set!("bookstore_v1");

    pub mod errors {
        bomboni_proto::include_proto!("bookstore.v1.errors");
        bomboni_proto::include_proto!("bookstore.v1.errors.plus");
    }
}
