use super::{
    protobuf::Any,
    rpc::{Code, Status},
};
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};

impl Status {
    pub fn new(code: Code, message: String, details: Vec<Any>) -> Self {
        Status {
            code: code as i32,
            message,
            details,
        }
    }
}

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        struct AnyProxy<'a>(&'a Any);
        impl<'a> ::serde::Serialize for AnyProxy<'a> {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<<S as ::serde::Serializer>::Ok, <S as ::serde::Serializer>::Error>
            where
                S: ::serde::Serializer,
            {
                detail_serde::serialize(self.0, serializer)
            }
        }

        #[derive(Serialize)]
        struct StatusProxy<'a> {
            code: Code,
            message: &'a str,
            details: Vec<AnyProxy<'a>>,
        }
        let proxy = StatusProxy {
            code: self.code.try_into().map_err(|_| {
                serde::ser::Error::custom(format!("invalid status code {}", self.code))
            })?,
            message: &self.message,
            details: self.details.iter().map(AnyProxy).collect::<Vec<_>>(),
        };
        proxy.serialize(serializer)
    }
}

// impl<'de> Deserialize<'de> for Status {
//     fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         luet mut status = deserializer.deserialize_struct(name, fields, visitor)
//     }
// }

pub mod detail_serde {
    use crate::{
        impl_proto_any_serde,
        proto::google::{
            protobuf::Any,
            rpc::{
                BadRequest, DebugInfo, ErrorInfo, Help, LocalizedMessage, PreconditionFailure,
                QuotaFailure, RequestInfo, ResourceInfo, RetryInfo,
            },
        },
    };
    use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};

    // impl Serialize for &[Any] {
    //     fn serialize<S>(
    //         &self,
    //         serializer: S,
    //     ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    //     where
    //         S: Serializer,
    //     {
    //         let mut seq = serializer.serialize_seq(Some(self.len()))?;
    //         for detail in *self {
    //             seq.serialize_element(detail)?;
    //         }
    //         seq.end()
    //     }
    // }

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

// pub mod details_serde {
//     pub fn serialize<S, T>(
//         details: &[T],
//         serializer: S,
//     ) -> Result<<S as ::serde::Serializer>::Ok, <S as ::serde::Serializer>::Error>
//     where
//         S: ::serde::Serializer,
//         T: TryInto<crate::proto::google::protobuf::Any>,
//     {
//         use ::serde::ser::SerializeSeq;

//         let mut seq = serializer.serialize_seq(Some(details.len()))?;

//         // let mut seq = serializer.serialize_seq(Some(details.len()))?;
//         // for detail in details {
//         //     // seq.serialize_element(detail)?;
//         //     // crate::proto::google::status::detail_serde::serialize(detail, seq);
//         //     let awd = crate::proto::google::status::detail_serde::serialize(detail, seq);
//         // }
//         // seq.end()
//         todo!()
//     }

//     //     pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
//     //     where
//     //         T: TryFrom<crate::proto::google::protobuf::Any>,
//     //         D: ::serde::Deserializer<'de>,
//     //     {
//     //         // use ::serde::ser::SerializeSeq;
//     //         // use serde::de::Error;
//     //         // let details: Vec<T> = Vec::deserialize(deserializer)?;
//     //         // Ok(details)
//     //         todo!()
//     //     }
// }

#[cfg(test)]
mod tests {
    use crate::proto::google::rpc::ErrorInfo;

    use super::*;

    #[test]
    fn serde() {
        let s = Status::new(
            Code::InvalidArgument,
            "error".to_string(),
            vec![Any::pack_from(&ErrorInfo {
                reason: "reason".to_string(),
                domain: "domain".to_string(),
                metadata: Default::default(),
            })
            .unwrap()],
        );
        println!("{}", serde_json::to_string_pretty(&s).unwrap());
    }
}
