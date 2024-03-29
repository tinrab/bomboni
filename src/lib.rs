pub mod common {
    pub use bomboni_common::*;
}

#[cfg(feature = "prost")]
pub mod prost {
    pub use bomboni_prost::*;
}

#[cfg(feature = "proto")]
pub mod proto {
    pub use bomboni_proto::*;
}

#[cfg(feature = "request")]
pub mod request {
    pub use bomboni_request::*;
}

#[cfg(feature = "template")]
pub mod template {
    pub use bomboni_template::*;
}

#[cfg(feature = "wasm")]
pub mod wasm {
    pub use bomboni_wasm::*;
}

#[cfg(feature = "fs")]
pub mod fs {
    pub use bomboni_fs::*;
}
