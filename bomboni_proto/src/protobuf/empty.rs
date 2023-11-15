use crate::google::protobuf::Empty;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
impl Empty {
    pub const fn new() -> Self {
        Empty {}
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
        Ok(Empty {})
    }
}
