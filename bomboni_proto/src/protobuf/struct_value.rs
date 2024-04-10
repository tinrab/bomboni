use crate::google::protobuf::{
    value::Kind as ValueKind, Empty, ListValue, NullValue, Struct, Value,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Number as JsonNumber, Value as JsonValue};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    fmt::Display,
};

impl Struct {
    #[must_use]
    pub fn new(fields: BTreeMap<String, Value>) -> Self {
        Self { fields }
    }
}

impl From<JsonValue> for Struct {
    fn from(value: JsonValue) -> Self {
        match value {
            JsonValue::Object(o) => Self::new(o.into_iter().map(|(k, v)| (k, v.into())).collect()),
            _ => panic!("JsonValue::Object is expected"),
        }
    }
}

impl From<Struct> for JsonValue {
    fn from(value: Struct) -> Self {
        JsonValue::Object(
            value
                .fields
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        )
    }
}

impl From<BTreeMap<String, Value>> for Struct {
    fn from(fields: BTreeMap<String, Value>) -> Self {
        Self { fields }
    }
}

impl From<HashMap<String, Value>> for Struct {
    fn from(fields: HashMap<String, Value>) -> Self {
        Self {
            fields: fields.into_iter().collect(),
        }
    }
}

impl From<BTreeMap<String, JsonValue>> for Struct {
    fn from(fields: BTreeMap<String, JsonValue>) -> Self {
        Self {
            fields: fields.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }
}

impl From<HashMap<String, JsonValue>> for Struct {
    fn from(fields: HashMap<String, JsonValue>) -> Self {
        Self {
            fields: fields.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (i, (k, v)) in self.fields.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{k}: {v}")?;
        }
        write!(f, "}}")
    }
}

impl Serialize for Struct {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        use ::serde::ser::SerializeMap;

        let mut s = serializer.serialize_map(Some(self.fields.len()))?;
        for (k, v) in &self.fields {
            s.serialize_entry(k, v)?;
        }
        s.end()
    }
}

impl<'de> Deserialize<'de> for Struct {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let value = BTreeMap::<String, JsonValue>::deserialize(deserializer)?;
        Ok(value.into())
    }
}

impl From<Value> for JsonValue {
    fn from(value: Value) -> Self {
        let Some(kind) = value.kind else {
            return JsonValue::Null;
        };
        match kind {
            ValueKind::NullValue(_) => JsonValue::Null,
            ValueKind::NumberValue(n) => JsonValue::Number(
                JsonNumber::from_f64(n).expect("NumberValue is expected to be a valid f64"),
            ),
            ValueKind::StringValue(s) => JsonValue::String(s),
            ValueKind::BoolValue(b) => JsonValue::Bool(b),
            ValueKind::StructValue(s) => {
                JsonValue::Object(s.fields.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
            ValueKind::ListValue(l) => {
                JsonValue::Array(l.values.into_iter().map(Into::into).collect())
            }
        }
    }
}

impl From<JsonValue> for Value {
    fn from(value: JsonValue) -> Self {
        match value {
            JsonValue::Null => Self {
                kind: Some(ValueKind::NullValue(0)),
            },
            JsonValue::Number(n) => Self {
                kind: Some(ValueKind::NumberValue(
                    n.as_f64()
                        .expect("NumberValue is expected to be a valid f64"),
                )),
            },
            JsonValue::String(s) => Self {
                kind: Some(ValueKind::StringValue(s)),
            },
            JsonValue::Bool(b) => Self {
                kind: Some(ValueKind::BoolValue(b)),
            },
            JsonValue::Object(o) => Self {
                kind: Some(ValueKind::StructValue(Struct::new(
                    o.into_iter().map(|(k, v)| (k, v.into())).collect(),
                ))),
            },
            JsonValue::Array(a) => Self {
                kind: Some(ValueKind::ListValue(
                    a.into_iter().map(Into::<Value>::into).collect(),
                )),
            },
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = JsonValue::from(self.clone());
        value.fmt(f)
    }
}

impl Serialize for Value {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        JsonValue::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let value = JsonValue::deserialize(deserializer)?;
        Ok(value.into())
    }
}

impl<T> FromIterator<T> for ListValue
where
    T: Into<Value>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            values: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl<I, T> From<I> for ListValue
where
    I: IntoIterator<Item = T>,
    T: Into<Value>,
{
    fn from(values: I) -> Self {
        Self::from_iter(values)
    }
}

impl From<Empty> for NullValue {
    fn from(_: Empty) -> Self {
        Self::NullValue
    }
}

impl From<NullValue> for Empty {
    fn from(_: NullValue) -> Self {
        Self {}
    }
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm",
))]
mod struct_wasm {
    use super::*;
    use js_sys::{Array, Object};
    use wasm_bindgen::{
        convert::{
            FromWasmAbi, IntoWasmAbi, LongRefFromWasmAbi, OptionFromWasmAbi, OptionIntoWasmAbi,
            RefFromWasmAbi, TryFromJsValue, VectorFromWasmAbi, VectorIntoWasmAbi,
        },
        describe::{WasmDescribe, WasmDescribeVector},
        prelude::*,
        JsCast, JsValue, UnwrapThrowExt,
    };

