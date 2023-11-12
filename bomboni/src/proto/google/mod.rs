mod any;
mod duration;
mod empty;
mod field_mask;
mod status;
mod timestamp;
mod wrappers;

pub mod protobuf {
    include!("../google.protobuf.rs");
    include!("../google.protobuf.plus.rs");
}

pub mod rpc {
    include!("../google.rpc.rs");
    include!("../google.rpc.plus.rs");
}
