use crate::google::protobuf::Any;

use crate::google::rpc::{
    BadRequest, Code, DebugInfo, ErrorInfo, Help, LocalizedMessage, PreconditionFailure,
    QuotaFailure, RequestInfo, ResourceInfo, RetryInfo, Status,
};
use crate::impl_proto_any_convert;

impl Status {
    #[must_use]
    pub fn new(code: Code, message: String, details: Vec<Any>) -> Self {
        Self {
            code: code as i32,
            message,
            details,
        }
    }
}

#[cfg(feature = "tonic")]
impl From<tonic::Status> for Status {
    fn from(status: tonic::Status) -> Self {
        use prost::Message;
        let details = Self::decode(status.details()).unwrap();
        Self {
            code: status.code() as i32,
            message: status.message().into(),
            details: details.details,
        }
    }
}

#[cfg(feature = "tonic")]
impl From<Status> for tonic::Status {
    fn from(status: Status) -> Self {
        use prost::Message;
        let code = tonic::Code::from(status.code);
        let message = status.message.clone();
        let mut encoded_details = Vec::new();
        status.encode(&mut encoded_details).unwrap();
        Self::with_details(code, message, encoded_details.into())
    }
}

pub mod detail_serde {
    use super::{
        BadRequest, DebugInfo, ErrorInfo, Help, LocalizedMessage, PreconditionFailure,
        QuotaFailure, RequestInfo, ResourceInfo, RetryInfo,
    };
    use crate::impl_proto_any_serde;

    impl_proto_any_serde!([
        BadRequest,
        DebugInfo,
        ErrorInfo,
        Help,
        LocalizedMessage,
        PreconditionFailure,
        QuotaFailure,
        RequestInfo,
        ResourceInfo,
        RetryInfo,
    ]);
}

pub mod details_serde {
    use super::detail_serde;
    use crate::impl_proto_any_seq_serde;

    impl_proto_any_seq_serde!(detail_serde);
}

impl_proto_any_convert![
    BadRequest,
    DebugInfo,
    ErrorInfo,
    Help,
    LocalizedMessage,
    PreconditionFailure,
    QuotaFailure,
    RequestInfo,
    ResourceInfo,
    RetryInfo,
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::google::rpc::ErrorInfo;

    use super::*;

    #[test]
    fn serde() {
        let s = Status::new(
            Code::InvalidArgument,
            "error".to_string(),
            vec![Any::pack_from(&ErrorInfo {
                reason: "a".to_string(),
                domain: "b".to_string(),
                metadata: BTreeMap::default(),
            })
            .unwrap()],
        );
        let js = serde_json::to_string(&s).unwrap();
        assert_eq!(
            js,
            r#"{"code":"INVALID_ARGUMENT","message":"error","details":[{"@type":"type.googleapis.com/google.rpc.ErrorInfo","reason":"a","domain":"b"}]}"#
        );
        let decoded: Status = serde_json::from_str(&js).unwrap();
        assert_eq!(decoded, s);
    }
}
