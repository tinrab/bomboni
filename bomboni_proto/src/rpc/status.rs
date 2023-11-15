use crate::google::protobuf::Any;

use crate::google::rpc::{Code, Status};

impl Status {
    pub fn new(code: Code, message: String, details: Vec<Any>) -> Self {
        Status {
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
        let details = Status::decode(status.details()).unwrap();
        Status {
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
        tonic::Status::with_details(code, message, encoded_details.into())
    }
}

pub mod details_serde {
    use super::*;
    use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(details: &[Any], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        struct Proxy<'a>(&'a Any);
        impl<'a> Serialize for Proxy<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                detail_serde::serialize(self.0, serializer)
            }
        }
        let mut seq = serializer.serialize_seq(Some(details.len()))?;
        for detail in details {
            seq.serialize_element(&Proxy(detail))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Any>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Proxy(Any);
        impl<'de> Deserialize<'de> for Proxy {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                detail_serde::deserialize(deserializer).map(Proxy)
            }
        }
        let details: Vec<Proxy> = Vec::deserialize(deserializer)?;
        Ok(details.into_iter().map(|p| p.0).collect())
    }
}

pub mod detail_serde {
    use crate::{
        google::rpc::{
            BadRequest, DebugInfo, ErrorInfo, Help, LocalizedMessage, PreconditionFailure,
            QuotaFailure, RequestInfo, ResourceInfo, RetryInfo,
        },
        impl_proto_any_serde,
    };

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

#[cfg(test)]
mod tests {
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
                metadata: Default::default(),
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
