///Implement [`prost::Name`] for `RetryInfo`.
impl ::prost::Name for RetryInfo {
    const NAME: &'static str = "RetryInfo";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl RetryInfo {
    pub const TYPE_URL: &'static str = "type.googleapis.com/RetryInfo";
}
impl RetryInfo {
    pub const RETRY_DELAY_FIELD_NAME: &'static str = "retry_delay";
}
///Implement [`prost::Name`] for `DebugInfo`.
impl ::prost::Name for DebugInfo {
    const NAME: &'static str = "DebugInfo";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl DebugInfo {
    pub const TYPE_URL: &'static str = "type.googleapis.com/DebugInfo";
}
impl DebugInfo {
    pub const STACK_ENTRIES_FIELD_NAME: &'static str = "stack_entries";
    pub const DETAIL_FIELD_NAME: &'static str = "detail";
}
///Implement [`prost::Name`] for `QuotaFailure`.
impl ::prost::Name for QuotaFailure {
    const NAME: &'static str = "QuotaFailure";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl QuotaFailure {
    pub const TYPE_URL: &'static str = "type.googleapis.com/QuotaFailure";
}
impl QuotaFailure {
    pub const VIOLATIONS_FIELD_NAME: &'static str = "violations";
}
///Implement [`prost::Name`] for `QuotaFailure.Violation`.
impl ::prost::Name for quota_failure::Violation {
    const NAME: &'static str = "QuotaFailure.Violation";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl quota_failure::Violation {
    pub const TYPE_URL: &'static str = "type.googleapis.com/QuotaFailure.Violation";
}
impl quota_failure::Violation {
    pub const SUBJECT_FIELD_NAME: &'static str = "subject";
    pub const DESCRIPTION_FIELD_NAME: &'static str = "description";
}
///Implement [`prost::Name`] for `ErrorInfo`.
impl ::prost::Name for ErrorInfo {
    const NAME: &'static str = "ErrorInfo";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl ErrorInfo {
    pub const TYPE_URL: &'static str = "type.googleapis.com/ErrorInfo";
}
impl ErrorInfo {
    pub const REASON_FIELD_NAME: &'static str = "reason";
    pub const DOMAIN_FIELD_NAME: &'static str = "domain";
    pub const METADATA_FIELD_NAME: &'static str = "metadata";
}
///Implement [`prost::Name`] for `PreconditionFailure`.
impl ::prost::Name for PreconditionFailure {
    const NAME: &'static str = "PreconditionFailure";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl PreconditionFailure {
    pub const TYPE_URL: &'static str = "type.googleapis.com/PreconditionFailure";
}
impl PreconditionFailure {
    pub const VIOLATIONS_FIELD_NAME: &'static str = "violations";
}
///Implement [`prost::Name`] for `PreconditionFailure.Violation`.
impl ::prost::Name for precondition_failure::Violation {
    const NAME: &'static str = "PreconditionFailure.Violation";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl precondition_failure::Violation {
    pub const TYPE_URL: &'static str = "type.googleapis.com/PreconditionFailure.Violation";
}
impl precondition_failure::Violation {
    pub const TYPE_FIELD_NAME: &'static str = "type";
    pub const SUBJECT_FIELD_NAME: &'static str = "subject";
    pub const DESCRIPTION_FIELD_NAME: &'static str = "description";
}
///Implement [`prost::Name`] for `BadRequest`.
impl ::prost::Name for BadRequest {
    const NAME: &'static str = "BadRequest";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl BadRequest {
    pub const TYPE_URL: &'static str = "type.googleapis.com/BadRequest";
}
impl BadRequest {
    pub const FIELD_VIOLATIONS_FIELD_NAME: &'static str = "field_violations";
}
///Implement [`prost::Name`] for `BadRequest.FieldViolation`.
impl ::prost::Name for bad_request::FieldViolation {
    const NAME: &'static str = "BadRequest.FieldViolation";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl bad_request::FieldViolation {
    pub const TYPE_URL: &'static str = "type.googleapis.com/BadRequest.FieldViolation";
}
impl bad_request::FieldViolation {
    pub const FIELD_FIELD_NAME: &'static str = "field";
    pub const DESCRIPTION_FIELD_NAME: &'static str = "description";
}
///Implement [`prost::Name`] for `RequestInfo`.
impl ::prost::Name for RequestInfo {
    const NAME: &'static str = "RequestInfo";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl RequestInfo {
    pub const TYPE_URL: &'static str = "type.googleapis.com/RequestInfo";
}
impl RequestInfo {
    pub const REQUEST_ID_FIELD_NAME: &'static str = "request_id";
    pub const SERVING_DATA_FIELD_NAME: &'static str = "serving_data";
}
///Implement [`prost::Name`] for `ResourceInfo`.
impl ::prost::Name for ResourceInfo {
    const NAME: &'static str = "ResourceInfo";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl ResourceInfo {
    pub const TYPE_URL: &'static str = "type.googleapis.com/ResourceInfo";
}
impl ResourceInfo {
    pub const RESOURCE_TYPE_FIELD_NAME: &'static str = "resource_type";
    pub const RESOURCE_NAME_FIELD_NAME: &'static str = "resource_name";
    pub const OWNER_FIELD_NAME: &'static str = "owner";
    pub const DESCRIPTION_FIELD_NAME: &'static str = "description";
}
///Implement [`prost::Name`] for `Help`.
impl ::prost::Name for Help {
    const NAME: &'static str = "Help";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl Help {
    pub const TYPE_URL: &'static str = "type.googleapis.com/Help";
}
impl Help {
    pub const LINKS_FIELD_NAME: &'static str = "links";
}
///Implement [`prost::Name`] for `Help.Link`.
impl ::prost::Name for help::Link {
    const NAME: &'static str = "Help.Link";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl help::Link {
    pub const TYPE_URL: &'static str = "type.googleapis.com/Help.Link";
}
impl help::Link {
    pub const DESCRIPTION_FIELD_NAME: &'static str = "description";
    pub const URL_FIELD_NAME: &'static str = "url";
}
///Implement [`prost::Name`] for `LocalizedMessage`.
impl ::prost::Name for LocalizedMessage {
    const NAME: &'static str = "LocalizedMessage";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl LocalizedMessage {
    pub const TYPE_URL: &'static str = "type.googleapis.com/LocalizedMessage";
}
impl LocalizedMessage {
    pub const LOCALE_FIELD_NAME: &'static str = "locale";
    pub const MESSAGE_FIELD_NAME: &'static str = "message";
}
impl Code {
    pub const NAME: &'static str = "Code";
    pub const PACKAGE: &'static str = "google.rpc";
}
impl Code {
    pub const OK_VALUE_NAME: &'static str = "OK";
    pub const CANCELLED_VALUE_NAME: &'static str = "CANCELLED";
    pub const UNKNOWN_VALUE_NAME: &'static str = "UNKNOWN";
    pub const INVALID_ARGUMENT_VALUE_NAME: &'static str = "INVALID_ARGUMENT";
    pub const DEADLINE_EXCEEDED_VALUE_NAME: &'static str = "DEADLINE_EXCEEDED";
    pub const NOT_FOUND_VALUE_NAME: &'static str = "NOT_FOUND";
    pub const ALREADY_EXISTS_VALUE_NAME: &'static str = "ALREADY_EXISTS";
    pub const PERMISSION_DENIED_VALUE_NAME: &'static str = "PERMISSION_DENIED";
    pub const UNAUTHENTICATED_VALUE_NAME: &'static str = "UNAUTHENTICATED";
    pub const RESOURCE_EXHAUSTED_VALUE_NAME: &'static str = "RESOURCE_EXHAUSTED";
    pub const FAILED_PRECONDITION_VALUE_NAME: &'static str = "FAILED_PRECONDITION";
    pub const ABORTED_VALUE_NAME: &'static str = "ABORTED";
    pub const OUT_OF_RANGE_VALUE_NAME: &'static str = "OUT_OF_RANGE";
    pub const UNIMPLEMENTED_VALUE_NAME: &'static str = "UNIMPLEMENTED";
    pub const INTERNAL_VALUE_NAME: &'static str = "INTERNAL";
    pub const UNAVAILABLE_VALUE_NAME: &'static str = "UNAVAILABLE";
    pub const DATA_LOSS_VALUE_NAME: &'static str = "DATA_LOSS";
    pub const VALUE_NAMES: &'static [&'static str] = &[
        Self::OK_VALUE_NAME,
        Self::CANCELLED_VALUE_NAME,
        Self::UNKNOWN_VALUE_NAME,
        Self::INVALID_ARGUMENT_VALUE_NAME,
        Self::DEADLINE_EXCEEDED_VALUE_NAME,
        Self::NOT_FOUND_VALUE_NAME,
        Self::ALREADY_EXISTS_VALUE_NAME,
        Self::PERMISSION_DENIED_VALUE_NAME,
        Self::UNAUTHENTICATED_VALUE_NAME,
        Self::RESOURCE_EXHAUSTED_VALUE_NAME,
        Self::FAILED_PRECONDITION_VALUE_NAME,
        Self::ABORTED_VALUE_NAME,
        Self::OUT_OF_RANGE_VALUE_NAME,
        Self::UNIMPLEMENTED_VALUE_NAME,
        Self::INTERNAL_VALUE_NAME,
        Self::UNAVAILABLE_VALUE_NAME,
        Self::DATA_LOSS_VALUE_NAME,
    ];
}
impl ::serde::Serialize for Code {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(self.as_str_name())
    }
}
impl<'de> ::serde::Deserialize<'de> for Code {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> ::serde::de::Visitor<'de> for Visitor {
            type Value = Code;
            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", Code::VALUE_NAMES)
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|v| Code::try_from(v).ok())
                    .ok_or_else(|| {
                        ::serde::de::Error::invalid_value(
                            ::serde::de::Unexpected::Signed(v),
                            &self,
                        )
                    })
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|v| Code::try_from(v).ok())
                    .ok_or_else(|| {
                        ::serde::de::Error::invalid_value(
                            ::serde::de::Unexpected::Unsigned(v),
                            &self,
                        )
                    })
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                Code::from_str_name(value)
                    .ok_or_else(|| ::serde::de::Error::unknown_variant(
                        value,
                        Code::VALUE_NAMES,
                    ))
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}
pub mod code_serde {
    use super::*;
    use ::serde::{Serialize, Deserialize};
    pub fn serialize<S>(
        value: &i32,
        serializer: S,
    ) -> Result<<S as ::serde::Serializer>::Ok, <S as ::serde::Serializer>::Error>
    where
        S: ::serde::Serializer,
    {
        let value = Code::try_from(*value).unwrap();
        value.serialize(serializer)
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let value = Code::deserialize(deserializer)?;
        Ok(value as i32)
    }
}
///Implement [`prost::Name`] for `Status`.
impl ::prost::Name for Status {
    const NAME: &'static str = "Status";
    const PACKAGE: &'static str = "google.rpc";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl Status {
    pub const TYPE_URL: &'static str = "type.googleapis.com/Status";
}
impl Status {
    pub const CODE_FIELD_NAME: &'static str = "code";
    pub const MESSAGE_FIELD_NAME: &'static str = "message";
    pub const DETAILS_FIELD_NAME: &'static str = "details";
}
