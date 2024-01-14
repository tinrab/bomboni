use wasm_bindgen::prelude::*;

pub mod macros;
pub mod utility;

pub trait Wasm {
    type JsType: JsCast;

    // const DECL: &'static str;

    fn to_js(&self) -> Result<Self::JsType, serde_wasm_bindgen::Error>
    where
        Self: serde::Serialize,
    {
        serde_wasm_bindgen::to_value(self).map(JsCast::unchecked_from_js)
    }

    fn from_js<T: Into<JsValue>>(js: T) -> Result<Self, serde_wasm_bindgen::Error>
    where
        Self: serde::de::DeserializeOwned,
    {
        serde_wasm_bindgen::from_value(js.into())
    }
}

#[cfg(feature = "derive")]
pub use bomboni_wasm_derive::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn structs() {
        #[derive(Serialize, Deserialize, Wasm)]
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
        #[wasm(rename_all = "camelCase")]
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
        pub enum ExternalTag {
            String(String),
            Number(f64),
        }

        #[derive(Serialize, Deserialize, Wasm)]
        #[serde(tag = "kind", content = "data")]
        pub enum AdjacentTag {
            String(String),
            Number(f64),
        }

        #[derive(Serialize, Deserialize, Wasm)]
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

        // #[derive(Serialize, Deserialize, Wasm)]
        // #[repr(i32)]
        // pub enum CStyle {
        //     A = 1,
        //     B = 2,
        // }

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

        // println!("{}", serde_json::to_string_pretty(&CStyle::A).unwrap());
    }
}
