use bomboni_common::date_time::UtcDateTime;
use bomboni_common::id::Id;
use bomboni_proto::google::protobuf::Timestamp;
use bomboni_request::derive::Parse;
use bomboni_request::error::CommonError;
use bomboni_request::error::RequestError;
use bomboni_request::error::RequestResult;
pub use bomboni_request::parse::ParsedResource;
use bomboni_request::parse::RequestParse;
use bomboni_request::parse::RequestParseInto;
use bomboni_wasm::console_log;
use bomboni_wasm::Wasm;
use chrono::NaiveDateTime;
use time::OffsetDateTime;
use ts_rs::TS;
use tsify::Tsify;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::describe::WasmDescribe;
use wasm_bindgen::{convert::IntoWasmAbi, prelude::*};

// #[derive(Clone, Serialize, Deserialize, Wasm)]
// #[derive(Debug, Clone, Wasm)]
// #[ts(export, export_to = "bindings/Item.ts")]
#[derive(Debug, Clone)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct Item {
    pub id: Id,
    pub create_time: UtcDateTime,
    pub date_time: NaiveDateTime,
}

#[wasm_bindgen]
impl Item {
    #[wasm_bindgen(constructor)]
    pub fn new(id: Id, create_time: Option<UtcDateTime>) -> Self {
        Self {
            id,
            create_time: create_time.unwrap_or_else(|| UtcDateTime::now()),
        }
    }
}

#[wasm_bindgen]
pub fn compute(item: &Item) {
    console_log!("compute: {:?}", item);
}

// #[derive(Debug)]
// // #[serde(transparent)]
// #[wasm_bindgen]
// pub struct AccountId(pub Id);

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
// #[wasm_bindgen(getter_with_clone, inspectable)]
// // #[serde(rename = "IParsedAccount")]
// pub struct ParsedAccount {
//     pub id: AccountId,
//     pub project_id: Id,
//     pub created_at: UtcDateTime,
// }

// #[derive(Debug, Clone, PartialEq)]
// // #[wasm(proxy = String)]
// pub struct Id(u128);

// impl Id {
//     #[must_use]
//     pub const fn new(id: u128) -> Self {
//         Self(id)
//     }
// }

// const ITEXT_STYLE: &'static str = r#"
// interface ITextStyle {
//     bold: boolean;
//     italic: boolean;
//     size: number;
// }
// "#;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(typescript_type = "ParsedAccount")]
//     pub type IParsedAccount;
// }

// // #[wasm_bindgen(inline_js = r"
// // export class PyError extends Error {
// //     constructor(value) {
// //         this.info = info;
// //     }
// // }
// // ")]
// extern "C" {
//     // pub type ParsedAccount;
//     #[wasm_bindgen(constructor)]
//     fn new(value: JsValue) -> ParsedAccount;
// }

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(typescript_type = "ParsedAccount")]
//     pub type IParsedAccount;
// }

// #[wasm_bindgen]
// impl ParsedAccount {
//     #[wasm_bindgen(constructor)]
//     pub fn new(value: IParsedAccount) -> Result<ParsedAccount, JsValue> {
//         match ParsedAccount::from_js(value.obj) {
//             Ok(value) => Ok(value),
//             Err(err) => Err(err.into()),
//         }
//     }
// }

// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
// struct Item {
//     name: String,
//     create_time: Option<Timestamp>,
//     value: i32,
// }

// #[derive(Debug, Clone, PartialEq, Parse, Serialize, Deserialize, Wasm)]
// #[parse(source = Item, write)]
// // #[wasm_bindgen(getter_with_clone, inspectable, skip_typescript)]
// // #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
// // #[wasm(wasm_abi)]
// #[wasm_bindgen(getter_with_clone)]
// pub struct ParsedItem {
//     // #[parse(resource {
//     //     fields = [name, create_time],
//     // })]
//     // pub resource: ParsedResource,
//     pub name: String,
//     pub value: i32,
//     // pub create_time: Option<Timestamp>,
// }

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(typescript_type = "ParsedItem2")]
//     pub type IParsedItem;
// }

// #[wasm_bindgen]
// impl ParsedItem {
//     // #[wasm_bindgen(constructor)]
//     // pub fn new(name: String) -> Self {
//     //     Self {
//     //         name,
//     //         value: 0,
//     //         // create_time: None,
//     //     }
//     // }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
