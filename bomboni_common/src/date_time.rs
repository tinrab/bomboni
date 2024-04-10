use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use time::convert::{Nanosecond, Second};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
use wasm_bindgen::{
    convert::{FromWasmAbi, IntoWasmAbi, OptionFromWasmAbi, OptionIntoWasmAbi},
    describe::WasmDescribe,
    prelude::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcDateTime(OffsetDateTime);

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum UtcDateTimeError {
    #[error("invalid nanoseconds")]
    InvalidNanoseconds,
    #[error("not a UTC date time")]
    NotUtc,
    #[error("invalid date time string format `{0}`")]
    InvalidFormat(String),
}

impl UtcDateTime {
    pub const UNIX_EPOCH: Self = Self(OffsetDateTime::UNIX_EPOCH);

    pub fn now() -> Self {
        Self(OffsetDateTime::now_utc())
    }

    pub fn from_timestamp(seconds: i64, nanoseconds: u32) -> Result<Self, UtcDateTimeError> {
        OffsetDateTime::from_unix_timestamp_nanos(
            i128::from(seconds) * i128::from(Nanosecond::per(Second)) + i128::from(nanoseconds),
        )
        .map_err(|_| UtcDateTimeError::NotUtc)
        .map(Self)
    }

    pub fn from_ymd(year: i32, month: u8, day: u8) -> Result<Self, UtcDateTimeError> {
        OffsetDateTime::UNIX_EPOCH
            .replace_year(year)
            .and_then(|dt| dt.replace_month(month.try_into().unwrap()))
            .and_then(|dt| dt.replace_day(day))
            .map_err(|_| UtcDateTimeError::NotUtc)
            .map(Self)
    }

    pub fn with_ymd(self, year: i32, month: u8, day: u8) -> Result<Self, UtcDateTimeError> {
        self.0
            .replace_year(year)
            .and_then(|dt| dt.replace_month(month.try_into().unwrap()))
            .and_then(|dt| dt.replace_day(day))
            .map_err(|_| UtcDateTimeError::NotUtc)
            .map(Self)
    }

    pub fn with_hms(self, hour: u8, minute: u8, second: u8) -> Result<Self, UtcDateTimeError> {
        self.0
            .replace_hour(hour)
            .and_then(|dt| dt.replace_minute(minute))
            .and_then(|dt| dt.replace_second(second))
            .map_err(|_| UtcDateTimeError::NotUtc)
            .map(Self)
    }

    pub fn timestamp(self) -> (i64, u32) {
        (self.0.unix_timestamp(), self.0.nanosecond())
    }

    pub fn parse_rfc3339<S: AsRef<str>>(input: S) -> Result<Self, UtcDateTimeError> {
        OffsetDateTime::parse(input.as_ref(), &Rfc3339)
            .map_err(|_| UtcDateTimeError::InvalidFormat(input.as_ref().into()))
            .map(Self)
    }

    pub fn format_rfc3339(&self) -> Result<String, time::error::Format> {
        self.0.format(&Rfc3339)
    }
}

impl Deref for UtcDateTime {
    type Target = OffsetDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for UtcDateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.format_rfc3339() {
            Ok(odt) => odt.fmt(f),
            Err(_) => "INVALID_UTC_DATE_TIME".fmt(f),
        }
    }
}

impl FromStr for UtcDateTime {
    type Err = UtcDateTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_rfc3339(s).map_err(|_| UtcDateTimeError::InvalidFormat(s.into()))
    }
}

impl From<OffsetDateTime> for UtcDateTime {
    fn from(value: OffsetDateTime) -> Self {
        Self(value)
    }
}

impl From<UtcDateTime> for OffsetDateTime {
    fn from(value: UtcDateTime) -> Self {
        value.0
    }
}

impl TryFrom<SystemTime> for UtcDateTime {
    type Error = UtcDateTimeError;

    fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
        const NANOS_PER_SECOND: i32 = 1_000_000_000;
        let (seconds, nanoseconds) = match value.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let seconds = i64::try_from(duration.as_secs()).unwrap();
                (seconds, duration.subsec_nanos() as i32)
            }
            Err(error) => {
                let duration = error.duration();
                let seconds = i64::try_from(duration.as_secs()).unwrap();
                let nanoseconds = duration.subsec_nanos() as i32;
                if nanoseconds == 0 {
                    (-seconds, 0)
                } else {
                    (-seconds - 1, NANOS_PER_SECOND - nanoseconds)
                }
            }
        };
        Ok(OffsetDateTime::from_unix_timestamp_nanos(
            i128::from(seconds) * i128::from(Nanosecond::per(Second)) + i128::from(nanoseconds),
        )
        .map_err(|_| UtcDateTimeError::NotUtc)?
        .into())
    }
}

#[cfg(feature = "chrono")]
mod chrono_impl {
    use super::{UtcDateTime, UtcDateTimeError};
    use chrono::{DateTime, NaiveDateTime, Utc};

    impl TryFrom<NaiveDateTime> for UtcDateTime {
        type Error = UtcDateTimeError;

        fn try_from(value: NaiveDateTime) -> Result<Self, Self::Error> {
            let utc = value.and_utc();
            Self::from_timestamp(utc.timestamp(), utc.timestamp_subsec_nanos())
        }
    }

    impl TryFrom<UtcDateTime> for NaiveDateTime {
        type Error = UtcDateTimeError;

        fn try_from(value: UtcDateTime) -> Result<Self, Self::Error> {
            DateTime::try_from(value).map(|dt| dt.naive_utc())
        }
    }

    impl TryFrom<DateTime<Utc>> for UtcDateTime {
        type Error = UtcDateTimeError;

        fn try_from(value: DateTime<Utc>) -> Result<Self, Self::Error> {
            Self::from_timestamp(value.timestamp(), value.timestamp_subsec_nanos())
        }
    }

    impl TryFrom<UtcDateTime> for DateTime<Utc> {
        type Error = UtcDateTimeError;

        fn try_from(value: UtcDateTime) -> Result<Self, Self::Error> {
            let (seconds, nanoseconds) = value.timestamp();
            DateTime::from_timestamp(seconds, nanoseconds).ok_or(UtcDateTimeError::NotUtc)
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::UtcDateTime;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::str::FromStr;

    impl Serialize for UtcDateTime {
        fn serialize<S>(
            &self,
            serializer: S,
        ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
        {
            let s = self.to_string();
            serializer.serialize_str(&s)
        }
    }

    impl<'de> Deserialize<'de> for UtcDateTime {
        fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
        where
            D: Deserializer<'de>,
        {
            use serde::de;
            let s = String::deserialize(deserializer)?;
            Self::from_str(&s).map_err(de::Error::custom)
        }
    }
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm",
    not(feature = "js")
))]
mod wasm_as_string {
    use super::*;

    impl WasmDescribe for UtcDateTime {
        fn describe() {
            <js_sys::JsString as WasmDescribe>::describe()
        }
    }

    impl IntoWasmAbi for UtcDateTime {
        type Abi = <js_sys::JsString as IntoWasmAbi>::Abi;

        fn into_abi(self) -> Self::Abi {
            js_sys::JsString::from(self.format_rfc3339().unwrap()).into_abi()
        }
    }

    impl OptionIntoWasmAbi for UtcDateTime {
        #[inline]
        fn none() -> Self::Abi {
            <js_sys::JsString as OptionIntoWasmAbi>::none()
        }
    }

    impl FromWasmAbi for UtcDateTime {
        type Abi = <js_sys::JsString as FromWasmAbi>::Abi;

        unsafe fn from_abi(js: Self::Abi) -> Self {
            match js_sys::JsString::from_abi(js)
                .as_string()
                .as_ref()
                .map(|s| Self::parse_rfc3339(s))
            {
                Some(Ok(value)) => value,
                Some(Err(err)) => {
                    wasm_bindgen::throw_str(&err.to_string());
                }
                None => {
                    wasm_bindgen::throw_str("expected RFC 3339 date string");
                }
            }
        }
    }

    impl OptionFromWasmAbi for UtcDateTime {
        #[inline]
        fn is_none(js: &Self::Abi) -> bool {
            <js_sys::JsString as OptionFromWasmAbi>::is_none(js)
        }
    }