    impl WasmDescribe for Struct {
        #[inline]
        fn describe() {
            use wasm_bindgen::describe::*;
            const NAME: &str = "JsonObject";

            inform(NAMED_EXTERNREF);
            inform(NAME.len() as u32);
            for b in NAME.bytes() {
                inform(b as u32);
            }
        }
    }

    impl WasmDescribeVector for Struct {
        #[inline]
        fn describe_vector() {
            use wasm_bindgen::describe::*;
            inform(VECTOR);
            <Struct as WasmDescribe>::describe();
        }
    }

    #[wasm_bindgen(typescript_custom_section)]
    const TS_APPEND_CONTENT: &'static str = r#"
        type JsonObject = { [key: string]: JsonValue };
    "#;
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm",
))]
mod value_wasm {
    use super::*;
    use js_sys::{Array, Object};
    use wasm_bindgen::{
        convert::{
            FromWasmAbi, IntoWasmAbi, LongRefFromWasmAbi, OptionFromWasmAbi, OptionIntoWasmAbi,
            RefFromWasmAbi, TryFromJsValue, VectorFromWasmAbi, VectorIntoWasmAbi,
        },
        describe::{WasmDescribe, WasmDescribeVector},
        prelude::*,
        JsCast, JsValue, UnwrapThrowExt,
    };

    impl From<Value> for JsValue {
        fn from(value: Value) -> Self {
            let Some(kind) = value.kind else {
                return JsValue::null();
            };
            match kind {
                ValueKind::NullValue(_) => JsValue::null(),
                ValueKind::NumberValue(n) => JsValue::from_f64(n),
                ValueKind::StringValue(s) => JsValue::from_str(&s),
                ValueKind::BoolValue(b) => JsValue::from_bool(b),
                ValueKind::StructValue(Struct { fields }) => {
                    let obj = js_sys::Object::new();
                    for (k, v) in fields {
                        js_sys::Reflect::set(&obj, &k.into(), &v.into());
                    }
                    obj.into()
                }
                ValueKind::ListValue(list) => {
                    let js_arr = Array::new_with_length(list.values.len() as u32);
                    for v in list.values {
                        js_arr.push(&v.into());
                    }
                    js_arr.into()
                }
            }
        }
    }

    impl TryFrom<JsValue> for Value {
        type Error = JsValue;

        fn try_from(value: JsValue) -> Result<Self, Self::Error> {
            Ok(Value {
                kind: Some(if value.is_null() || value.is_undefined() {
                    ValueKind::NullValue(0)
                } else if let Some(n) = value.as_f64() {
                    ValueKind::NumberValue(n)
                } else if let Some(s) = value.as_string() {
                    ValueKind::StringValue(s)
                } else if let Some(b) = value.as_bool() {
                    ValueKind::BoolValue(b)
                } else if value.is_array() {
                    let arr: Array = value.unchecked_into();
                    let mut values = Vec::with_capacity(arr.length() as usize);
                    for value in arr.into_iter() {
                        values.push(value.try_into()?);
                    }
                    ValueKind::ListValue(ListValue { values })
                } else if value.is_object() {
                    let obj: BTreeMap<String, JsonValue> = serde_wasm_bindgen::from_value(value)?;
                    ValueKind::StructValue(obj.into())
                } else {
                    return Err(JsValue::from_str(&format!(
                        "cannot convert JSON value: {:?}",
                        value
                    )));
                }),
            })
        }
    }

