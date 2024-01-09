use bomboni_common::date_time::{UtcDateTime, UtcDateTimeError};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
    time::SystemTime,
};
use time::OffsetDateTime;
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

use crate::google::protobuf::Timestamp;

const NANOS_PER_SECOND: i32 = 1_000_000_000;

impl Timestamp {
    #[must_use]
    pub const fn new(seconds: i64, nanos: i32) -> Self {
        Self { seconds, nanos }
    }

    /// Get Timestamp normalized to a canonical format.
    /// Based on [1] and [2].
    ///
    /// [1]: https://github.com/tokio-rs/prost/blob/v0.12.1/prost-types/src/lib.rs#L274
    /// [2]: https://github.com/google/protobuf/blob/v3.3.2/src/google/protobuf/util/time_util.cc#L59-L77
    #[must_use]
    pub fn normalized(self) -> Self {
        // const TIMESTAMP_MIN_SECONDS: i64 = -62_135_596_800;
        // const TIMESTAMP_MAX_SECONDS: i64 = 253_402_300_799;

        let mut seconds = self.seconds;
        let mut nanos = self.nanos;

        // Make sure nanos is in the range.
        if nanos <= -NANOS_PER_SECOND || nanos >= NANOS_PER_SECOND {
            if let Some(new_seconds) =
                seconds.checked_add(i64::from(nanos) / i64::from(NANOS_PER_SECOND))
            {
                seconds = new_seconds;
                nanos %= NANOS_PER_SECOND;
            } else if nanos < 0 {
                seconds = i64::MIN;
                nanos = 0;
            } else {
                seconds = i64::MAX;
                nanos = 999_999_999;
            }
        }

        // https://github.com/protocolbuffers/protobuf/blob/v3.3.2/src/google/protobuf/util/time_util.cc#L66
        // Timestamp nanos should be in the range [0, 999999999]
        if nanos < 0 {
            if let Some(new_seconds) = seconds.checked_sub(1) {
                seconds = new_seconds;
                nanos += NANOS_PER_SECOND;
            } else {
                nanos = 0;
            }
        }

        // debug_assert!(
        //     seconds >= TIMESTAMP_MIN_SECONDS && seconds <= TIMESTAMP_MAX_SECONDS,
        //     "seconds out of range: {}",
        //     seconds
        // );

        Self { seconds, nanos }
    }

    // pub fn is_normalized(self) -> bool {
    //     let n = self.normalized();
    //     n.seconds != self.seconds && (n.seconds == i64::MIN || n.seconds == i64::MAX)
    // }
}

impl From<UtcDateTime> for Timestamp {
    fn from(value: UtcDateTime) -> Self {
        let (seconds, nanoseconds) = value.timestamp();
        Self {
            seconds,
            nanos: nanoseconds as i32,
        }
    }
}

impl TryFrom<Timestamp> for UtcDateTime {
    type Error = UtcDateTimeError;

    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        if value.nanos < 0 {
            return Err(UtcDateTimeError::InvalidNanoseconds);
        }
        UtcDateTime::from_timestamp(value.seconds, value.nanos as u32)
    }
}

impl From<OffsetDateTime> for Timestamp {
    fn from(value: OffsetDateTime) -> Self {
        UtcDateTime::from(value).into()
    }
}

impl TryFrom<Timestamp> for OffsetDateTime {
    type Error = UtcDateTimeError;

    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        UtcDateTime::try_from(value).map(Into::into)
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<chrono::NaiveDateTime> for Timestamp {
    type Error = UtcDateTimeError;

    fn try_from(value: chrono::NaiveDateTime) -> Result<Self, Self::Error> {
        UtcDateTime::try_from(value).map(Into::into)
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<Timestamp> for chrono::NaiveDateTime {
    type Error = UtcDateTimeError;

    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        UtcDateTime::try_from(value)?.try_into()
    }
}

impl TryFrom<SystemTime> for Timestamp {
    type Error = UtcDateTimeError;

    fn try_from(system_time: SystemTime) -> Result<Self, Self::Error> {
        UtcDateTime::try_from(system_time).map(Into::into)
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Could panic if Timestamp is invalid
        match UtcDateTime::try_from(*self)
            .ok()
            .and_then(|dt| dt.format_rfc3339().ok())
        {
            Some(odt) => odt.fmt(f),
            None => "INVALID_TIMESTAMP".fmt(f),
        }
    }
}

impl FromStr for Timestamp {
    type Err = UtcDateTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UtcDateTime::parse_rfc3339(s)?.into())
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm",
))]
mod wasm {
    use super::*;

    impl WasmDescribe for Timestamp {
        fn describe() {
            <UtcDateTime as WasmDescribe>::describe()
        }
    }

    impl IntoWasmAbi for Timestamp {
        type Abi = <UtcDateTime as IntoWasmAbi>::Abi;

        fn into_abi(self) -> Self::Abi {
            UtcDateTime::try_from(self).unwrap().into_abi()
        }
    }

    impl OptionIntoWasmAbi for Timestamp {
        #[inline]
        fn none() -> Self::Abi {
            <UtcDateTime as OptionIntoWasmAbi>::none()
        }
    }

    impl FromWasmAbi for Timestamp {
        type Abi = <UtcDateTime as FromWasmAbi>::Abi;

        unsafe fn from_abi(js: Self::Abi) -> Self {
            UtcDateTime::from_abi(js).into()
        }
    }

    impl OptionFromWasmAbi for Timestamp {
        #[inline]
        fn is_none(js: &Self::Abi) -> bool {
            <UtcDateTime as OptionFromWasmAbi>::is_none(js)
        }
    }