    #[wasm_bindgen(typescript_custom_section)]
    const TS_APPEND_CONTENT: &'static str = r#"
        export type UtcDateTime = string;
    "#;
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm",
    feature = "js"
))]
mod wasm_as_date {
    use super::*;

    impl WasmDescribe for UtcDateTime {
        fn describe() {
            <js_sys::Date as WasmDescribe>::describe()
        }
    }

    impl IntoWasmAbi for UtcDateTime {
        type Abi = <js_sys::Date as IntoWasmAbi>::Abi;

        fn into_abi(self) -> Self::Abi {
            js_sys::Date::from(self.0).into_abi()
        }
    }

    impl OptionIntoWasmAbi for UtcDateTime {
        #[inline]
        fn none() -> Self::Abi {
            <js_sys::Date as OptionIntoWasmAbi>::none()
        }
    }

    impl FromWasmAbi for UtcDateTime {
        type Abi = <js_sys::Date as FromWasmAbi>::Abi;

        unsafe fn from_abi(js: Self::Abi) -> Self {
            OffsetDateTime::from(js_sys::Date::from_abi(js)).into()
        }
    }

    impl OptionFromWasmAbi for UtcDateTime {
        #[inline]
        fn is_none(js: &Self::Abi) -> bool {
            <js_sys::Date as OptionFromWasmAbi>::is_none(js)
        }
    }

    #[wasm_bindgen(typescript_custom_section)]
    const TS_APPEND_CONTENT: &'static str = r#"
        export type UtcDateTime = Date;
    "#;
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn it_works() {
        let dt = UtcDateTime::now();
        let js = serde_json::to_string_pretty(&dt).unwrap();
        let parsed: UtcDateTime = serde_json::from_str(&js).unwrap();
        assert_eq!(dt, parsed);

        assert_eq!(
            UtcDateTime::try_from(UNIX_EPOCH + Duration::from_secs(1)).unwrap(),
            UtcDateTime::from_str("1970-01-01T00:00:01Z").unwrap()
        );
        assert_eq!(
            UtcDateTime::try_from(UNIX_EPOCH + Duration::from_nanos(1)).unwrap(),
            UtcDateTime::from_timestamp(0, 1).unwrap()
        );

        assert_eq!(
            UtcDateTime::from_timestamp(10, 20).unwrap().timestamp(),
            (10, 20)
        );
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn convert_chrono() {
        let chrono_naive =
            chrono::NaiveDateTime::parse_from_str("2020-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap();
        let utc = UtcDateTime::try_from(chrono_naive).unwrap();
        assert_eq!(utc.to_string(), "2020-01-01T12:00:00Z");
        assert_eq!(chrono::NaiveDateTime::try_from(utc).unwrap(), chrono_naive);

        let chrono_naive_nanos = chrono::DateTime::from_timestamp(1337, 420)
            .unwrap()
            .naive_utc();
        let utc = UtcDateTime::try_from(chrono_naive_nanos).unwrap();
        assert_eq!(utc.timestamp(), (1337, 420));
        assert_eq!(
            chrono::NaiveDateTime::try_from(utc).unwrap(),
            chrono_naive_nanos
        );

        let chrono_dt = chrono::DateTime::parse_from_rfc3339("2020-01-01T12:00:00Z")
            .unwrap()
            .to_utc();
        let utc = UtcDateTime::try_from(chrono_dt).unwrap();
        assert_eq!(utc.to_string(), "2020-01-01T12:00:00Z");
        assert_eq!(
            chrono::DateTime::<chrono::Utc>::try_from(utc).unwrap(),
            chrono_dt,
        );

        let chrono_dt_nanos = chrono::DateTime::from_timestamp(1337, 420).unwrap();
        let utc = UtcDateTime::try_from(chrono_dt_nanos).unwrap();
        assert_eq!(utc.timestamp(), (1337, 420));
        assert_eq!(
            chrono::DateTime::<chrono::Utc>::try_from(utc).unwrap(),
            chrono_dt_nanos,
        );
    }
}
