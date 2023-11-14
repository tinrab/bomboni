///Implement [`prost::Name`] for `CommandRequest`.
impl ::prost::Name for CommandRequest {
    const NAME: &'static str = "CommandRequest";
    const PACKAGE: &'static str = "tools";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl CommandRequest {
    pub const TYPE_URL: &'static str = "/CommandRequest";
}
impl CommandRequest {
    pub const COMMAND_FIELD_NAME: &'static str = "command";
}
///Implement [`prost::Name`] for `CommandResponse`.
impl ::prost::Name for CommandResponse {
    const NAME: &'static str = "CommandResponse";
    const PACKAGE: &'static str = "tools";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl CommandResponse {
    pub const TYPE_URL: &'static str = "/CommandResponse";
}
impl CommandResponse {
    pub const RESULT_FIELD_NAME: &'static str = "result";
}
impl ServingStatus {
    pub const NAME: &'static str = "ServingStatus";
    pub const PACKAGE: &'static str = "tools";
}
impl ServingStatus {
    pub const UNKNOWN_VALUE_NAME: &'static str = "UNKNOWN";
    pub const SERVING_VALUE_NAME: &'static str = "SERVING";
    pub const NOT_SERVING_VALUE_NAME: &'static str = "NOT_SERVING";
    pub const VALUE_NAMES: &'static [&'static str] = &[
        Self::UNKNOWN_VALUE_NAME,
        Self::SERVING_VALUE_NAME,
        Self::NOT_SERVING_VALUE_NAME,
    ];
}
impl ::serde::Serialize for ServingStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(self.as_str_name())
    }
}
impl<'de> ::serde::Deserialize<'de> for ServingStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> ::serde::de::Visitor<'de> for Visitor {
            type Value = ServingStatus;
            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", ServingStatus::VALUE_NAMES)
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|v| ServingStatus::try_from(v).ok())
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
                    .and_then(|v| ServingStatus::try_from(v).ok())
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
                ServingStatus::from_str_name(value)
                    .ok_or_else(|| ::serde::de::Error::unknown_variant(
                        value,
                        ServingStatus::VALUE_NAMES,
                    ))
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}
pub mod serving_status_serde {
    use super::*;
    use ::serde::{Serialize, Deserialize};
    pub fn serialize<S>(
        value: &i32,
        serializer: S,
    ) -> Result<<S as ::serde::Serializer>::Ok, <S as ::serde::Serializer>::Error>
    where
        S: ::serde::Serializer,
    {
        let value = ServingStatus::try_from(*value).unwrap();
        value.serialize(serializer)
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let value = ServingStatus::deserialize(deserializer)?;
        Ok(value as i32)
    }
}
