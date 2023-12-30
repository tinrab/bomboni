//! # Id
//!
//! Semi-globally unique and sortable identifiers.
use std::{
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(feature = "serde")]
use serde::{de::Unexpected, Deserialize, Deserializer, Serialize, Serializer};

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
use wasm_bindgen::prelude::*;
#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
use wasm_bindgen::{
    convert::{FromWasmAbi, IntoWasmAbi},
    describe::WasmDescribe,
};

pub mod generator;
#[cfg(feature = "mysql")]
mod mysql;

const TIMESTAMP_BITS: i64 = 64;
const WORKER_BITS: i64 = 16;
const SEQUENCE_BITS: i64 = 16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
// #[cfg_attr(
//     all(
//         target_family = "wasm",
//         not(any(target_os = "emscripten", target_os = "wasi")),
//         feature = "wasm"
//     ),
//     // derive(Wasm),
//     // wasm(as_string)
//     wasm_bindgen(inspectable),
// )]
pub struct Id(u128);

impl Id {
    #[must_use]
    pub const fn new(id: u128) -> Self {
        Self(id)
    }

    /// Encodes the Id from parts.
    #[must_use]
    pub fn from_parts(time: SystemTime, worker: u16, sequence: u16) -> Self {
        let timestamp = u128::from(time.duration_since(UNIX_EPOCH).unwrap().as_secs());
        let worker = u128::from(worker);
        let sequence = u128::from(sequence);

        assert!(timestamp < (1 << TIMESTAMP_BITS));
        assert!(worker < (1 << WORKER_BITS));
        assert!(sequence < (1 << SEQUENCE_BITS));

        Self(
            (timestamp & ((1 << TIMESTAMP_BITS) - 1)) << (WORKER_BITS + SEQUENCE_BITS)
                | ((worker & ((1 << WORKER_BITS) - 1)) << SEQUENCE_BITS)
                | (sequence & ((1 << SEQUENCE_BITS) - 1)),
        )
    }

    /// Decodes Id's parts.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use std::time::{SystemTime, Duration};
    /// use bomboni_common::id::Id;
    ///
    /// let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1337);
    /// let id = Id::from_parts(time, 42, 1);
    /// let (timestamp, worker, sequence) = id.decode();
    /// assert_eq!(timestamp, time);
    /// assert_eq!(worker, 42);
    /// assert_eq!(sequence, 1);
    /// ```
    #[must_use]
    pub fn decode(self) -> (SystemTime, u16, u16) {
        let timestamp = SystemTime::UNIX_EPOCH
            + Duration::from_secs((self.0 >> (WORKER_BITS + SEQUENCE_BITS)) as u64);
        let worker = ((self.0 >> SEQUENCE_BITS) & ((1 << WORKER_BITS) - 1)) as u16;
        let sequence = (self.0 & ((1 << SEQUENCE_BITS) - 1)) as u16;
        (timestamp, worker, sequence)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl FromStr for Id {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = u128::from_str_radix(s, 16)?;
        Ok(Self::new(value))
    }
}

macro_rules! impl_from {
    ( $( $source:ty ),* $(,)? ) => {
        $(impl From<$source> for Id {
            fn from(x: $source) -> Self {
                Id::new(x as u128)
            }
        })*
    };
}
impl_from!(i8, i16, i32, i64, i128, u8, u16, u32, u64);

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

// #[cfg_attr(
//     all(
//         target_family = "wasm",
//         not(any(target_os = "emscripten", target_os = "wasi")),
//         feature = "wasm"
//     ),
//     wasm_bindgen
// )]
// impl Id {
//     #[wasm_bindgen(constructor)]
//     pub fn new_wasm(value: String) -> Self {
//         Self::from_str(&value).unwrap()
//     }

//     #[wasm_bindgen(getter)]
//     pub fn value(&self) -> String {
//         self.to_string()
//     }
// }

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
impl From<Id> for js_sys::JsString {
    fn from(value: Id) -> Self {
        value.to_string().into()
    }
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
impl From<js_sys::JsString> for Id {
    fn from(value: js_sys::JsString) -> Self {
        Self::from_str(&value.as_string().unwrap()).unwrap()
    }
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
impl From<&js_sys::JsString> for Id {
    fn from(value: &js_sys::JsString) -> Self {
        Self::from_str(&value.as_string().unwrap()).unwrap()
    }
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
impl WasmDescribe for Id {
    fn describe() {
        <js_sys::JsString as WasmDescribe>::describe()
    }
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
impl IntoWasmAbi for Id {
    type Abi = <js_sys::JsString as IntoWasmAbi>::Abi;

    fn into_abi(self) -> Self::Abi {
        js_sys::JsString::from(self.to_string()).into_abi()
    }
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
impl FromWasmAbi for Id {
    type Abi = <js_sys::JsString as FromWasmAbi>::Abi;

    unsafe fn from_abi(js_string: Self::Abi) -> Self {
        Self::from_str(
            &js_sys::JsString::from_abi(js_string)
                .as_string()
                .as_ref()
                .unwrap(),
        )
        .unwrap()
    }
}

// #[cfg(all(
//     target_family = "wasm",
//     not(any(target_os = "emscripten", target_os = "wasi")),
//     feature = "wasm"
// ))]
// #[wasm_bindgen(typescript_custom_section)]
// const TS_APPEND_CONTENT: &'static str = r#"
//     export type Id = string;
// "#;

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(
            Id::from_parts(SystemTime::UNIX_EPOCH + Duration::from_secs(10), 1, 1),
            Id(0b1010_0000_0000_0000_0001_0000_0000_0000_0001)
        );
        let max_time = SystemTime::UNIX_EPOCH + Duration::from_secs(Duration::MAX.as_secs() / 2);
        let id = Id::from_parts(max_time, 1, 1);
        assert_eq!(
            id,
            Id(0b111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_0000_0000_0000_0001_0000_0000_0000_0001)
        );
        let (timestamp, worker, sequence) = id.decode();
        assert_eq!(timestamp, max_time);
        assert_eq!(worker, 1);
        assert_eq!(sequence, 1);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize() {
        let id = Id::from_parts(SystemTime::UNIX_EPOCH + Duration::from_secs(2 << 20), 1, 1);
        assert_eq!(serde_json::to_string(&id).unwrap(), r#""20000000010001""#);
    }
}
