//! Serde utilities for the Bomboni ecosystem.

use std::{
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::Value as JsonValue;

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
        str_value
            .parse::<T>()
            .map_err(|_| <D as Deserializer<'de>>::Error::custom("unexpected string value"))
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
                // JSON numbers cannot be infinite in serde_json.
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

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};

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
