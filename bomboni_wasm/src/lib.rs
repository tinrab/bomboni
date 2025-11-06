#![doc = include_str!("../README.md")]

use wasm_bindgen::prelude::*;

/// Console logging and debugging utilities.
pub mod macros;

const JSON_SERIALIZER: serde_wasm_bindgen::Serializer =
    serde_wasm_bindgen::Serializer::json_compatible();

/// Trait for converting between Rust types and JavaScript values.
///
/// This trait provides automatic serialization and deserialization
/// between Rust types and their JavaScript representations using
/// `serde-wasm-bindgen`.
pub trait Wasm {
    /// The JavaScript type this converts to.
    type JsType: JsCast;

    /// Converts this value to a JavaScript value.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    fn to_js(&self) -> Result<Self::JsType, serde_wasm_bindgen::Error>
    where
        Self: serde::Serialize,
    {
        self.serialize(&JSON_SERIALIZER)
            .map(JsCast::unchecked_from_js)
    }

    /// Converts a JavaScript value to this Rust type.
    ///
    /// # Errors
    /// Returns an error if deserialization fails.
    fn from_js<T: Into<JsValue>>(js: T) -> Result<Self, serde_wasm_bindgen::Error>
    where
        Self: serde::de::DeserializeOwned,
    {
        serde_wasm_bindgen::from_value(js.into())
    }
}

#[cfg(feature = "derive")]
pub use bomboni_wasm_derive::*;

#[cfg(feature = "derive")]
#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};

    mod bomboni {
        pub mod wasm {
            pub use crate::*;
        }
    }

    #[test]
    fn structs() {
        #[derive(Serialize, Deserialize, Wasm)]
        #[wasm(bomboni_wasm_crate = "crate")]
        pub struct Simple {
            a: String,
            b: i32,
            #[serde(default)]
            optional: Option<i32>,
        }

        assert_eq!(
            Simple::DECL,
            "export interface Simple {\n  a: string;\n  b: number;\n  optional?: number | null;\n}"
        );
    }

    #[test]
    fn renames() {
        #[derive(Serialize, Deserialize, Wasm)]
        #[wasm(bomboni_wasm_crate = "crate", rename_all = "camelCase")]
        struct Item {
            test_name: String,
            #[wasm(rename = "x")]
            value: i32,
        }

        assert_eq!(
            Item::DECL,
            "export interface Item {\n  testName: string;\n  x: number;\n}"
        );
    }

    #[test]
    fn enums() {
        #[derive(Serialize, Deserialize, Wasm)]
        #[wasm(bomboni_wasm_crate = "crate")]
        pub enum ExternalTag {
            String(String),
            Number(f64),
        }

        #[derive(Serialize, Deserialize, Wasm)]
        #[wasm(bomboni_wasm_crate = "crate")]
        #[serde(tag = "kind", content = "data")]
        pub enum AdjacentTag {
            String(String),
            Number(f64),
        }

        #[derive(Serialize, Deserialize, Wasm)]
        #[wasm(bomboni_wasm_crate = "crate")]
        #[serde(tag = "kind")]
        pub enum InternalTag {
            String { value: String },
            Item(InternalItem),
        }
        #[derive(Serialize, Deserialize)]
        #[serde(tag = "type")]
        struct InternalItem {
            value: i32,
        }

        assert_eq!(
            ExternalTag::DECL,
            "export type ExternalTag = {\n  String: string;\n  Number?: null;\n} | {\n  Number: number;\n  String?: null;\n};"
        );
        assert_eq!(
            AdjacentTag::DECL,
            "export type AdjacentTag = {\n  kind: \"String\";\n  data: string;\n} | {\n  kind: \"Number\";\n  data: number;\n};"
        );

        assert_eq!(
            InternalTag::DECL,
            "export type InternalTag = {\n  kind: \"String\";\n  value: string;\n} | ({\n  kind: \"Item\";\n} & InternalItem);"
        );
    }
}
