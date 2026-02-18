use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Number as JsonNumber, Value as JsonValue};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    fmt::Display,
};

use crate::google::protobuf::{
    Empty, ListValue, NullValue, Struct, Value, value::Kind as ValueKind,
};

impl Struct {
    /// Creates a new struct with the given fields.
    #[must_use]
    pub const fn new(fields: BTreeMap<String, Value>) -> Self {
        Self { fields }
    }
}

impl TryFrom<JsonValue> for Struct {
    type Error = &'static str;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        match value {
            JsonValue::Object(o) => Ok(Self::new(
                o.into_iter().map(|(k, v)| (k, v.into())).collect(),
            )),
            _ => Err("JsonValue::Object is expected"),
        }
    }
}

impl From<Struct> for JsonValue {
    fn from(value: Struct) -> Self {
        Self::Object(
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
        use serde::ser::SerializeMap;

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
            return Self::Null;
        };
        match kind {
            ValueKind::NullValue(_) => Self::Null,
            ValueKind::NumberValue(n) => Self::Number(
                JsonNumber::from_f64(n).expect("NumberValue is expected to be a valid f64"),
            ),
            ValueKind::StringValue(s) => Self::String(s),
            ValueKind::BoolValue(b) => Self::Bool(b),
            ValueKind::StructValue(s) => {
                Self::Object(s.fields.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
            ValueKind::ListValue(l) => Self::Array(l.values.into_iter().map(Into::into).collect()),
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
                    a.into_iter().map(Into::<Self>::into).collect(),
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
const _: () = {
    use js_sys::{Array, Object};
    use wasm_bindgen::{JsCast, JsValue};

    impl From<Struct> for JsValue {
        fn from(value: Struct) -> Self {
            let obj = Object::new();
            for (k, v) in value.fields {
                js_sys::Reflect::set(&obj, &k.into(), &v.into()).unwrap();
            }
            obj.into()
        }
    }

    impl TryFrom<JsValue> for Struct {
        type Error = JsValue;

        fn try_from(value: JsValue) -> Result<Self, Self::Error> {
            let obj: Object = value.unchecked_into();
            let mut fields = BTreeMap::new();
            for k in Object::keys(&obj).iter() {
                let v = js_sys::Reflect::get(&obj, &k).unwrap();
                fields.insert(
                    k.as_string()
                        .ok_or_else(|| js_sys::Error::new("key is not a string"))?,
                    v.try_into()?,
                );
            }
            Ok(Struct { fields })
        }
    }

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
                    let obj = Object::new();
                    for (k, v) in fields {
                        js_sys::Reflect::set(&obj, &k.into(), &v.into()).unwrap();
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
                    for value in arr {
                        values.push(value.try_into()?);
                    }
                    ValueKind::ListValue(ListValue { values })
                } else if value.is_object() {
                    let obj: BTreeMap<String, JsonValue> = serde_wasm_bindgen::from_value(value)?;
                    ValueKind::StructValue(obj.into())
                } else {
                    return Err(js_sys::Error::new(&format!(
                        "cannot convert JSON value: {value:?}"
                    ))
                    .into());
                }),
            })
        }
    }
};

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
        let s: Struct = if let serde_json::Value::Object(map) = js {
            map.into_iter().collect::<BTreeMap<_, _>>().into()
        } else {
            panic!("Expected object");
        };
        assert_eq!(
            &serde_json::from_str::<Struct>(&serde_json::to_string_pretty(&s).unwrap()).unwrap(),
            &s
        );
    }
}
