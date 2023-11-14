///Implement [`prost::Name`] for `Timestamp`.
impl ::prost::Name for Timestamp {
    const NAME: &'static str = "Timestamp";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl Timestamp {
    pub const TYPE_URL: &'static str = "type.googleapis.com/Timestamp";
}
impl Timestamp {
    pub const SECONDS_FIELD_NAME: &'static str = "seconds";
    pub const NANOS_FIELD_NAME: &'static str = "nanos";
}
///Implement [`prost::Name`] for `DoubleValue`.
impl ::prost::Name for DoubleValue {
    const NAME: &'static str = "DoubleValue";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl DoubleValue {
    pub const TYPE_URL: &'static str = "type.googleapis.com/DoubleValue";
}
impl DoubleValue {
    pub const VALUE_FIELD_NAME: &'static str = "value";
}
///Implement [`prost::Name`] for `FloatValue`.
impl ::prost::Name for FloatValue {
    const NAME: &'static str = "FloatValue";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl FloatValue {
    pub const TYPE_URL: &'static str = "type.googleapis.com/FloatValue";
}
impl FloatValue {
    pub const VALUE_FIELD_NAME: &'static str = "value";
}
///Implement [`prost::Name`] for `Int64Value`.
impl ::prost::Name for Int64Value {
    const NAME: &'static str = "Int64Value";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl Int64Value {
    pub const TYPE_URL: &'static str = "type.googleapis.com/Int64Value";
}
impl Int64Value {
    pub const VALUE_FIELD_NAME: &'static str = "value";
}
///Implement [`prost::Name`] for `UInt64Value`.
impl ::prost::Name for UInt64Value {
    const NAME: &'static str = "UInt64Value";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl UInt64Value {
    pub const TYPE_URL: &'static str = "type.googleapis.com/UInt64Value";
}
impl UInt64Value {
    pub const VALUE_FIELD_NAME: &'static str = "value";
}
///Implement [`prost::Name`] for `Int32Value`.
impl ::prost::Name for Int32Value {
    const NAME: &'static str = "Int32Value";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl Int32Value {
    pub const TYPE_URL: &'static str = "type.googleapis.com/Int32Value";
}
impl Int32Value {
    pub const VALUE_FIELD_NAME: &'static str = "value";
}
///Implement [`prost::Name`] for `UInt32Value`.
impl ::prost::Name for UInt32Value {
    const NAME: &'static str = "UInt32Value";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl UInt32Value {
    pub const TYPE_URL: &'static str = "type.googleapis.com/UInt32Value";
}
impl UInt32Value {
    pub const VALUE_FIELD_NAME: &'static str = "value";
}
///Implement [`prost::Name`] for `BoolValue`.
impl ::prost::Name for BoolValue {
    const NAME: &'static str = "BoolValue";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl BoolValue {
    pub const TYPE_URL: &'static str = "type.googleapis.com/BoolValue";
}
impl BoolValue {
    pub const VALUE_FIELD_NAME: &'static str = "value";
}
///Implement [`prost::Name`] for `StringValue`.
impl ::prost::Name for StringValue {
    const NAME: &'static str = "StringValue";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl StringValue {
    pub const TYPE_URL: &'static str = "type.googleapis.com/StringValue";
}
impl StringValue {
    pub const VALUE_FIELD_NAME: &'static str = "value";
}
///Implement [`prost::Name`] for `BytesValue`.
impl ::prost::Name for BytesValue {
    const NAME: &'static str = "BytesValue";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl BytesValue {
    pub const TYPE_URL: &'static str = "type.googleapis.com/BytesValue";
}
impl BytesValue {
    pub const VALUE_FIELD_NAME: &'static str = "value";
}
///Implement [`prost::Name`] for `Any`.
impl ::prost::Name for Any {
    const NAME: &'static str = "Any";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl Any {
    pub const TYPE_URL: &'static str = "type.googleapis.com/Any";
}
impl Any {
    pub const TYPE_URL_FIELD_NAME: &'static str = "type_url";
    pub const VALUE_FIELD_NAME: &'static str = "value";
}
///Implement [`prost::Name`] for `FieldMask`.
impl ::prost::Name for FieldMask {
    const NAME: &'static str = "FieldMask";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl FieldMask {
    pub const TYPE_URL: &'static str = "type.googleapis.com/FieldMask";
}
impl FieldMask {
    pub const PATHS_FIELD_NAME: &'static str = "paths";
}
///Implement [`prost::Name`] for `Empty`.
impl ::prost::Name for Empty {
    const NAME: &'static str = "Empty";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl Empty {
    pub const TYPE_URL: &'static str = "type.googleapis.com/Empty";
}
///Implement [`prost::Name`] for `Duration`.
impl ::prost::Name for Duration {
    const NAME: &'static str = "Duration";
    const PACKAGE: &'static str = "google.protobuf";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl Duration {
    pub const TYPE_URL: &'static str = "type.googleapis.com/Duration";
}
impl Duration {
    pub const SECONDS_FIELD_NAME: &'static str = "seconds";
    pub const NANOS_FIELD_NAME: &'static str = "nanos";
}
