#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Command {
    #[prost(string, tag = "1")]
    pub user: ::prost::alloc::string::String,
    #[prost(oneof = "command::Kind", tags = "2, 4, 5")]
    pub kind: ::core::option::Option<command::Kind>,
}
/// Nested message and enum types in `Command`.
pub mod command {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Status {
        #[prost(oneof = "status::Kind", tags = "1, 2")]
        pub kind: ::core::option::Option<status::Kind>,
    }
    /// Nested message and enum types in `Status`.
    pub mod status {
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Kind {
            #[prost(bool, tag = "1")]
            Ok(bool),
            #[prost(string, tag = "2")]
            Error(::prost::alloc::string::String),
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Kind {
        #[prost(message, tag = "2")]
        Status(Status),
        #[prost(string, tag = "4")]
        Print(::prost::alloc::string::String),
        #[prost(message, tag = "5")]
        ApplyPerms(super::super::perms::Perms),
    }
}
