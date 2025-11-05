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
    /// Returns an error if serialization fails.
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
    /// Returns an error if deserialization fails or the string cannot be parsed.
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
    /// Returns an error if serialization fails.
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
    /// Returns an error if deserialization fails or string cannot be parsed.
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
    /// Returns an error if serialization fails or duration conversion fails.
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
    /// Returns an error if deserialization fails or duration conversion fails.
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
