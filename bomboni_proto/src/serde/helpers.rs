use std::{
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de, ser};
use serde_json::Value as JsonValue;

use crate::google::protobuf::Duration;

/// Checks if a value is equal to its default value.
pub fn is_default<T>(value: &T) -> bool
where
    T: Default + PartialEq<T>,
{
    value == &T::default()
}

/// Returns `true` as a default value.
#[must_use]
pub const fn default_bool_true() -> bool {
    true
}

/// Serialization utilities for converting values to strings.
pub mod as_string {
    use super::{Deserialize, Deserializer, FromStr, Serializer, de};

    /// Serializes a value as a string.
    ///
    /// # Errors
    ///
    /// Will return the serializer's error if string serialization fails.
    pub fn serialize<T, S>(
        value: &T,
        serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        T: ToString,
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    /// Deserializes a string value.
    ///
    /// # Errors
    ///
    /// Will return the deserializer's error if string deserialization fails
    /// or if the string cannot be parsed into the target type.
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        D: Deserializer<'de>,
    {
        use de::Error;
        let str_value = String::deserialize(deserializer)?;
        let value = str_value
            .parse::<T>()
            .map_err(|_| <D as Deserializer<'de>>::Error::custom("unexpected string value"))?;
        Ok(value)
    }
}

/// Serialization utilities for comma-separated string lists.
pub mod string_list {
    use super::{
        Deserializer, Display, Formatter, FromStr, PhantomData, Serialize, Serializer, de, fmt,
    };

    /// Serializes a slice of values as a comma-separated string.
    ///
    /// # Errors
    ///
    /// Will return the serializer's error if string serialization fails.
    pub fn serialize<T, S>(
        value: &[T],
        serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        T: ToString,
        S: Serializer,
    {
        let value = value
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        value.serialize(serializer)
    }

    /// Deserializes a comma-separated string into a collection.
    ///
    /// # Errors
    ///
    /// Will return the deserializer's error if string deserialization fails
    /// or if any element cannot be parsed into the target type.
    pub fn deserialize<'de, V, T, D>(deserializer: D) -> Result<V, D::Error>
    where
        V: FromIterator<T>,
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        struct Visitor<V, T>(PhantomData<V>, PhantomData<T>);

        impl<V, T> de::Visitor<'_> for Visitor<V, T>
        where
            V: FromIterator<T>,
            T: FromStr,
            T::Err: Display,
        {
            type Value = V;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("string containing comma-separated elements")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let iter = s.split(',').map(FromStr::from_str);
                iter.collect::<Result<_, _>>().map_err(de::Error::custom)
            }
        }

        let visitor = Visitor(PhantomData, PhantomData);
        deserializer.deserialize_str(visitor)
    }
}

/// Checks if a JSON value is truthy.
///
/// Credit: <https://github.com/sunng87/handlebars-rust/blob/v4.5.0/src/json/value.rs#L113>
#[must_use]
pub fn is_truthy(value: &JsonValue, include_zero: bool) -> bool {
    match value {
        JsonValue::Bool(b) => *b,
        JsonValue::Number(n) => {
            if include_zero {
                n.as_f64().is_some_and(|f| !f.is_nan())
            } else {
                // there is no inifity in json/serde_json
                n.as_f64().is_some_and(f64::is_normal)
            }
        }
        JsonValue::Null => false,
        JsonValue::String(s) => !s.is_empty(),
        JsonValue::Array(a) => !a.is_empty(),
        JsonValue::Object(obj) => !obj.is_empty(),
    }
}

/// Merges two JSON values recursively.
pub fn merge_json(a: &mut JsonValue, b: JsonValue) {
    if let JsonValue::Object(a) = a
        && let JsonValue::Object(b) = b
    {
        for (k, v) in b {
            if v.is_null() {
                a.remove(&k);
            } else {
                merge_json(a.entry(k).or_insert(JsonValue::Null), v);
            }
        }
        return;
    }
    *a = b;
}

/// Serialization utilities for duration values.
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
        // Copy?
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

/// Serialization utilities for timestamp values as seconds.
pub mod timestamp_as_seconds {
    use super::{Deserialize, Deserializer, Serializer};
    use crate::google::protobuf::Timestamp;

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
        use super::{Deserializer, Serializer};
        use crate::google::protobuf::Timestamp;

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

    #[test]
    fn string_list() {
        #[derive(Serialize, Deserialize)]
        struct TestStringArray {
            #[serde(with = "string_list")]
            value: Vec<i32>,
        }

        let a = TestStringArray {
            value: vec![1, 2, 3],
        };
        let s = serde_json::to_string(&a).unwrap();
        let v: TestStringArray = serde_json::from_str(&s).unwrap();
        assert_eq!(v.value, a.value);
    }

    #[test]
    fn serde_as_string() {
        #[derive(Serialize, Deserialize)]
        struct TestIntString {
            #[serde(with = "as_string")]
            value: u64,
        }

        let v = TestIntString { value: 42 };
        let encoded = serde_json::to_string(&v).unwrap();
        assert_eq!(encoded, r#"{"value":"42"}"#);
        let decoded: TestIntString = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.value, v.value);
    }
}
