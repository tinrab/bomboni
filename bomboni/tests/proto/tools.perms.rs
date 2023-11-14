#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Perms {
    #[prost(uint32, tag = "1")]
    pub code: u32,
}
