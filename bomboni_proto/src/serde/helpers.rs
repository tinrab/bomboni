use std::{fmt::Display, str::FromStr};

pub use bomboni_serde::{
    as_string, default_bool_true, is_default, is_truthy, merge_json, string_list,
};
use serde::{Deserialize, Deserializer, Serializer, de, ser};

use crate::google::protobuf::{Duration, Timestamp};

/// Serialization utilities for protobuf duration values.
pub mod duration {
    use super::{Deserialize, Deserializer, Display, Duration, FromStr, Serializer, de, ser};

    /// Serializes a duration value.
    ///
    /// # Errors
    ///
    /// Will return the serializer's error if string serialization fails
    /// or if the value cannot be converted to a duration.
    pub fn serialize<T, S>(
        value: &T,
        serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        T: Clone + TryInto<Duration>,
        <T as TryInto<Duration>>::Error: Display,
        S: Serializer,
    {
        use ser::Error;
        let d: Duration = value.clone().try_into().map_err(|err| {
            <S as Serializer>::Error::custom(format!("cannot serialize duration: {err}"))
        })?;
        serializer.serialize_str(&d.to_string())
    }

    /// Deserializes a duration value.
    ///
    /// # Errors
    ///
    /// Will return the deserializer's error if string deserialization fails,
    /// if the string cannot be parsed as a duration, or if the duration
    /// cannot be converted to the target type.
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: Clone + TryFrom<Duration>,
        <T as TryFrom<Duration>>::Error: Display,
        D: Deserializer<'de>,
    {
        use de::Error;
        let s = String::deserialize(deserializer)?;
        Duration::from_str(&s)
            .map_err(|err| {
                <D as Deserializer<'de>>::Error::custom(format!(
                    "cannot deserialize duration: {err}"
                ))
            })?
            .try_into()
            .map_err(|err| {
                <D as Deserializer<'de>>::Error::custom(format!("cannot convert duration: {err}"))
            })
    }
}

/// Serialization utilities for protobuf timestamp values as seconds.
pub mod timestamp_as_seconds {
    use super::{Deserialize, Deserializer, Serializer, Timestamp};

    /// Serializes a timestamp as seconds (f64).
    ///
    /// # Errors
    ///
    /// Will return the serializer's error if f64 serialization fails.
    #[allow(clippy::cast_precision_loss)]
    pub fn serialize<S>(
        value: &Timestamp,
        serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let seconds = value.seconds as f64 + f64::from(value.nanos) / 1_000_000_000.0;
        serializer.serialize_f64(seconds)
    }

    /// Deserializes a timestamp from seconds (f64).
    ///
    /// # Errors
    ///
    /// Will return the deserializer's error if f64 deserialization fails.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = f64::deserialize(deserializer)?;

        let whole_seconds = seconds.trunc() as i64;
        let nanos = ((seconds.fract() * 1_000_000_000.0).round() as i32).clamp(0, 999_999_999);

        Ok(Timestamp::new(whole_seconds, nanos))
    }

    /// Serialization utilities for optional timestamp values as seconds.
    pub mod option {
        use super::{Deserializer, Serializer, Timestamp};

        /// Serializes an optional timestamp as seconds (f64).
        ///
        /// # Errors
        ///
        /// Will return the serializer's error if f64 serialization fails.
        pub fn serialize<S>(
            value: &Option<Timestamp>,
            serializer: S,
        ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
        {
            match value {
                Some(timestamp) => super::serialize(timestamp, serializer),
                None => serializer.serialize_none(),
            }
        }

        /// Deserializes an optional timestamp from seconds (f64).
        ///
        /// # Errors
        ///
        /// Will return the deserializer's error if f64 deserialization fails.
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Timestamp>, D::Error>
        where
            D: Deserializer<'de>,
        {
            use serde::Deserialize;
            Option::<f64>::deserialize(deserializer)?
                .map(|seconds| {
                    let whole_seconds = seconds.trunc() as i64;
                    let nanos =
                        ((seconds.fract() * 1_000_000_000.0).round() as i32).clamp(0, 999_999_999);
                    Ok(Timestamp::new(whole_seconds, nanos))
                })
                .transpose()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn duration() {
        #[derive(Serialize, Deserialize)]
        struct TestDuration {
            #[serde(with = "duration")]
            value: Duration,
        }

        let d = TestDuration {
            value: Duration::new(3, 1),
        };
        let encoded = serde_json::to_string(&d).unwrap();
        assert_eq!(encoded, r#"{"value":"3.000000001s"}"#);
        let decoded: TestDuration = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.value, d.value);

        assert_eq!(
            serde_json::to_string(&Duration::new(3, 1000)).unwrap(),
            r#""3.000001s""#
        );

        assert_eq!(
            serde_json::from_str::<Duration>(r#""5.00000001s""#)
                .unwrap()
                .nanos,
            10
        );
        assert_eq!(
            serde_json::from_str::<Duration>(r#""5s""#).unwrap().nanos,
            0
        );
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn chrono_duration() {
        use chrono::Duration as ChronoDuration;

        #[derive(Serialize, Deserialize)]
        struct TestDuration {
            #[serde(with = "duration")]
            value: ChronoDuration,
        }

        let d = TestDuration {
            value: ChronoDuration::seconds(3) + ChronoDuration::nanoseconds(1),
        };
        let encoded = serde_json::to_string(&d).unwrap();
        assert_eq!(encoded, r#"{"value":"3.000000001s"}"#);
        let decoded: TestDuration = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.value, d.value);
    }
}
