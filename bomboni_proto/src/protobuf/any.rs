use crate::google::protobuf::Any;
use prost::{DecodeError, EncodeError, Message, Name};
impl Any {
    #[must_use]
    pub fn new(type_url: String, value: Vec<u8>) -> Self {
        Self { type_url, value }
    }

    pub fn pack_from<T>(message: &T) -> Result<Self, EncodeError>
    where
        T: Name,
    {
        let type_url = T::type_url();
        let mut value = Vec::new();
        Message::encode(message, &mut value)?;
        Ok(Self { type_url, value })
    }

    pub fn unpack_into<T>(self) -> Result<T, DecodeError>
    where
        T: Default + Name,
    {
        let expected_type_url = T::type_url();
        if expected_type_url != self.type_url {
            return Err(DecodeError::new(format!(
                "expected type URL `{}`, but got `{}`",
                expected_type_url, &self.type_url
            )));
        }
        T::decode(&*self.value)
    }
}

#[macro_export(local_inner_macros)]
macro_rules! impl_proto_any_serde {
    ([$($message:ty),* $(,)?]) => {
        /// Serialize different messages determined by Type URL.
        pub fn serialize<S>(
            value: &$crate::google::protobuf::Any,
            serializer: S,
        ) -> Result<<S as ::serde::Serializer>::Ok, <S as ::serde::Serializer>::Error>
        where
            S: ::serde::Serializer,
        {
            use ::serde::Serialize;

            #[derive(::serde::Serialize)]
            struct Proxy<T> {
                #[serde(rename = "@type")]
                type_url: String,
                #[serde(flatten)]
                message: T,
            }

            match value.type_url.as_str() {
                $(
                    <$message>::TYPE_URL => {
                        Proxy {
                            type_url: <$message>::TYPE_URL.into(),
                            message: value.clone().unpack_into::<$message>().unwrap(),
                        }.serialize(serializer)
                    }
                )*
                _ => {
                    ::core::unimplemented!("any serialize for type url {}", value.type_url)
                }
            }
        }

        /// Deserialize different messages based on Type URL.
        /// We deserialize to a proxy Value form `pot` library then convert to the target message type.
        /// Pot is used because it's self-describing and supports all of serde's types.
        pub fn deserialize<'de, D>(
            deserializer: D,
        ) -> Result<$crate::google::protobuf::Any, <D as ::serde::Deserializer<'de>>::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            use ::serde::Deserialize;
            use ::serde::de::Error;
            use ::pot::Value;

            const TYPE_FIELD_NAME: &'static str=  "@type";

            let proxy = Value::deserialize(deserializer)?;
            let (type_url, mappings) =    if let Value::Mappings(mut mappings) = proxy {
                let type_url = mappings
                    .remove(
                        mappings
                            .iter()
                            .position(|(k, _)| ::std::matches!(k, Value::String(k) if k == TYPE_FIELD_NAME))
                            .ok_or_else(|| Error::missing_field(TYPE_FIELD_NAME))?,
                    )
                    .1;
                if let Value::String(type_url) = type_url {
                    (type_url.to_string(), mappings)
                } else {
                    return Err(Error::custom("expected a string @type field"));
                }
            } else {
                return Err(Error::custom("expected a map"));
            };

            match type_url.as_str() {
                $(
                    <$message>::TYPE_URL => {
                        let message = Value::Mappings(mappings)
                        .deserialize_as::<$message>()
                        .map_err(|err| {
                            Error::custom(::std::format!("failed to deserialize {}: {}", type_url, err))
                        })?;
                        $crate::google::protobuf::Any::pack_from(&message)
                        .map_err(|err| {
                            Error::custom(::std::format!("failed to pack {}: {}", type_url, err))
                        })
                    }
                )*
                _ => {
                    ::core::unimplemented!("any deserialize for type url {}", type_url)
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::google::rpc::{ErrorInfo, RetryInfo};
    use serde::{Deserialize, Serialize};

    use super::*;

    #[test]
    fn it_works() {
        let msg = ErrorInfo {
            reason: "reason".to_string(),
            domain: "domain".to_string(),
            metadata: BTreeMap::default(),
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

    #[test]
    fn any_serde() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Item {
            #[serde(with = "item_serde")]
            any: Any,
        }
        mod item_serde {
            use crate::google::rpc::ErrorInfo;

            impl_proto_any_serde!([ErrorInfo]);
        }

        let item = Item {
            any: Any::pack_from(&ErrorInfo {
                reason: "reason".to_string(),
                domain: "domain".to_string(),
                metadata: BTreeMap::default(),
            })
            .unwrap(),
        };
        let js = serde_json::to_string_pretty(&item).unwrap();
        let decoded = serde_json::from_str::<Item>(&js).unwrap();
        assert_eq!(decoded, item);
    }
}
