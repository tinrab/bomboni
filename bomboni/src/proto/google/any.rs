use super::protobuf::Any;
use super::rpc::ErrorInfo;
use prost::{DecodeError, EncodeError, Message, Name};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

impl Any {
    pub fn new(type_url: String, value: Vec<u8>) -> Self {
        Any { type_url, value }
    }

    pub fn pack_from<T>(message: &T) -> Result<Self, EncodeError>
    where
        T: Name,
    {
        let type_url = T::type_url();
        let mut value = Vec::new();
        Message::encode(message, &mut value)?;
        Ok(Any { type_url, value })
    }

    pub fn unpack_into<T>(self) -> Result<T, DecodeError>
    where
        T: Default + Name,
    {
        let expected_type_url = T::type_url();
        if expected_type_url != self.type_url {
            let mut err = DecodeError::new(format!(
                "expected type URL `{}`, but got `{}`",
                expected_type_url, &self.type_url
            ));
            err.push("unexpected type URL", "type_url");
            return Err(err);
        }
        T::decode(&*self.value)
    }
}

#[macro_export(local_inner_macros)]
macro_rules! impl_proto_any_serde {
    ([$($message:ty),* $(,)?]) => {
        // impl ::serde::Serialize for $crate::proto::google::protobuf::Any {
        //     fn serialize<S>(&self, serializer: S) -> Result<<S as ::serde::Serializer>::Ok, <S as ::serde::Serializer>::Error>
        //     where
        //         S: ::serde::Serializer,
        //     {
        //         serialize(self, serializer)
        //     }
        // }

        // impl<'de> ::serde::Deserialize<'de> for $crate::proto::google::protobuf::Any {
        //     fn deserialize<D>(deserializer: D) -> std::result::Result<Self, <D as ::serde::Deserializer<'de>>::Error>
        //     where
        //         D: ::serde::Deserializer<'de>,
        //     {
        //         deserialize(deserializer)
        //     }
        // }

        pub fn serialize<S>(
            value: &$crate::proto::google::protobuf::Any,
            serializer: S,
        ) -> Result<<S as ::serde::Serializer>::Ok, <S as ::serde::Serializer>::Error>
        where
            S: ::serde::Serializer,
        {
            use ::prost::Name;
            use ::serde::{Serialize, Serializer};

            #[derive(::serde::Serialize)]
            struct AnySerdeProxy<T> {
                #[serde(rename = "@type")]
                type_url: String,
                #[serde(flatten)]
                message: T,
            }

            match value.type_url.as_str() {
                $(
                    <$message>::TYPE_URL => {
                        return AnySerdeProxy {
                            type_url: <$message>::TYPE_URL.into(),
                            message: value.clone().unpack_into::<$message>().unwrap(),
                        }.serialize(serializer);
                    }
                )*
                _ => {
                    ::core::unimplemented!("any serialize for type url {}", value.type_url)
                }
            }
        }

        pub fn deserialize<'de, D>(
            deserializer: D,
        ) -> Result<$crate::proto::google::protobuf::Any, <D as ::serde::Deserializer<'de>>::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            use ::prost::Name;
            use ::serde::Deserialize;

            #[derive(::serde::Deserialize)]
            struct AnySerdeProxy {
                #[serde(rename = "@type")]
                type_url: String,
                #[serde(flatten)]
                obj: ::std::collections::BTreeMap<String, ::serde_json::Value>,
            }

            let proxy = AnySerdeProxy::deserialize(deserializer)?;
            match proxy.type_url.as_str() {
                $(
                    <$message>::TYPE_URL => {
                        let message = ::serde_json::from_value::<$message>(
                            ::serde_json::Value::Object(proxy.obj.into_iter()
                        .collect())
                        ).map_err(|err| {
                            ::serde::de::Error::custom(::std::format!("failed to deserialize {}: {}", proxy.type_url, err))
                        })?;
                        $crate::proto::google::protobuf::Any::pack_from(&message)
                        .map_err(|err| {
                            ::serde::de::Error::custom(::std::format!("failed to pack {}: {}", proxy.type_url, err))
                        })
                    }
                )*
                _ => {
                    ::core::unimplemented!("any deserialize for type url {}", proxy.type_url)
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::proto::google::rpc::{ErrorInfo, RetryInfo};

    use super::*;

    #[test]
    fn it_works() {
        let msg = ErrorInfo {
            reason: "reason".to_string(),
            domain: "domain".to_string(),
            metadata: Default::default(),
        };
        let any = Any::pack_from(&msg).unwrap();
        let decoded: ErrorInfo = any.unpack_into().unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn errors() {
        let any = Any::pack_from(&ErrorInfo::default()).unwrap();
        assert!(any.unpack_into::<RetryInfo>().is_err());
    }
}
