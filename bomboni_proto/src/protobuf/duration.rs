use crate::serde::helpers as serde_helpers;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use chrono::Duration as ChronoDuration;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
    time::Duration as StdDuration,
};
use thiserror::Error;

use crate::google::protobuf::Duration;

#[derive(Error, Debug)]
pub enum DurationError {
    #[error("duration is out of range")]
    OutOfRange,
    #[error("duration is negative")]
    NegativeDuration,
    #[error("invalid duration string format `{0}`")]
    InvalidFormat(String),
}

// pub type ProtoResult<T> = Result<T, ProtoError>;

impl Duration {
    pub fn new(seconds: i64, nanos: i32) -> Self {
        Self { seconds, nanos }
    }

    /// Get Duration normalized to a canonical format.
    /// Based on [1] and [2].
    ///
    /// [1]: https://github.com/tokio-rs/prost/blob/v0.12.1/prost-types/src/lib.rs#L107
    /// [2]: https://github.com/protocolbuffers/protobuf/blob/v3.3.2/src/google/protobuf/util/time_util.cc#L79-L100
    pub fn normalized(self) -> Self {
        const NANOS_PER_SECOND: i64 = 1_000_000_000;
        const NANOS_MAX: i64 = NANOS_PER_SECOND - 1;
        // const DURATION_MIN_SECONDS: i64 = -315_576_000_000;
        // const DURATION_MAX_SECONDS: i64 = 315_576_000_000;

        let mut seconds = self.seconds;
        let mut nanos = self.nanos as i64;

        // Make sure nanos is in the range.
        if nanos <= -NANOS_PER_SECOND || nanos >= NANOS_PER_SECOND {
            if let Some(new_seconds) = seconds.checked_add(nanos / NANOS_PER_SECOND) {
                seconds = new_seconds;
                nanos %= NANOS_PER_SECOND;
            } else if nanos < 0 {
                seconds = i64::MIN;
                nanos = -NANOS_MAX;
            } else {
                seconds = i64::MAX;
                nanos = NANOS_MAX;
            }
        }

        // Nanos should have the same sign as seconds.
        if seconds < 0 && nanos > 0 {
            if let Some(new_seconds) = seconds.checked_add(1) {
                seconds = new_seconds;
                nanos -= NANOS_PER_SECOND;
            } else {
                nanos = NANOS_MAX;
            }
        } else if seconds > 0 && nanos < 0 {
            if let Some(new_seconds) = seconds.checked_sub(1) {
                seconds = new_seconds;
                nanos += NANOS_PER_SECOND;
            } else {
                nanos = -NANOS_MAX;
            }
        }

        // debug_assert!(
        //     seconds >= DURATION_MIN_SECONDS && seconds <= DURATION_MAX_SECONDS,
        //     "seconds out of range: {}",
        //     seconds
        // );

        Self {
            seconds,
            nanos: nanos as i32,
        }
    }
}

impl TryFrom<StdDuration> for Duration {
    type Error = DurationError;

    fn try_from(duration: StdDuration) -> Result<Duration, DurationError> {
        let seconds = i64::try_from(duration.as_secs()).map_err(|_| DurationError::OutOfRange)?;
        let nanos = duration.subsec_nanos() as i32;
        Ok(Duration { seconds, nanos }.normalized())
    }
}

impl TryFrom<Duration> for StdDuration {
    type Error = DurationError;

    fn try_from(duration: Duration) -> Result<StdDuration, DurationError> {
        let d = duration.normalized();
        if d.seconds >= 0 && d.nanos >= 0 {
            Ok(StdDuration::new(d.seconds as u64, d.nanos as u32))
        } else {
            Err(DurationError::NegativeDuration)
        }
    }
}

impl TryFrom<ChronoDuration> for Duration {
    type Error = DurationError;

    fn try_from(duration: ChronoDuration) -> Result<Duration, DurationError> {
        duration
            .to_std()
            .map_err(|_| DurationError::OutOfRange)?
            .try_into()
    }
}

impl TryFrom<Duration> for ChronoDuration {
    type Error = DurationError;

    fn try_from(duration: Duration) -> Result<ChronoDuration, DurationError> {
        ChronoDuration::from_std(StdDuration::try_from(duration)?)
            .map_err(|_| DurationError::OutOfRange)
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.nanos == 0 {
            write!(f, "{}s", self.seconds)
        } else {
            let fractional_seconds = (self.nanos as f64 / 1_000_000_000.0).to_string();
            write!(
                f,
                "{}.{}s",
                self.seconds,
                fractional_seconds.trim_start_matches("0.")
            )
        }
    }
}

