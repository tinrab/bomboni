#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandRequest {
    #[prost(message, optional, tag = "1")]
    pub command: ::core::option::Option<command::Command>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandResponse {
    #[prost(string, tag = "1")]
    pub result: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ServingStatus {
    Unknown = 0,
    Serving = 1,
    NotServing = 2,
}
impl ServingStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ServingStatus::Unknown => "UNKNOWN",
            ServingStatus::Serving => "SERVING",
            ServingStatus::NotServing => "NOT_SERVING",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNKNOWN" => Some(Self::Unknown),
            "SERVING" => Some(Self::Serving),
            "NOT_SERVING" => Some(Self::NotServing),
            _ => None,
        }
    }
}