    #[cfg_attr(feature = "js", wasm_bindgen(typescript_custom_section))]
    const TS_APPEND_CONTENT: &'static str = r#"
        export type Timestamp = Date;
    "#;

    #[cfg_attr(not(feature = "js"), wasm_bindgen(typescript_custom_section))]
    const TS_APPEND_CONTENT: &'static str = r#"
        export type Timestamp = string;
    "#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_convert() {
        let t = Timestamp::new(10, 2);
        let s = t.to_string();
        assert_eq!(s, "1970-01-01T00:00:10.000000002Z");
        let t2 = Timestamp::from_str(&s).unwrap();
        assert_eq!(t, t2);

        assert_eq!(
            Timestamp::from_str("2017-01-15T01:30:15.01Z").unwrap(),
            Timestamp::new(1_484_443_815, 10_000_000)
        );
    }

    #[test]
    fn normalize() {
        #[rustfmt::skip]
        let cases = [
            // --- Table of test cases ---
            //        test seconds      test nanos  expected seconds  expected nanos
            (line!(),            0,              0,                0,              0),
            (line!(),            1,              1,                1,              1),
            (line!(),           -1,             -1,               -2,    999_999_999),
            (line!(),            0,    999_999_999,                0,    999_999_999),
            (line!(),            0,   -999_999_999,               -1,              1),
            (line!(),            0,  1_000_000_000,                1,              0),
            (line!(),            0, -1_000_000_000,               -1,              0),
            (line!(),            0,  1_000_000_001,                1,              1),
            (line!(),            0, -1_000_000_001,               -2,    999_999_999),
            (line!(),           -1,              1,               -1,              1),
            (line!(),            1,             -1,                0,    999_999_999),
            (line!(),           -1,  1_000_000_000,                0,              0),
            (line!(),            1, -1_000_000_000,                0,              0),
            (line!(), i64::MIN    ,              0,     i64::MIN    ,              0),
            (line!(), i64::MIN + 1,              0,     i64::MIN + 1,              0),
            (line!(), i64::MIN    ,              1,     i64::MIN    ,              1),
            (line!(), i64::MIN    ,  1_000_000_000,     i64::MIN + 1,              0),
            (line!(), i64::MIN    , -1_000_000_000,     i64::MIN    ,              0),
            (line!(), i64::MIN + 1, -1_000_000_000,     i64::MIN    ,              0),
            (line!(), i64::MIN + 2, -1_000_000_000,     i64::MIN + 1,              0),
            (line!(), i64::MIN    , -1_999_999_998,     i64::MIN    ,              0),
            (line!(), i64::MIN + 1, -1_999_999_998,     i64::MIN    ,              0),
            (line!(), i64::MIN + 2, -1_999_999_998,     i64::MIN    ,              2),
            (line!(), i64::MIN    , -1_999_999_999,     i64::MIN    ,              0),
            (line!(), i64::MIN + 1, -1_999_999_999,     i64::MIN    ,              0),
            (line!(), i64::MIN + 2, -1_999_999_999,     i64::MIN    ,              1),
            (line!(), i64::MIN    , -2_000_000_000,     i64::MIN    ,              0),
            (line!(), i64::MIN + 1, -2_000_000_000,     i64::MIN    ,              0),
            (line!(), i64::MIN + 2, -2_000_000_000,     i64::MIN    ,              0),
            (line!(), i64::MIN    ,   -999_999_998,     i64::MIN    ,              0),
            (line!(), i64::MIN + 1,   -999_999_998,     i64::MIN    ,              2),
            (line!(), i64::MAX    ,              0,     i64::MAX    ,              0),
            (line!(), i64::MAX - 1,              0,     i64::MAX - 1,              0),
            (line!(), i64::MAX    ,             -1,     i64::MAX - 1,    999_999_999),
            (line!(), i64::MAX    ,  1_000_000_000,     i64::MAX    ,    999_999_999),
            (line!(), i64::MAX - 1,  1_000_000_000,     i64::MAX    ,              0),
            (line!(), i64::MAX - 2,  1_000_000_000,     i64::MAX - 1,              0),
            (line!(), i64::MAX    ,  1_999_999_998,     i64::MAX    ,    999_999_999),
            (line!(), i64::MAX - 1,  1_999_999_998,     i64::MAX    ,    999_999_998),
            (line!(), i64::MAX - 2,  1_999_999_998,     i64::MAX - 1,    999_999_998),
            (line!(), i64::MAX    ,  1_999_999_999,     i64::MAX    ,    999_999_999),
            (line!(), i64::MAX - 1,  1_999_999_999,     i64::MAX    ,    999_999_999),
            (line!(), i64::MAX - 2,  1_999_999_999,     i64::MAX - 1,    999_999_999),
            (line!(), i64::MAX    ,  2_000_000_000,     i64::MAX    ,    999_999_999),
            (line!(), i64::MAX - 1,  2_000_000_000,     i64::MAX    ,    999_999_999),
            (line!(), i64::MAX - 2,  2_000_000_000,     i64::MAX    ,              0),
            (line!(), i64::MAX    ,    999_999_998,     i64::MAX    ,    999_999_998),
            (line!(), i64::MAX - 1,    999_999_998,     i64::MAX - 1,    999_999_998),
        ];

        for case in &cases {
            assert_eq!(
                Timestamp::new(case.1, case.2).normalized(),
                Timestamp::new(case.3, case.4),
                "test case on line {} doesn't match",
                case.0,
            );
        }
    }
}
