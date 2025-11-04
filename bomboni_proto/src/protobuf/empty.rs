use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::google::protobuf::Empty;

impl Empty {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Serialize for Empty {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'de> Deserialize<'de> for Empty {
    fn deserialize<D>(_deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {})
    }
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm",
))]
const _: () = {
    use wasm_bindgen::JsValue;

    impl From<Empty> for JsValue {
        fn from(_: Empty) -> Self {
            if cfg!(feature = "js") {
                JsValue::undefined()
            } else {
                JsValue::null()
            }
        }
    }

    impl TryFrom<JsValue> for Empty {
        type Error = JsValue;

        fn try_from(_: JsValue) -> Result<Self, Self::Error> {
            Ok(Self {})
        }
    }
};
