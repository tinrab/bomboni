use prost::{DecodeError, EncodeError, Message, Name};

use crate::google::protobuf::Any;

impl Any {
    /// Creates a new `Any` message with the given type URL and value.
    pub const fn new(type_url: String, value: Vec<u8>) -> Self {
        Self { type_url, value }
    }

    /// Converts a protobuf message to an `Any` message.
    ///
    /// # Errors
    ///
    /// Will return [`EncodeError`] if the message fails to encode.
    pub fn from_msg<T>(message: &T) -> Result<Self, EncodeError>
    where
        T: Name,
    {
        let type_url = T::type_url();
        let mut value = Vec::new();
        Message::encode(message, &mut value)?;
        Ok(Self { type_url, value })
    }

    /// Converts an `Any` message back to the original protobuf message.
    ///
    /// # Errors
    ///
    /// Will return [`DecodeError`] if the type URL doesn't match or decoding fails.
    pub fn to_msg<T>(self) -> Result<T, DecodeError>
    where
        T: Default + Name,
    {
        let expected_type_url = T::type_url();
        if expected_type_url != self.type_url {
            return Err(DecodeError::new_unexpected_type_url(
                &self.type_url,
                expected_type_url,
            ));
        }
        T::decode(&*self.value)
    }
}

/// Implements `TryFrom` conversions between protobuf messages and `Any` type.
#[macro_export(local_inner_macros)]
macro_rules! impl_proto_any_convert {
    ($($message:ty),* $(,)?) => {
    $(
        impl TryFrom<$message> for $crate::google::protobuf::Any {
            type Error = ::prost::EncodeError;

            fn try_from(value: $message) -> Result<Self, Self::Error> {
                $crate::google::protobuf::Any::from_msg(&value)
            }
        }

        impl TryFrom<$crate::google::protobuf::Any> for $message {
            type Error = ::prost::DecodeError;

            fn try_from(value: $crate::google::protobuf::Any) -> Result<Self, Self::Error> {
                value.to_msg()
            }
        }
    )*
    };
}

/// Implements serde serialization/deserialization for `Any` types.
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
            use ::prost::Name;

            #[derive(::serde::Serialize)]
            struct Proxy<T> {
                #[serde(rename = "@type")]
                type_url: String,
                #[serde(flatten)]
                message: T,
            }

            /*
                This crate used to provide `TYPE_URL` constant per messages.
                Now `prost` has the `Name` trait and `Name::type_url()`.
                TODO: Maybe change back.
            */

            // match value.type_url.as_str() {
            //     $(
            //         <$message>::TYPE_URL => {
            //             Proxy {
            //                 type_url: <$message>::TYPE_URL.into(),
            //                 message: value.clone().to_msg::<$message>().unwrap(),
            //             }.serialize(serializer)
            //         }
            //     )*
            //     _ => {
            //         ::core::unimplemented!("any serialize for type url {}", value.type_url)
            //     }
            // }
            $(
                let type_url = <$message>::type_url();
                if value.type_url == type_url {
                    return Proxy {
                        type_url,
                        message: value.clone().to_msg::<$message>().unwrap(),
                    }.serialize(serializer);
                }
            )*

            ::core::unimplemented!("any serialize for type url {}", value.type_url)
        }

        /// Deserialize different messages based on Type URL.
        /// We deserialize to a proxy Value from `pot` library then convert to the target message type.
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
            use ::prost::Name;

            const TYPE_FIELD_NAME: &'static str=  "@type";

            let proxy = Value::deserialize(deserializer)?;
            let (type_url, mappings) = if let Value::Mappings(mut mappings) = proxy {
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

            // match type_url.as_str() {
            //     $(
            //         <$message>::TYPE_URL => {
            //             let message = Value::Mappings(mappings)
            //             .deserialize_as::<$message>()
            //             .map_err(|err| {
            //                 Error::custom(::std::format!("failed to deserialize {}: {}", type_url, err))
            //             })?;
            //             $crate::google::protobuf::Any::pack_from(&message)
            //             .map_err(|err| {
            //                 Error::custom(::std::format!("failed to pack {}: {}", type_url, err))
            //             })
            //         }
            //     )*
            //     _ => {
            //         ::core::unimplemented!("any deserialize for type url {}", type_url)
            //     }
            // }

            $(
                if <$message>::type_url() == type_url {
                    let message = Value::Mappings(mappings)
                        .deserialize_as::<$message>()
                        .map_err(|err| {
                            Error::custom(::std::format!("failed to deserialize {}: {}", type_url, err))
                        })?;
                    return $crate::google::protobuf::Any::from_msg(&message)
                        .map_err(|err| {
                            Error::custom(::std::format!("failed to pack {}: {}", type_url, err))
                        });
                }
            )*

            ::core::unimplemented!("any deserialize for type url {}", type_url)
        }
    };
}

/// Implements serde serialization/deserialization for sequences of `Any` types.
#[macro_export(local_inner_macros)]
macro_rules! impl_proto_any_seq_serde {
    ($any_serde:ident) => {
        pub fn serialize<S>(
            details: &[$crate::google::protobuf::Any],
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            use serde::ser::SerializeSeq;

            struct Proxy<'a>(&'a $crate::google::protobuf::Any);

            impl<'a> ::serde::Serialize for Proxy<'a> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    $any_serde::serialize(self.0, serializer)
                }
            }

            let mut seq = serializer.serialize_seq(Some(details.len()))?;
            for detail in details {
                seq.serialize_element(&Proxy(detail))?;
            }

            seq.end()
        }

        pub fn deserialize<'de, D>(
            deserializer: D,
        ) -> Result<::std::vec::Vec<$crate::google::protobuf::Any>, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            use serde::Deserialize;

            struct Proxy($crate::google::protobuf::Any);

            impl<'de> ::serde::Deserialize<'de> for Proxy {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    $any_serde::deserialize(deserializer).map(Proxy)
                }
            }
            let details: Vec<Proxy> = Vec::deserialize(deserializer)?;
            Ok(details.into_iter().map(|p| p.0).collect())
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
        let any = Any::from_msg(&msg).unwrap();
        let decoded: ErrorInfo = any.to_msg().unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn errors() {
        let any = Any::from_msg(&ErrorInfo::default()).unwrap();
        assert!(any.to_msg::<RetryInfo>().is_err());
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
            any: Any::from_msg(&ErrorInfo {
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
