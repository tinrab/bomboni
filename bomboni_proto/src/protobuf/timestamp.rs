use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::google::protobuf::Timestamp;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TimestampError {
    #[error("invalid nanoseconds")]
    InvalidNanoseconds,
    #[error("not a UTC date time")]
    NotUtc,
    #[error("invalid timestamp string format `{0}`")]
    InvalidFormat(String),
}

const NANOS_PER_SECOND: i32 = 1_000_000_000;

impl Timestamp {
    pub const fn new(seconds: i64, nanos: i32) -> Self {
        Timestamp { seconds, nanos }
    }

    /// Get Timestamp normalized to a canonical format.
    /// Based on [1] and [2].
    ///
    /// [1]: https://github.com/tokio-rs/prost/blob/v0.12.1/prost-types/src/lib.rs#L274
    /// [2]: https://github.com/google/protobuf/blob/v3.3.2/src/google/protobuf/util/time_util.cc#L59-L77
    pub fn normalized(self) -> Self {
        // const TIMESTAMP_MIN_SECONDS: i64 = -62_135_596_800;
        // const TIMESTAMP_MAX_SECONDS: i64 = 253_402_300_799;

        let mut seconds = self.seconds;
        let mut nanos = self.nanos;

        // Make sure nanos is in the range.
        if nanos <= -NANOS_PER_SECOND || nanos >= NANOS_PER_SECOND {
            if let Some(new_seconds) = seconds.checked_add(nanos as i64 / NANOS_PER_SECOND as i64) {
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

        Timestamp { seconds, nanos }
    }

    // pub fn is_normalized(self) -> bool {
    //     let n = self.normalized();
    //     n.seconds != self.seconds && (n.seconds == i64::MIN || n.seconds == i64::MAX)
    // }
}

impl From<OffsetDateTime> for Timestamp {
    fn from(t: OffsetDateTime) -> Self {
        let seconds = t.unix_timestamp();
        let nanos = t.nanosecond();
        Timestamp {
            seconds,
            nanos: nanos as i32,
        }
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDateTime> for Timestamp {
    fn from(t: chrono::NaiveDateTime) -> Self {
        Timestamp {
            seconds: t.timestamp(),
            nanos: t.timestamp_subsec_nanos() as i32,
        }
    }
}

impl TryFrom<Timestamp> for OffsetDateTime {
    type Error = TimestampError;

    fn try_from(t: Timestamp) -> Result<Self, Self::Error> {
        if t.nanos < 0 {
            return Err(TimestampError::InvalidNanoseconds);
        }
        // NaiveDateTime::from_timestamp_opt(t.seconds, t.nanos as u32)
        OffsetDateTime::from_unix_timestamp_nanos(
            t.seconds as i128 * NANOS_PER_SECOND as i128 + t.nanos as i128,
        )
        .map_err(|_| TimestampError::NotUtc)
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<Timestamp> for chrono::NaiveDateTime {
    type Error = TimestampError;

    fn try_from(t: Timestamp) -> Result<Self, Self::Error> {
        if t.nanos < 0 {
            return Err(TimestampError::InvalidNanoseconds);
        }
        chrono::NaiveDateTime::from_timestamp_opt(t.seconds, t.nanos as u32)
            .ok_or(TimestampError::NotUtc)
    }
}

impl From<SystemTime> for Timestamp {
    fn from(system_time: SystemTime) -> Timestamp {
        let (seconds, nanos) = match system_time.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let seconds = i64::try_from(duration.as_secs()).unwrap();
                (seconds, duration.subsec_nanos() as i32)
            }
            Err(error) => {
                let duration = error.duration();
                let seconds = i64::try_from(duration.as_secs()).unwrap();
                let nanos = duration.subsec_nanos() as i32;
                if nanos == 0 {
                    (-seconds, 0)
                } else {
                    (-seconds - 1, NANOS_PER_SECOND - nanos)
                }
            }
        };
        Timestamp { seconds, nanos }
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Could panic if Timestamp is invalid
        match OffsetDateTime::try_from(*self)
            .ok()
            .and_then(|odt| odt.format(&Rfc3339).ok())
        {
            Some(odt) => odt.fmt(f),
            None => "INVALID_TIMESTAMP".fmt(f),
        }
    }
}

impl FromStr for Timestamp {
    type Err = TimestampError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(OffsetDateTime::parse(s, &Rfc3339)
            .map_err(|_| TimestampError::InvalidFormat(s.into()))?
            .into())
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
        Timestamp::from_str(&s).map_err(de::Error::custom)
    }
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

        for case in cases.iter() {
            assert_eq!(
                Timestamp::new(case.1, case.2).normalized(),
                Timestamp::new(case.3, case.4),
                "test case on line {} doesn't match",
                case.0,
            );
        }
    }
}
