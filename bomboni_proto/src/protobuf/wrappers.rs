use std::{
    fmt::{Display, Formatter},
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::google::protobuf::{
    BoolValue, BytesValue, DoubleValue, FloatValue, Int32Value, Int64Value, StringValue,
    UInt32Value, UInt64Value,
};
use crate::serde::helpers as serde_helpers;

impl From<String> for StringValue {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl From<StringValue> for String {
    fn from(value: StringValue) -> Self {
        value.value
    }
}

impl From<&str> for StringValue {
    fn from(value: &str) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl From<Vec<u8>> for BytesValue {
    fn from(value: Vec<u8>) -> Self {
        Self { value }
    }
}

impl From<BytesValue> for Vec<u8> {
    fn from(value: BytesValue) -> Self {
        value.value
    }
}

macro_rules! impl_primitive_wrapper {
    ($type:tt, [ $($as:ty),* $(,)? ]) => {
        impl Display for $type {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.value, f)
            }
        }
        $(
            impl From<$as> for $type {
                fn from(value: $as) -> Self {
                    $type {
                        value: value.into(),
                    }
                }
            }
            impl From<&$as> for $type {
                fn from(value: &$as) -> Self {
                    $type {
                        value: (*value).into(),
                    }
                }
            }
            impl From<$type> for $as {
                fn from(value: $type) -> Self {
                    #![allow(trivial_casts, trivial_numeric_casts)]
                    value.value as $as
                }
            }
        )*
    };
}

impl_primitive_wrapper!(Int32Value, [i8, i16, i32]);
impl_primitive_wrapper!(UInt32Value, [u8, u16, u32]);
impl_primitive_wrapper!(Int64Value, [i8, i16, i32, i64]);
impl_primitive_wrapper!(UInt64Value, [u8, u16, u32, u64]);
impl_primitive_wrapper!(BoolValue, [bool]);
impl_primitive_wrapper!(FloatValue, [f32]);
impl_primitive_wrapper!(DoubleValue, [f32, f64]);

macro_rules! impl_size_wrapper {
    ($type:tt, $as:ty) => {
        impl From<isize> for $type {
            fn from(value: isize) -> Self {
                Self {
                    value: value as $as,
                }
            }
        }

        impl From<&isize> for $type {
            fn from(value: &isize) -> Self {
                Self {
                    value: (*value) as $as,
                }
            }
        }

        impl From<$type> for isize {
            fn from(value: $type) -> Self {
                #![allow(trivial_casts, trivial_numeric_casts)]
                value.value as isize
            }
        }

        impl From<usize> for $type {
            fn from(value: usize) -> Self {
                Self {
                    value: value as $as,
                }
            }
        }

        impl From<&usize> for $type {
            fn from(value: &usize) -> Self {
                Self {
                    value: (*value) as $as,
                }
            }
        }

        impl From<$type> for usize {
            fn from(value: $type) -> Self {
                #![allow(trivial_casts, trivial_numeric_casts)]
                value.value as usize
            }
        }
    };
}

impl_size_wrapper!(Int32Value, i32);
impl_size_wrapper!(UInt32Value, u32);

impl_size_wrapper!(Int64Value, i64);
impl_size_wrapper!(UInt64Value, u64);

macro_rules! impl_int_from_str {
    ($type:tt, $as:ty) => {
        impl FromStr for $type {
            type Err = ParseIntError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(<$as>::from_str(s)?.into())
            }
        }
    };
}
impl_int_from_str!(Int32Value, i32);
impl_int_from_str!(UInt32Value, u32);
impl_int_from_str!(Int64Value, i64);
impl_int_from_str!(UInt64Value, u64);

macro_rules! impl_float_from_str {
    ($type:tt, $as:ty) => {
        impl FromStr for $type {
            type Err = ParseFloatError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(<$as>::from_str(s)?.into())
            }
        }
    };
}

impl_float_from_str!(FloatValue, f32);
impl_float_from_str!(DoubleValue, f64);

macro_rules! impl_value_serde {
    ($type:ty, $as:ty) => {
        impl Serialize for $type {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
            where
                S: Serializer,
            {
                <$as>::serialize(&self.value, serializer)
            }
        }
        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
            where
                D: Deserializer<'de>,
            {
                let value = <$as>::deserialize(deserializer)?;
                Ok(value.into())
            }
        }
    };
}

impl_value_serde!(DoubleValue, f64);
impl_value_serde!(FloatValue, f32);
impl_value_serde!(Int32Value, i32);
impl_value_serde!(UInt32Value, u32);
impl_value_serde!(BoolValue, bool);
impl_value_serde!(StringValue, String);

impl Serialize for UInt64Value {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serde_helpers::as_string::serialize(&self.value, serializer)
    }
}

impl<'de> Deserialize<'de> for UInt64Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let value: u64 = serde_helpers::as_string::deserialize(deserializer)?;
        Ok(value.into())
    }
}

impl Serialize for Int64Value {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serde_helpers::as_string::serialize(&self.value, serializer)
    }
}

impl<'de> Deserialize<'de> for Int64Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let value: i64 = serde_helpers::as_string::deserialize(deserializer)?;
        Ok(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde() {
        let x: DoubleValue = 42f32.into();
        let encoded = serde_json::to_string(&x).unwrap();
        let decoded: DoubleValue = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, x);

        let x: UInt64Value = 42u64.into();
        let encoded = serde_json::to_string(&x).unwrap();
        let decoded: UInt64Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, x);

        let x: Int64Value = 42.into();
        let encoded = serde_json::to_string(&x).unwrap();
        let decoded: Int64Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, x);
    }
}
