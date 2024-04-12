use crate::google::protobuf::Empty;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(typescript_custom_section)]
    const TS_APPEND_CONTENT: &'static str = r#"
        export type Empty = {};
    "#;
};