    impl TryFromJsValue for Value {
        type Error = JsValue;

        #[inline]
        fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
            let js_value = value.try_into()?;
            Ok(js_value)
        }
    }

    impl WasmDescribe for Value {
        #[inline]
        fn describe() {
            use wasm_bindgen::describe::*;
            const NAME: &str = "JsonValue";

            inform(NAMED_EXTERNREF);
            inform(NAME.len() as u32);
            for b in NAME.bytes() {
                inform(b as u32);
            }
        }
    }

    impl WasmDescribeVector for Value {
        #[inline]
        fn describe_vector() {
            use wasm_bindgen::describe::*;
            inform(VECTOR);
            <Value as WasmDescribe>::describe();
        }
    }

    impl FromWasmAbi for Value {
        type Abi = <JsValue as FromWasmAbi>::Abi;

        #[inline]
        unsafe fn from_abi(js: Self::Abi) -> Self {
            let js_value = JsValue::from_abi(js);
            match js_value.try_into() {
                Ok(value) => value,
                Err(err) => wasm_bindgen::throw_val(err),
            }
        }
    }

    impl VectorFromWasmAbi for Value {
        type Abi = <wasm_bindgen::__rt::std::boxed::Box<[JsValue]> as FromWasmAbi>::Abi;

        #[inline]
        unsafe fn vector_from_abi(js: Self::Abi) -> wasm_bindgen::__rt::std::boxed::Box<[Value]> {
            wasm_bindgen::convert::js_value_vector_from_abi(js)
        }
    }

    impl OptionFromWasmAbi for Value {
        #[inline]
        fn is_none(js: &Self::Abi) -> bool {
            // TODO: Improve somehow?
            unsafe {
                let js_value = JsValue::from_abi(js.clone());
                js_value.is_null() || js_value.is_undefined()
            }
        }
    }

    impl RefFromWasmAbi for Value {
        type Abi = <JsValue as RefFromWasmAbi>::Abi;
        type Anchor = core::mem::ManuallyDrop<Value>;

        #[inline]
        unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
            let js_value = <JsValue as RefFromWasmAbi>::ref_from_abi(js);
            core::mem::ManuallyDrop::new(
                core::mem::ManuallyDrop::into_inner(js_value)
                    .try_into()
                    .unwrap_throw(),
            )
        }
    }

    impl LongRefFromWasmAbi for Value {
        type Abi = <JsValue as LongRefFromWasmAbi>::Abi;
        type Anchor = Value;

        #[inline]
        unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
            let js_value = <JsValue as LongRefFromWasmAbi>::long_ref_from_abi(js);
            js_value.try_into().unwrap_throw()
        }
    }

    impl IntoWasmAbi for Value {
        type Abi = <JsValue as IntoWasmAbi>::Abi;

        #[inline]
        fn into_abi(self) -> Self::Abi {
            JsValue::from(self).into_abi()
        }
    }

    impl VectorIntoWasmAbi for Value {
        type Abi = <wasm_bindgen::__rt::std::boxed::Box<[JsValue]> as IntoWasmAbi>::Abi;

        #[inline]
        fn vector_into_abi(vector: wasm_bindgen::__rt::std::boxed::Box<[Value]>) -> Self::Abi {
            wasm_bindgen::convert::js_value_vector_into_abi(vector)
        }
    }

    impl OptionIntoWasmAbi for Value {
        #[inline]
        fn none() -> Self::Abi {
            <js_sys::Object as OptionIntoWasmAbi>::none()
        }
    }

    #[wasm_bindgen(typescript_custom_section)]
    const TS_APPEND_CONTENT: &'static str = r#"
        type JsonValue = null | number | string | boolean | JsonObject | Array<JsonValue>;
    "#;
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn struct_values() {
        let js = json!({
            "x": 42,
            "msg": "Hello, World!",
            "seq": [1, 2, 3],
        });
        let s: Struct = js.into();
        assert_eq!(
            &serde_json::from_str::<Struct>(&serde_json::to_string_pretty(&s).unwrap()).unwrap(),
            &s
        );
    }
}
