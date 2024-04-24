pub mod schema;

pub mod bomboni {
    pub mod proto {
        pub use bomboni_proto::*;
    }

    pub mod request {
        pub use crate::*;
    }
}
