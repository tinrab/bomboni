use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::SystemTime;
use thiserror::Error;
use time::PrimitiveDateTime;
use time::{
    OffsetDateTime,
    convert::{Nanosecond, Second},
    format_description::well_known::Rfc3339,
};

#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;

/// A date and time in the UTC time zone.
///
/// This exists (temporary?) because other crates don't support WASM well.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    all(
        target_family = "wasm",
        not(any(target_os = "emscripten", target_os = "wasi")),
        feature = "wasm",
        not(feature = "js"),
    ),
    derive(bomboni_wasm::Wasm),
    wasm(
        bomboni_wasm_crate = bomboni_wasm,
        wasm_abi,
        js_value { convert_string },
    )
)]
#[cfg_attr(
    all(
        target_family = "wasm",
        not(any(target_os = "emscripten", target_os = "wasi")),
        feature = "wasm",
        feature = "js",
    ),
    derive(bomboni_wasm::Wasm),
    wasm(wasm_abi, js_value, override_type = "Date")
)]
pub struct UtcDateTime(OffsetDateTime);

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum UtcDateTimeError {
    #[error("invalid nanoseconds")]
    InvalidNanoseconds,
    #[error("not a UTC date time")]
    NotUtc,
    #[error("invalid date time string format `{0}`")]
    InvalidFormat(String),
    #[error("timestamp is out of range")]
    OutOfRange,
}

impl UtcDateTime {
    pub const UNIX_EPOCH: Self = Self(OffsetDateTime::UNIX_EPOCH);

    pub fn now() -> Self {
        Self(OffsetDateTime::now_utc())
    }

    pub const fn new(seconds: i64, nanoseconds: i32) -> Self {
        match OffsetDateTime::from_unix_timestamp_nanos(
            (seconds as i128) * (Nanosecond::per(Second) as i128) + nanoseconds as i128,
        ) {
            Ok(value) => Self(value),
            Err(_err) => Self(OffsetDateTime::UNIX_EPOCH),
        }
    }

    pub fn from_timestamp(seconds: i64, nanoseconds: i32) -> Result<Self, UtcDateTimeError> {
        OffsetDateTime::from_unix_timestamp_nanos(
            (i128::from(seconds)) * i128::from(Nanosecond::per(Second)) + i128::from(nanoseconds),
        )
        .map(Self)
        .map_err(|_| UtcDateTimeError::NotUtc)
    }

    pub fn from_seconds(seconds: i64) -> Result<Self, UtcDateTimeError> {
        match OffsetDateTime::from_unix_timestamp(seconds) {
            Err(_) => Err(UtcDateTimeError::NotUtc),
            Ok(value) => Ok(Self(value)),
        }
    }

    pub fn from_nanoseconds(nanoseconds: i128) -> Result<Self, UtcDateTimeError> {
        match OffsetDateTime::from_unix_timestamp_nanos(nanoseconds) {
            Err(_) => Err(UtcDateTimeError::NotUtc),
            Ok(value) => Ok(Self(value)),
        }
    }

