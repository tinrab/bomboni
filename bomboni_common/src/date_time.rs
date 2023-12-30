#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    all(
        target_family = "wasm",
        not(any(target_os = "emscripten", target_os = "wasi")),
        feature = "wasm"
    ),
    // derive(WasmNewtype),
    // wasm(with = js_sys::Date),
    wasm_bindgen(inspectable),
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
}

impl UtcDateTime {
    pub const UNIX_EPOCH: Self = Self(OffsetDateTime::UNIX_EPOCH);

    pub fn now() -> Self {
        Self(OffsetDateTime::now_utc())
    }

    pub fn from_timestamp(seconds: i64, nanoseconds: u32) -> Result<Self, UtcDateTimeError> {
        OffsetDateTime::from_unix_timestamp_nanos(
            seconds as i128 * Nanosecond::per(Second) as i128 + nanoseconds as i128,
        )
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
        Ok(Self::parse_rfc3339(s)
            .map_err(|_| UtcDateTimeError::InvalidFormat(s.into()))?
            .into())
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
            seconds as i128 * Nanosecond::per(Second) as i128 + nanoseconds as i128,
        )
        .map_err(|_| UtcDateTimeError::NotUtc)?
        .into())
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<chrono::NaiveDateTime> for UtcDateTime {
    type Error = UtcDateTimeError;

    fn try_from(value: chrono::NaiveDateTime) -> Result<Self, Self::Error> {
        let seconds = value.timestamp();
        let nanoseconds = value.timestamp_subsec_nanos();
        if nanoseconds < 0 {
            return Err(UtcDateTimeError::InvalidNanoseconds);
        }
        Self::from_timestamp(seconds, nanoseconds as u32)
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<UtcDateTime> for chrono::NaiveDateTime {
    type Error = UtcDateTimeError;

    fn try_from(value: UtcDateTime) -> Result<Self, Self::Error> {
        let (seconds, nanoseconds) = value.timestamp();
        Self::from_timestamp_opt(seconds, nanoseconds).ok_or(UtcDateTimeError::NotUtc)
    }
}

#[cfg(feature = "serde")]
impl Serialize for UtcDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

#[cfg(feature = "serde")]
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

// #[cfg(all(
//     target_family = "wasm",
//     not(any(target_os = "emscripten", target_os = "wasi")),
//     feature = "wasm"
// ))]
// #[wasm_bindgen(typescript_custom_section)]
// const TS_APPEND_CONTENT: &'static str = r#"
//     export type UtcDateTime = Date;
// "#;

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
}
