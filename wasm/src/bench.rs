use bomboni_common::date_time::UtcDateTime;
use bomboni_common::id::Id;
use bomboni_proto::google::protobuf::Timestamp;
use bomboni_request::derive::Parse;
use bomboni_request::error::CommonError;
use bomboni_request::error::RequestError;
use bomboni_request::error::RequestResult;
use bomboni_request::parse::RequestParse;
use bomboni_request::parse::RequestParseInto;
use bomboni_wasm::Wasm;
use time::OffsetDateTime;
use ts_rs::TS;
use tsify::Tsify;
use wasm_bindgen::JsObject;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::describe::WasmDescribe;
use wasm_bindgen::{convert::IntoWasmAbi, prelude::*};

#[derive(Debug, Clone, Default)]
// #[wasm_bindgen(getter_with_clone, inspectable, skip_typescript)]
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
// #[wasm(wasm_abi)]
#[wasm_bindgen(getter_with_clone)]
pub struct BenchBindgen {
    pub name: String,
    pub age: i32,
    pub values: Vec<i32>,
}

#[wasm_bindgen]
impl BenchBindgen {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen]
pub fn compute_bench_bindgen(bench: Vec<BenchBindgen>) -> i32 {
    bench.iter().map(|b| b.age).sum()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Tsify)]
// #[wasm_bindgen(getter_with_clone, inspectable, skip_typescript)]
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
// #[wasm(wasm_abi)]
#[tsify(into_wasm_abi, from_wasm_abi)]
// #[wasm_bindgen(getter_with_clone)]
pub struct BenchSerde {
    pub name: String,
    pub age: i32,
    pub values: Vec<i32>,
}

#[wasm_bindgen]
impl BenchSerde {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
}

// impl JsObject for BenchSerde {}

// impl JsCast for BenchSerde {
//     fn instanceof(val: &JsValue) -> bool {
//         val.is_instance_of::<BenchSerde>()
//     }
// }

// impl From<BenchSerde> for JsValue {
//     fn from(bench: BenchSerde) -> Self {
//         JsValue::from_serde(&bench).unwrap_throw()
//     }
// }

// #[wasm_bindgen]
// pub fn compute_bench_serde(bench: Vec<BenchSerde>) -> i32 {
//     bench.iter().map(|b| b.age).sum()
// }
