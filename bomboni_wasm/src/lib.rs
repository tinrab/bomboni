use wasm_bindgen::prelude::*;

pub mod macros;
pub mod utility;

pub trait Wasm {
    type JsType: JsCast;

    const DECL: &'static str;

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
        }

        assert_eq!(
            Simple::DECL,
            "export interface Simple {\n  a: string;\n  b: number;\n}"
        );
    }

    #[test]
    fn enums() {
        #[derive(Serialize, Deserialize, WasmTypescript)]
        pub enum Simple {
            A,
            B,
        }
        // #[derive(Serialize, Deserialize, WasmTypescript)]
        // #[repr(i32)]
        // pub enum CStyle {
        //     A = 1,
        //     B = 2,
        // }
        // #[derive(Serialize, Deserialize, WasmTypescript)]
        // pub enum UnionKind {
        //     String(String),
        //     Number(f64),
        // }

        assert_eq!(Simple::DECL, r#"export type Simple = "A" | "B";"#);
    }
}