    pub fn timestamp(self) -> (i64, i32) {
        (self.0.unix_timestamp(), self.0.nanosecond() as i32)
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

impl From<PrimitiveDateTime> for UtcDateTime {
    fn from(value: PrimitiveDateTime) -> Self {
        Self(value.assume_utc())
    }
}

impl From<UtcDateTime> for PrimitiveDateTime {
    fn from(value: UtcDateTime) -> Self {
        PrimitiveDateTime::new(value.0.date(), value.0.time())
    }
}

impl From<SystemTime> for UtcDateTime {
    fn from(value: SystemTime) -> Self {
        Self(OffsetDateTime::from(value))
    }
}

#[cfg(feature = "chrono")]
const _: () = {
    use chrono::{DateTime, NaiveDateTime, Utc};

    impl From<DateTime<Utc>> for UtcDateTime {
        fn from(value: DateTime<Utc>) -> Self {
            Self::from_timestamp(value.timestamp(), value.timestamp_subsec_nanos() as i32)
                // Always valid UTC
                .unwrap()
        }
    }

    impl From<UtcDateTime> for DateTime<Utc> {
        fn from(value: UtcDateTime) -> Self {
            let (seconds, nanoseconds) = value.timestamp();
            DateTime::from_timestamp_nanos(
                seconds * i64::from(Nanosecond::per(Second)) + i64::from(nanoseconds),
            )
        }
    }

    impl From<NaiveDateTime> for UtcDateTime {
        fn from(value: NaiveDateTime) -> Self {
            value.and_utc().into()
        }
    }

    impl From<UtcDateTime> for NaiveDateTime {
        fn from(value: UtcDateTime) -> Self {
            DateTime::from(value).naive_utc()
        }
    }
};

#[cfg(feature = "serde")]
const _: () = {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
};

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm",
    feature = "js"
))]
const _: () = {
    use wasm_bindgen::{JsCast, JsValue};

    impl From<UtcDateTime> for JsValue {
        fn from(value: UtcDateTime) -> Self {
            let mut date = js_sys::Date::new_with_year_month_day_hr_min_sec(
                value.year() as u32,
                Into::<u8>::into(value.month()) as i32 - 1,
                value.day() as i32,
                value.hour() as i32 + 1,
                value.minute() as i32,
                value.second() as i32,
            );

            let milliseconds = value.millisecond();
            if milliseconds > 0 {
                date.set_utc_milliseconds(milliseconds as u32);
            }

            date.into()
        }
    }

    impl TryFrom<JsValue> for UtcDateTime {
        type Error = JsValue;

        fn try_from(value: JsValue) -> Result<Self, Self::Error> {
            let date: js_sys::Date = value.unchecked_into();

            let iso = date
                .to_iso_string()
                .as_string()
                .ok_or_else(|| js_sys::Error::new("invalid date"))?;

            Ok(UtcDateTime::parse_rfc3339(iso)
                .map_err(|err| js_sys::Error::new(&err.to_string()))?)
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert() {
        assert_eq!(
            UtcDateTime::new(1, 0),
            UtcDateTime::from_str("1970-01-01T00:00:01Z").unwrap()
        );
        assert_eq!(
            UtcDateTime::new(0, 1),
            UtcDateTime::from_nanoseconds(1).unwrap()
        );

        assert_eq!(UtcDateTime::new(10, 20).timestamp(), (10, 20));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize() {
        let dt = UtcDateTime::now();
        let js = serde_json::to_string_pretty(&dt).unwrap();
        let parsed: UtcDateTime = serde_json::from_str(&js).unwrap();
        assert_eq!(dt, parsed);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn convert_chrono() {
        let chrono_naive =
            chrono::NaiveDateTime::parse_from_str("2020-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap();
        let utc = UtcDateTime::from(chrono_naive);
        assert_eq!(utc.to_string(), "2020-01-01T12:00:00Z");
        assert_eq!(chrono::NaiveDateTime::from(utc), chrono_naive);

        let chrono_naive_nanos = chrono::DateTime::from_timestamp(1337, 420)
            .unwrap()
            .naive_utc();
        let utc = UtcDateTime::from(chrono_naive_nanos);
        assert_eq!(utc.timestamp(), (1337, 420));
        assert_eq!(chrono::NaiveDateTime::from(utc), chrono_naive_nanos);

        let chrono_dt = chrono::DateTime::parse_from_rfc3339("2020-01-01T12:00:00Z")
            .unwrap()
            .to_utc();
        let utc = UtcDateTime::from(chrono_dt);
        assert_eq!(utc.to_string(), "2020-01-01T12:00:00Z");
        assert_eq!(chrono::DateTime::<chrono::Utc>::from(utc), chrono_dt);

        let chrono_dt_nanos = chrono::DateTime::from_timestamp(1337, 420).unwrap();
        let utc = UtcDateTime::from(chrono_dt_nanos);
        assert_eq!(utc.timestamp(), (1337, 420));
        assert_eq!(chrono::DateTime::<chrono::Utc>::from(utc), chrono_dt_nanos);
    }
}
