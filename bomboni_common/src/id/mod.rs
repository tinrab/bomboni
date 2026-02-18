//! # Id
//!
//! Semi-globally unique and sortable identifiers.

use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};
use thiserror::Error;
use time::{Duration, OffsetDateTime};
use ulid::Ulid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Unexpected};

#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;

/// Worker ID generation utilities.
pub mod worker;

/// A semi-globally unique and sortable identifier.
///
/// This is based on ULID (Universally Unique Lexicographically Sortable Identifier)
/// and provides both uniqueness and sortability.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    all(
        target_family = "wasm",
        not(any(target_os = "emscripten", target_os = "wasi")),
        feature = "wasm"
    ),
    derive(bomboni_wasm::Wasm),
    wasm(
        bomboni_wasm_crate = bomboni_wasm,
        wasm_abi,
        js_value { convert_string },
    )
)]
pub struct Id(u128);

/// Errors that can occur when parsing an `Id` from a string.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParseIdError {
    /// The string is not a valid ID format.
    #[error("invalid id string")]
    InvalidString,
}

const TIMESTAMP_BITS: i64 = 64;
const WORKER_BITS: i64 = 16;
const SEQUENCE_BITS: i64 = 16;

impl Id {
    /// Creates a new ID from a 128-bit integer.
    #[must_use]
    pub const fn new(id: u128) -> Self {
        Self(id)
    }

    /// Generates a new random sortable ID.
    #[must_use]
    pub fn generate() -> Self {
        Self(Ulid::new().0)
    }

    /// Generate multiple random sortable IDs.
    ///
    /// Generated IDs are monotonically increasing.
    ///
    /// # Panics
    ///
    /// Panics if the ULID generator fails to generate an ID.
    #[must_use]
    pub fn generate_multiple(count: usize) -> Vec<Self> {
        let mut ids = Vec::with_capacity(count);
        let mut g = ulid::Generator::new();
        for _ in 0..count {
            ids.push(Self::new(g.generate().unwrap().0));
        }
        ids
    }

    /// Creates an ID from worker parts.
    ///
    /// # Panics
    ///
    /// Panics if the timestamp, worker ID, or sequence number is too large.
    #[must_use]
    pub fn from_worker_parts(time: OffsetDateTime, worker: u16, sequence: u16) -> Self {
        let timestamp_ms = time.unix_timestamp_nanos() as u128 / 1_000_000;
        let worker = u128::from(worker);
        let sequence = u128::from(sequence);

        assert!(timestamp_ms < (1 << TIMESTAMP_BITS));
        assert!(worker < (1 << WORKER_BITS));
        assert!(sequence < (1 << SEQUENCE_BITS));

        Self(
            (timestamp_ms & ((1 << TIMESTAMP_BITS) - 1)) << (WORKER_BITS + SEQUENCE_BITS)
                | ((worker & ((1 << WORKER_BITS) - 1)) << SEQUENCE_BITS)
                | (sequence & ((1 << SEQUENCE_BITS) - 1)),
        )
    }

    /// Creates an ID from time and a random number.
    #[must_use]
    pub const fn from_time_and_random(time: OffsetDateTime, random: u128) -> Self {
        let timestamp_ms = time.unix_timestamp_nanos() / 1_000_000;
        let id = Ulid::from_parts(timestamp_ms as u64, random);
        Self::new(id.0)
    }

    /// Decodes the ID into worker parts.
    #[must_use]
    pub fn decode_worker(self) -> (OffsetDateTime, u16, u16) {
        let milliseconds = (self.0 >> (WORKER_BITS + SEQUENCE_BITS)) as i64;
        let timestamp = OffsetDateTime::UNIX_EPOCH + Duration::milliseconds(milliseconds);

        let worker = ((self.0 >> SEQUENCE_BITS) & ((1 << WORKER_BITS) - 1)) as u16;
        let sequence = (self.0 & ((1 << SEQUENCE_BITS) - 1)) as u16;

        (timestamp, worker, sequence)
    }

    /// Decodes the ID into time and randomness parts.
    #[must_use]
    pub fn decode_time_and_random(self) -> (OffsetDateTime, u128) {
        let id = Ulid::from(self.0);

        let milliseconds = id.timestamp_ms() as i64;
        let timestamp = OffsetDateTime::UNIX_EPOCH + Duration::milliseconds(milliseconds);

        let random = id.random();

        (timestamp, random)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Ulid(self.0).fmt(f)
    }
}

impl FromStr for Id {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ulid::from_str(s)
            .map(From::from)
            .map_err(|_| ParseIdError::InvalidString)
    }
}

macro_rules! impl_from {
    ( $( $source:ty ),* $(,)? ) => {
        $(impl From<$source> for Id {
            fn from(x: $source) -> Self {
                Id::new(x.into())
            }
        })*
    };
}
impl_from!(u8, u16, u32, u64);

macro_rules! impl_as {
    ( $( $source:ty ),* $(,)? ) => {
        $(impl From<$source> for Id {
            fn from(x: $source) -> Self {
                Id::new(x as u128)
            }
        })*
    };
}
impl_as!(i8, i16, i32, i64, i128);

impl From<Id> for u128 {
    fn from(id: Id) -> Self {
        id.0
    }
}

impl From<Ulid> for Id {
    fn from(ulid: Ulid) -> Self {
        Self(ulid.into())
    }
}

impl From<Id> for Ulid {
    fn from(id: Id) -> Self {
        Self::from(id.0)
    }
}

#[cfg(feature = "serde")]
impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let value = String::deserialize(deserializer)?;
        value.parse::<Self>().map_err(|_| {
            <D as Deserializer<'de>>::Error::invalid_value(Unexpected::Str(value.as_str()), &"Id")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_random() {
        use std::collections::HashMap;
        const N: usize = 10;

        let mut ids = HashMap::new();
        for _ in 0..N {
            let id = Id::generate();
            ids.insert(id.to_string(), id);
        }
        assert_eq!(ids.len(), N);

        ids = Id::generate_multiple(N)
            .into_iter()
            .map(|id| (id.to_string(), id))
            .collect();
        assert_eq!(ids.len(), N);

        for (id_str, id) in ids {
            let decoded: Id = id_str.parse().unwrap();
            assert_eq!(decoded, id);
        }
    }

    #[test]
    fn worker_parts() {
        let ts = OffsetDateTime::from_unix_timestamp(42).unwrap() + Duration::milliseconds(1337);

        let id = Id::from_worker_parts(ts, 1, 1);
        assert_eq!(
            id,
            Id(0b1010_1001_0100_1001_0000_0000_0000_0001_0000_0000_0000_0001)
        );
        let (timestamp, worker, sequence) = id.decode_worker();
        assert_eq!(timestamp, ts);
        assert_eq!(worker, 1);
        assert_eq!(sequence, 1);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize() {
        let id = Id::from_worker_parts(OffsetDateTime::from_unix_timestamp(42).unwrap(), 5, 7);
        assert_eq!(
            serde_json::to_string(&id).unwrap(),
            r#""0000000000000000542000A007""#
        );
    }
}