impl FromStr for Duration {
    type Err = DurationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.ends_with('s') {
            return Err(DurationError::InvalidFormat(s.into()));
        }

        let v = &s[..s.len() - 1]; // Remove 's'
        let parts: Vec<&str> = v.split('.').collect();

        if parts.len() > 2 {
            return Err(DurationError::InvalidFormat(s.into()));
        }

        let seconds = parts[0]
            .parse::<i64>()
            .map_err(|_| DurationError::InvalidFormat(s.into()))?;
        let nanos = if parts.len() > 1 {
            (format!("0.{}", parts[1])
                .parse::<f64>()
                .map_err(|_| DurationError::InvalidFormat(s.into()))?
                * 1_000_000_000f64) as i32
        } else {
            0
        };

        Ok(Duration::new(seconds, nanos))
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serde_helpers::duration::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        serde_helpers::duration::deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_convert() {
        let d = Duration::new(10, 2);
        let s = d.to_string();
        assert_eq!(s, "10.000000002s");
        let d2 = s.parse::<Duration>().unwrap();
        assert_eq!(d, d2);
    }

    #[test]
    fn normalize() {
        #[rustfmt::skip]
        let cases = [
            // --- Table of test cases ---
            //        test seconds      test nanos  expected seconds  expected nanos
            (line!(),            0,              0,                0,              0),
            (line!(),            1,              1,                1,              1),
            (line!(),           -1,             -1,               -1,             -1),
            (line!(),            0,    999_999_999,                0,    999_999_999),
            (line!(),            0,   -999_999_999,                0,   -999_999_999),
            (line!(),            0,  1_000_000_000,                1,              0),
            (line!(),            0, -1_000_000_000,               -1,              0),
            (line!(),            0,  1_000_000_001,                1,              1),
            (line!(),            0, -1_000_000_001,               -1,             -1),
            (line!(),           -1,              1,                0,   -999_999_999),
            (line!(),            1,             -1,                0,    999_999_999),
            (line!(),           -1,  1_000_000_000,                0,              0),
            (line!(),            1, -1_000_000_000,                0,              0),
            (line!(), i64::MIN    ,              0,     i64::MIN    ,              0),
            (line!(), i64::MIN + 1,              0,     i64::MIN + 1,              0),
            (line!(), i64::MIN    ,              1,     i64::MIN + 1,   -999_999_999),
            (line!(), i64::MIN    ,  1_000_000_000,     i64::MIN + 1,              0),
            (line!(), i64::MIN    , -1_000_000_000,     i64::MIN    ,   -999_999_999),
            (line!(), i64::MIN + 1, -1_000_000_000,     i64::MIN    ,              0),
            (line!(), i64::MIN + 2, -1_000_000_000,     i64::MIN + 1,              0),
            (line!(), i64::MIN    , -1_999_999_998,     i64::MIN    ,   -999_999_999),
            (line!(), i64::MIN + 1, -1_999_999_998,     i64::MIN    ,   -999_999_998),
            (line!(), i64::MIN + 2, -1_999_999_998,     i64::MIN + 1,   -999_999_998),
            (line!(), i64::MIN    , -1_999_999_999,     i64::MIN    ,   -999_999_999),
            (line!(), i64::MIN + 1, -1_999_999_999,     i64::MIN    ,   -999_999_999),
            (line!(), i64::MIN + 2, -1_999_999_999,     i64::MIN + 1,   -999_999_999),
            (line!(), i64::MIN    , -2_000_000_000,     i64::MIN    ,   -999_999_999),
            (line!(), i64::MIN + 1, -2_000_000_000,     i64::MIN    ,   -999_999_999),
            (line!(), i64::MIN + 2, -2_000_000_000,     i64::MIN    ,              0),
            (line!(), i64::MIN    ,   -999_999_998,     i64::MIN    ,   -999_999_998),
            (line!(), i64::MIN + 1,   -999_999_998,     i64::MIN + 1,   -999_999_998),
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
                Duration {
                    seconds: case.1,
                    nanos: case.2,
                }
                .normalized(),
                Duration {
                    seconds: case.3,
                    nanos: case.4,
                },
                "test case on line {} doesn't match",
                case.0,
            );
        }

        dbg!(StdDuration::new(0, u32::MAX));
    }
}
