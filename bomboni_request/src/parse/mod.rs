use bomboni_common::id::Id;
use time::OffsetDateTime;
pub mod helpers;

pub trait RequestParse<T>: Sized {
    type Error;

    fn parse(value: T) -> Result<Self, Self::Error>;
}

pub trait RequestParseInto<T>: Sized {
    type Error;

    fn parse_into(self) -> Result<T, Self::Error>;
}

impl<T, U, E> RequestParseInto<U> for T
where
    U: RequestParse<T, Error = E>,
{
    type Error = U::Error;

    fn parse_into(self) -> Result<U, Self::Error> {
        U::parse(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedResource {
    pub name: String,
    pub create_time: Option<OffsetDateTime>,
    pub update_time: Option<OffsetDateTime>,
    pub delete_time: Option<OffsetDateTime>,
    pub deleted: bool,
    pub etag: Option<String>,
    pub revision_id: Option<Id>,
    pub revision_create_time: Option<OffsetDateTime>,
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use bomboni_common::{btree_map, btree_map_into, hash_map_into};
    use bomboni_proto::google::protobuf::{
        Int32Value, Int64Value, StringValue, Timestamp, UInt32Value,
    };
    use bomboni_request_derive::{impl_parse_into_map, parse_resource_name, Parse};

    use crate::error::{CommonError, FieldError, RequestError, RequestResult};

    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(i32)]
    enum DataTypeEnum {
        Unspecified = 0,
        String = 1,
        Boolean = 2,
        Number = 3,
    }

    impl TryFrom<i32> for DataTypeEnum {
        type Error = ();

        fn try_from(value: i32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(Self::Unspecified),
                1 => Ok(Self::String),
                2 => Ok(Self::Boolean),
                3 => Ok(Self::Number),
                _ => Err(()),
            }
        }
    }

    #[test]
    fn it_works() {
        #[derive(Debug, PartialEq)]
        struct Item {
            string: String,
            optional_string: Option<String>,
            required_string: String,
            required_string_optional: Option<String>,
            nested: Option<NestedItem>,
            optional_nested: Option<NestedItem>,
            default_nested: Option<NestedItem>,
            custom_parse: String,
            enum_value: i32,
            oneof: Option<OneofKind>,
        }

        impl Default for Item {
            fn default() -> Self {
                Self {
                    string: "abc".into(),
                    optional_string: Some("abc".into()),
                    required_string: "abc".into(),
                    required_string_optional: Some("abc".into()),
                    nested: Some(NestedItem {}),
                    optional_nested: Some(NestedItem {}),
                    default_nested: Some(NestedItem {}),
                    custom_parse: "42".into(),
                    enum_value: 1,
                    oneof: Some(OneofKind::String("abc".into())),
                }
            }
        }

        #[derive(Debug, PartialEq, Default)]
        struct NestedItem {}

        #[derive(Parse, Debug, PartialEq)]
        #[parse(source = Item, write)]
        struct ParsedItem {
            #[parse(source_name = "string")]
            s: String,
            #[parse(source_name = "optional_string")]
            opt_s: Option<String>,
            required_string: String,
            #[parse(source_option)]
            required_string_optional: String,
            nested: ParsedNestedItem,
            optional_nested: Option<ParsedNestedItem>,
            #[parse(default = NestedItem::default())]
            default_nested: ParsedNestedItem,
            #[parse(with = custom_parse)]
            custom_parse: u64,
            #[parse(enumeration)]
            enum_value: Enum,
            #[parse(oneof)]
            oneof: ParsedOneofKind,
            #[parse(skip)]
            extra: i32,
        }

        #[derive(Parse, Debug, Default, PartialEq)]
        #[parse(source = NestedItem, write)]
        struct ParsedNestedItem {}

        mod custom_parse {
            use super::*;

            pub fn parse(value: String) -> RequestResult<u64> {
                value
                    .parse()
                    .map_err(|_| CommonError::InvalidNumericValue.into())
            }

            pub fn write(value: u64) -> String {
                value.to_string()
            }
        }

        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        #[repr(i32)]
        enum Enum {
            Unspecified = 0,
            A = 1,
            B = 2,
        }

        impl TryFrom<i32> for Enum {
            type Error = ();

            fn try_from(value: i32) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(Self::Unspecified),
                    1 => Ok(Self::A),
                    2 => Ok(Self::B),
                    _ => Err(()),
                }
            }
        }

        #[derive(Debug, PartialEq)]
        #[allow(dead_code)]
        enum OneofKind {
            String(String),
            Boolean(bool),
            Nested(NestedItem),
        }

        #[derive(Parse, Debug, PartialEq)]
        #[allow(dead_code)]
        #[parse(source = OneofKind, write)]
        enum ParsedOneofKind {
            String(String),
            Boolean(bool),
            Nested(ParsedNestedItem),
        }

        impl OneofKind {
            pub fn get_variant_name(&self) -> &'static str {
                match self {
                    Self::String(_) => "string",
                    Self::Boolean(_) => "boolean",
                    Self::Nested(_) => "nested",
                }
            }
        }

        impl Default for ParsedOneofKind {
            fn default() -> Self {
                Self::Boolean(false)
            }
        }

        macro_rules! assert_parse_field_err {
            ($item:expr, $field:expr, $err:expr) => {{
                let err = ParsedItem::parse($item).unwrap_err();
                if let RequestError::Field(FieldError { error, field, .. }) = err {
                    assert_eq!(field, $field);
                    assert_eq!(error.as_any().downcast_ref::<CommonError>().unwrap(), $err);
                } else {
                    panic!("expected FieldError, got {:?}", err);
                }
            }};
        }

        assert_parse_field_err!(
            Item {
                required_string: String::new(),
                ..Default::default()
            },
            "required_string",
            &CommonError::RequiredFieldMissing
        );
        assert_parse_field_err!(
            Item {
                required_string_optional: None,
                ..Default::default()
            },
            "required_string_optional",
            &CommonError::RequiredFieldMissing
        );
        assert_parse_field_err!(
            Item {
                required_string_optional: Some(String::new()),
                ..Default::default()
            },
            "required_string_optional",
            &CommonError::RequiredFieldMissing
        );
        assert_parse_field_err!(
            Item {
                nested: None,
                ..Default::default()
            },
            "nested",
            &CommonError::RequiredFieldMissing
        );
        assert_parse_field_err!(
            Item {
                custom_parse: "abc".into(),
                ..Default::default()
            },
            "custom_parse",
            &CommonError::InvalidNumericValue
        );
        assert_parse_field_err!(
            Item {
                enum_value: 99,
                ..Default::default()
            },
            "enum_value",
            &CommonError::InvalidEnumValue
        );
        assert_parse_field_err!(
            Item {
                enum_value: 0,
                ..Default::default()
            },
            "enum_value",
            &CommonError::RequiredFieldMissing
        );
        assert_parse_field_err!(
            Item {
                oneof: None,
                ..Default::default()
            },
            "oneof",
            &CommonError::RequiredFieldMissing
        );

        assert_eq!(
            ParsedItem::parse(Item {
                string: "abc".into(),
                optional_string: Some("hello".into()),
                required_string: "world".into(),
                required_string_optional: Some("world".into()),
                nested: Some(NestedItem {}),
                optional_nested: Some(NestedItem {}),
                default_nested: None,
                custom_parse: "42".into(),
                enum_value: 1,
                oneof: Some(OneofKind::String("abc".into())),
            })
            .unwrap(),
            ParsedItem {
                s: "abc".into(),
                opt_s: Some("hello".into()),
                required_string: "world".into(),
                required_string_optional: "world".into(),
                nested: ParsedNestedItem {},
                optional_nested: Some(ParsedNestedItem {}),
                default_nested: ParsedNestedItem {},
                custom_parse: 42,
                enum_value: Enum::A,
                oneof: ParsedOneofKind::String("abc".into()),
                extra: 0,
            }
        );

        assert_eq!(
            Item::from(ParsedItem {
                s: "abc".into(),
                opt_s: Some("abc".into()),
                required_string: "abc".into(),
                required_string_optional: "abc".into(),
                nested: ParsedNestedItem {},
                optional_nested: Some(ParsedNestedItem {}),
                default_nested: ParsedNestedItem {},
                custom_parse: 1337,
                enum_value: Enum::B,
                oneof: ParsedOneofKind::String("abc".into()),
                extra: 0,
            }),
            Item {
                string: "abc".into(),
                optional_string: Some("abc".into()),
                required_string: "abc".into(),
                required_string_optional: Some("abc".into()),
                nested: Some(NestedItem {}),
                optional_nested: Some(NestedItem {}),
                default_nested: Some(NestedItem {}),
                custom_parse: "1337".into(),
                enum_value: 2,
                oneof: Some(OneofKind::String("abc".into())),
            }
        );
    }

    #[test]
    fn parse_regex() {
        #[derive(Debug)]
        struct Item {
            value: String,
        }
        #[derive(Parse, Debug, PartialEq)]
        #[parse(source = Item)]
        struct ParsedItem {
            #[parse(regex = "^[a-z]+$")]
            value: String,
        }

        assert!(ParsedItem::parse(Item {
            value: "abc".into(),
        })
        .is_ok());
        assert!(matches!(
            ParsedItem::parse(Item {
                value: "123".into(),
            })
            .unwrap_err(),
            RequestError::Field(FieldError {
                error,..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::InvalidStringFormat { .. }
            )
        ));
    }

    #[test]
    fn custom_parse_fields() {
        #[derive(Debug)]
        struct Item {
            name: String,
            value: String,
        }

        impl Default for Item {
            fn default() -> Self {
                Self {
                    name: "a/1/b/1".into(),
                    value: "42".into(),
                }
            }
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(source = Item)]
        struct ParsedItem {
            #[parse(parse_with = parse_name)]
            name: (u32, u64),
            #[parse(parse_with = parse_value)]
            value: i32,
        }

        impl Default for ParsedItem {
            fn default() -> Self {
                Self {
                    name: (1, 1),
                    value: 42,
                }
            }
        }

        fn parse_name<S: ToString>(name: S) -> RequestResult<(u32, u64)> {
            let name = name.to_string();
            Ok(parse_resource_name!({
                "a": u32,
                "b": u64,
            })(&name)
            .ok_or_else(|| CommonError::InvalidName {
                expected_format: "...".into(),
                name,
            })?)
        }

        fn parse_value<S: ToString>(value: S) -> RequestResult<i32> {
            let value = value.to_string();
            Ok(value
                .parse()
                .map_err(|_| CommonError::InvalidNumericValue)?)
        }

        assert_eq!(
            ParsedItem::parse(Item {
                name: "a/42/b/1337".into(),
                ..Default::default()
            })
            .unwrap(),
            ParsedItem {
                name: (42, 1337),
                ..Default::default()
            }
        );
        assert!(matches!(
            ParsedItem::parse(Item {
                name: "123".into(),
                ..Default::default()
            })
            .unwrap_err(),
            RequestError::Field(FieldError {
                error,..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::InvalidName { .. }
            )
        ));
    }

    #[test]
    fn parse_maps() {
        impl_parse_into_map!(
            pub parse1,
            |item: i32| -> (i32, String) { (item, item.to_string()) },
        );
        assert_eq!(
            parse1::parse(vec![1, 2, 3]).unwrap(),
            btree_map_into! {
                1 => "1",
                2 => "2",
                3 => "3",
            }
        );

        impl_parse_into_map!(
            parse2,
            |item: &'static str| -> RequestResult<(i32, String)> {
                let value: i32 = item.parse().map_err(|_| CommonError::InvalidNumericValue)?;
                Ok((value, item.to_string()))
            },
        );
        assert_eq!(
            parse2::parse(vec!["1", "2", "3"]).unwrap(),
            btree_map_into! {
                1 => "1",
                2 => "2",
                3 => "3",
            }
        );

        impl_parse_into_map!(
            parse3,
            |item| (item, ""),
            |item: (i32, &'static str)| -> i32 { item.0 },
        );
        assert_eq!(
            parse3::write(btree_map! {
                1 => "1",
                2 => "2",
                3 => "3",
            }),
            vec![1, 2, 3]
        );

        #[derive(Debug, Default)]
        struct Item {
            values: Vec<(i32, i32)>,
        }
        #[derive(Parse, Debug, PartialEq)]
        #[parse(source = Item)]
        struct ParsedItem {
            #[parse(with = values_parse_hash)]
            values: HashMap<i32, i32>,
        }
        impl_parse_into_map!(
            values_parse_hash,
            HashMap,
            |item: (i32, i32)| -> (i32, i32) { item },
        );

        assert_eq!(
            ParsedItem::parse(Item {
                values: vec![(1, 2), (2, 4), (3, 9)],
            })
            .unwrap(),
            ParsedItem {
                values: hash_map_into! {
                    1 => 2,
                    2 => 4,
                    3 => 9,
                },
            }
        );
        assert!(matches!(
            ParsedItem::parse(Item {
                values: vec![(1, 2), (1, 4)],
            })
            .unwrap_err(),
            RequestError::Field(FieldError {
                error,..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::DuplicateValue
            )
        ));
    }

    #[test]
    fn parse_resource() {
        #[derive(Debug, PartialEq, Default)]
        struct Item {
            name: String,
            create_time: Option<Timestamp>,
            update_time: Option<Timestamp>,
            delete_time: Option<Timestamp>,
            deleted: bool,
            etag: Option<String>,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(source = Item, write)]
        struct ParsedItem {
            #[parse(resource {
                fields = [name, create_time, deleted, etag],
            })]
            resource: ParsedResource,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(source = Item, write)]
        struct ParsedItemDefaultResource {
            #[parse(resource)]
            resource: ParsedResource,
        }

        assert_eq!(
            ParsedItem::parse(Item {
                name: "items/42".into(),
                create_time: Some(OffsetDateTime::UNIX_EPOCH.into()),
                deleted: true,
                etag: Some("abc".into()),
                ..Default::default()
            })
            .unwrap(),
            ParsedItem {
                resource: ParsedResource {
                    name: "items/42".into(),
                    create_time: Some(OffsetDateTime::UNIX_EPOCH),
                    deleted: true,
                    etag: Some("abc".into()),
                    ..Default::default()
                }
            }
        );
        assert_eq!(
            Item::from(ParsedItem {
                resource: ParsedResource {
                    name: "items/42".into(),
                    create_time: Some(OffsetDateTime::UNIX_EPOCH),
                    deleted: true,
                    ..Default::default()
                },
            }),
            Item {
                name: "items/42".into(),
                create_time: Some(OffsetDateTime::UNIX_EPOCH.into()),
                deleted: true,
                ..Default::default()
            }
        );

        assert_eq!(
            ParsedItemDefaultResource::parse(Item {
                name: "items/42".into(),
                create_time: Some(OffsetDateTime::UNIX_EPOCH.into()),
                deleted: true,
                ..Default::default()
            })
            .unwrap(),
            ParsedItemDefaultResource {
                resource: ParsedResource {
                    name: "items/42".into(),
                    create_time: Some(OffsetDateTime::UNIX_EPOCH),
                    deleted: true,
                    ..Default::default()
                }
            }
        );
    }

    #[test]
    fn derived_fields() {
        #[derive(Debug, PartialEq, Default)]
        struct Item {
            x: i32,
            y: i32,
            name: String,
        }
        #[derive(Parse, Debug, PartialEq)]
        #[parse(source = Item, write)]
        struct ParsedItem {
            #[parse(derive = derive_value)]
            z: i32,
            name: String,
            #[parse(derive = derive_upper_name)]
            upper_name: String,
            #[parse(derive = (derive_lower_name, name))]
            lower_name: String,
        }

        #[allow(clippy::unnecessary_wraps)]
        fn derive_value(item: &Item) -> RequestResult<i32> {
            Ok(item.x + item.y)
        }

        #[allow(clippy::unnecessary_wraps)]
        fn derive_upper_name(item: &Item) -> RequestResult<String> {
            Ok(item.name.to_uppercase())
        }

        #[allow(clippy::unnecessary_wraps)]
        fn derive_lower_name(name: &str, field_name: &str) -> RequestResult<String> {
            assert_eq!(field_name, "name");
            Ok(name.to_lowercase())
        }

        assert_eq!(
            ParsedItem::parse(Item {
                x: 3,
                y: 5,
                name: "Item".into()
            })
            .unwrap()
            .z,
            8
        );
        assert_eq!(
            ParsedItem::parse(Item {
                x: 3,
                y: 5,
                name: "Item".into()
            })
            .unwrap(),
            ParsedItem {
                z: 8,
                name: "Item".into(),
                upper_name: "ITEM".into(),
                lower_name: "item".into(),
            }
        );
        assert_eq!(
            Item::from(ParsedItem {
                z: 8,
                name: "Item".into(),
                upper_name: String::new(),
                lower_name: String::new(),
            }),
            Item {
                x: 0,
                y: 0,
                name: "Item".into()
            }
        );
    }

    #[test]
    fn parse_names() {
        let f = parse_resource_name!({
            "users": u32,
            "projects": u64,
            "revisions": Option<String>,
        });

        let (user_id, project_id, revision_id) = f("users/3/projects/5/revisions/1337").unwrap();
        assert_eq!(user_id, 3);
        assert_eq!(project_id, 5);
        assert_eq!(revision_id, Some("1337".to_string()));

        let (user_id, project_id, revision_id) = f("users/3/projects/5").unwrap();
        assert_eq!(user_id, 3);
        assert_eq!(project_id, 5);
        assert!(revision_id.is_none());

        assert!(parse_resource_name!({
            "a": u32,
            "b": u32,
        })("a/1/b/1/c/1")
        .is_none());
    }

    #[test]
    fn parse_collections() {
        #[derive(Debug, PartialEq, Default)]
        struct Item {
            values: Vec<i32>,
            strings: Vec<String>,
            items: Vec<NestedItem>,
            values_map: BTreeMap<String, i32>,
            items_map: HashMap<i32, NestedItem>,
            enums: Vec<i32>,
        }

        #[derive(Debug, PartialEq, Default)]
        struct NestedItem {
            value: i32,
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(source = Item, write)]
        struct ParsedItem {
            values: Vec<i32>,
            #[parse(regex = "^[a-z]$")]
            strings: Vec<String>,
            items: Vec<ParsedNestedItem>,
            values_map: BTreeMap<String, i32>,
            items_map: HashMap<i32, ParsedNestedItem>,
            #[parse(enumeration)]
            enums: Vec<DataTypeEnum>,
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(source = NestedItem, write)]
        struct ParsedNestedItem {
            value: i32,
        }

        assert_eq!(
            ParsedItem::parse(Item {
                values: vec![1, 2, 3],
                strings: vec!["a".into(), "b".into()],
                items: vec![NestedItem { value: 1 }, NestedItem { value: 2 }],
                values_map: btree_map_into! {
                    "a" => 1,
                    "b" => 2,
                },
                items_map: hash_map_into! {
                    1 => NestedItem { value: 1 },
                    2 => NestedItem { value: 2 },
                },
                enums: vec![1],
            })
            .unwrap(),
            ParsedItem {
                values: vec![1, 2, 3],
                strings: vec!["a".into(), "b".into()],
                items: vec![ParsedNestedItem { value: 1 }, ParsedNestedItem { value: 2 }],
                values_map: btree_map_into! {
                    "a" => 1,
                    "b" => 2,
                },
                items_map: hash_map_into! {
                    1 => ParsedNestedItem { value: 1 },
                    2 => ParsedNestedItem { value: 2 },
                },
                enums: vec![DataTypeEnum::String],
            }
        );
        assert!(matches!(
            ParsedItem::parse(Item {
                strings: vec!["Hello".into()],
                ..Default::default()
            }).unwrap_err(),
            RequestError::Field(FieldError {
                error, field, ..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::InvalidStringFormat { .. }
            ) && field == "strings"
        ));
        assert!(matches!(ParsedItem::parse(Item {
                enums: vec![99i32],
                ..Default::default()
            })
            .unwrap_err(),
            RequestError::Field(FieldError {
                error, field, ..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::InvalidEnumValue
            ) && field == "enums"
        ));

        assert_eq!(
            Item::from(ParsedItem {
                values: vec![1, 2, 3],
                strings: vec!["a".into(), "b".into()],
                items: vec![ParsedNestedItem { value: 1 }, ParsedNestedItem { value: 2 }],
                values_map: btree_map_into! {
                    "a" => 1,
                    "b" => 2,
                },
                items_map: hash_map_into! {
                    1 => ParsedNestedItem { value: 1 },
                    2 => ParsedNestedItem { value: 2 },
                },
                enums: vec![DataTypeEnum::String],
            }),
            Item {
                values: vec![1, 2, 3],
                strings: vec!["a".into(), "b".into()],
                items: vec![NestedItem { value: 1 }, NestedItem { value: 2 }],
                values_map: btree_map_into! {
                    "a" => 1,
                    "b" => 2,
                },
                items_map: hash_map_into! {
                    1 => NestedItem { value: 1 },
                    2 => NestedItem { value: 2 },
                },
                enums: vec![1],
            }
        );
    }

    #[test]
    fn parse_boxes() {
        #[derive(Debug, PartialEq, Default)]
        struct Item {
            value: Box<i32>,
            item: Option<Box<NestedItem>>,
            unboxed_item: Option<Box<NestedItem>>,
            optional_item: Option<Box<NestedItem>>,
        }

        #[derive(Debug, PartialEq, Default)]
        struct NestedItem {
            value: i32,
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(source = Item, write)]
        struct ParsedItem {
            value: Box<i32>,
            item: Box<ParsedNestedItem>,
            #[parse(source_box)]
            unboxed_item: ParsedNestedItem,
            optional_item: Option<Box<ParsedNestedItem>>,
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(source = NestedItem, write)]
        struct ParsedNestedItem {
            value: i32,
        }

        assert_eq!(
            ParsedItem::parse(Item {
                value: Box::new(42),
                item: Some(Box::new(NestedItem { value: 42 })),
                unboxed_item: Some(Box::new(NestedItem { value: 42 })),
                optional_item: Some(Box::new(NestedItem { value: 42 })),
            })
            .unwrap(),
            ParsedItem {
                value: Box::new(42),
                item: Box::new(ParsedNestedItem { value: 42 }),
                unboxed_item: ParsedNestedItem { value: 42 },
                optional_item: Some(Box::new(ParsedNestedItem { value: 42 })),
            }
        );
        assert_eq!(
            Item::from(ParsedItem {
                value: Box::new(42),
                item: Box::new(ParsedNestedItem { value: 42 }),
                unboxed_item: ParsedNestedItem { value: 42 },
                optional_item: Some(Box::new(ParsedNestedItem { value: 42 })),
            }),
            Item {
                value: Box::new(42),
                item: Some(Box::new(NestedItem { value: 42 })),
                unboxed_item: Some(Box::new(NestedItem { value: 42 })),
                optional_item: Some(Box::new(NestedItem { value: 42 })),
            }
        );
    }

    #[test]
    fn variant_wrapper_value() {
        #[derive(Debug, Clone, PartialEq)]
        struct Value {
            kind: Option<ValueKind>,
        }

        #[derive(Debug, Clone, PartialEq)]
        enum ValueKind {
            I32(Int32Value),
            U16(UInt32Value),
            String(StringValue),
            ISize(Int64Value),
            USize(UInt32Value),
        }

        impl ValueKind {
            pub fn get_variant_name(&self) -> &'static str {
                match self {
                    Self::I32(_) => "I32",
                    Self::U16(_) => "U16",
                    Self::String(_) => "String",
                    Self::ISize(_) => "ISize",
                    Self::USize(_) => "USize",
                }
            }
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(source = Value, tagged_union { oneof = ValueKind, field = kind }, write)]
        enum ParsedValue {
            #[parse(wrapper)]
            I32(i32),
            #[parse(wrapper)]
            U16(u16),
            #[parse(wrapper)]
            String(String),
            #[parse(wrapper)]
            ISize(isize),
            #[parse(wrapper)]
            USize(usize),
        }
    }

    #[test]
    fn parse_tagged_union() {
        #[derive(Debug, Clone, PartialEq)]
        struct Value {
            kind: Option<ValueKind>,
        }

        #[derive(Debug, Clone, PartialEq)]
        enum ValueKind {
            Number(i32),
            Inner(Box<Value>),
            Nested(NestedValue),
        }

        #[derive(Debug, Clone, Default, PartialEq)]
        struct NestedValue {
            value: i32,
        }

        impl ValueKind {
            pub fn get_variant_name(&self) -> &'static str {
                match self {
                    Self::Number(_) => "number",
                    Self::Inner(_) => "inner",
                    Self::Nested(_) => "nested",
                }
            }
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(source = Value, tagged_union { oneof = ValueKind, field = kind }, write)]
        enum ParsedValue {
            Number(i32),
            Inner(Box<ParsedValue>),
            Nested(ParsedNestedValue),
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(source = NestedValue, write)]
        struct ParsedNestedValue {
            value: i32,
        }

        assert_eq!(
            ParsedValue::parse(Value {
                kind: Some(ValueKind::Number(42)),
            })
            .unwrap(),
            ParsedValue::Number(42)
        );
        assert_eq!(
            Value::from(ParsedValue::Number(42)),
            Value {
                kind: Some(ValueKind::Number(42)),
            }
        );

        assert_eq!(
            ParsedValue::parse(Value {
                kind: Some(ValueKind::Inner(Box::new(Value {
                    kind: Some(ValueKind::Number(42)),
                }))),
            })
            .unwrap(),
            ParsedValue::Inner(Box::new(ParsedValue::Number(42)))
        );
        assert_eq!(
            Value::from(ParsedValue::Inner(Box::new(ParsedValue::Number(42)))),
            Value {
                kind: Some(ValueKind::Inner(Box::new(Value {
                    kind: Some(ValueKind::Number(42)),
                }))),
            }
        );

        assert_eq!(
            ParsedValue::parse(Value {
                kind: Some(ValueKind::Nested(NestedValue { value: 42 })),
            })
            .unwrap(),
            ParsedValue::Nested(ParsedNestedValue { value: 42 })
        );
        assert_eq!(
            Value::from(ParsedValue::Nested(ParsedNestedValue { value: 42 })),
            Value {
                kind: Some(ValueKind::Nested(NestedValue { value: 42 })),
            }
        );
    }

    #[test]
    fn parse_oneof() {
        #[derive(Debug, Clone, PartialEq)]
        enum Item {
            String(String),
            Data(Data),
            Null(()),
            Empty,
            Dropped(i32),
        }

        impl Item {
            pub fn get_variant_name(&self) -> &'static str {
                match self {
                    Self::String(_) => "string",
                    Self::Data(_) => "data",
                    Self::Null(()) => "null",
                    Self::Empty => "empty",
                    Self::Dropped(_) => "dropped",
                }
            }
        }

        #[derive(Debug, Clone, Default, PartialEq)]
        struct Data {
            value: i32,
        }

        #[derive(Debug, Clone, PartialEq, Parse)]
        #[parse(source= Item, write)]
        enum ParsedItem {
            String(String),
            Data(ParsedData),
            Null(()),
            #[parse(source_empty)]
            Empty,
            Dropped,
        }

        #[derive(Debug, Clone, Default, PartialEq, Parse)]
        #[parse(source= Data, write)]
        struct ParsedData {
            value: i32,
        }

        assert_eq!(
            ParsedItem::parse(Item::String("abc".into())).unwrap(),
            ParsedItem::String("abc".into())
        );
        assert_eq!(
            Item::from(ParsedItem::String("abc".into())),
            Item::String("abc".into())
        );

        assert_eq!(
            ParsedItem::parse(Item::Data(Data { value: 42 })).unwrap(),
            ParsedItem::Data(ParsedData { value: 42 })
        );
        assert_eq!(
            Item::from(ParsedItem::Data(ParsedData { value: 42 })),
            Item::Data(Data { value: 42 })
        );

        assert_eq!(ParsedItem::parse(Item::Empty).unwrap(), ParsedItem::Empty);
        assert_eq!(Item::from(ParsedItem::Empty), Item::Empty);
    }

    #[test]
    fn source_convert() {
        #[derive(Debug, Clone, PartialEq, Default)]
        struct Item {
            casted_value: u32,
            optional_casted: Option<u32>,
        }

        #[derive(Debug, Clone, PartialEq, Default, Parse)]
        #[parse(source = Item, write)]
        struct ParsedItem {
            #[parse(source_try_from = u32)]
            casted_value: usize,
            #[parse(source_try_from = u32)]
            optional_casted: Option<usize>,
        }

        #[derive(Debug, Clone, PartialEq)]
        enum Union {
            A(u32),
            B(Option<u32>),
        }

        impl Union {
            pub fn get_variant_name(&self) -> &'static str {
                match self {
                    Self::A(_) => "a",
                    Self::B(_) => "b",
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Parse)]
        #[parse(source = Union, write)]
        enum ParsedUnion {
            #[parse(source_try_from = u32)]
            A(usize),
            #[parse(source_try_from = u32)]
            B(Option<usize>),
        }

        assert_eq!(
            ParsedItem::parse(Item {
                casted_value: 42,
                optional_casted: Some(42),
            })
            .unwrap(),
            ParsedItem {
                casted_value: 42,
                optional_casted: Some(42),
            }
        );
        assert_eq!(
            Item::from(ParsedItem {
                casted_value: 42,
                optional_casted: Some(42),
            }),
            Item {
                casted_value: 42,
                optional_casted: Some(42),
            }
        );

        assert_eq!(
            ParsedUnion::parse(Union::A(42)).unwrap(),
            ParsedUnion::A(42)
        );
        assert_eq!(Union::from(ParsedUnion::A(42)), Union::A(42));
        assert_eq!(Union::from(ParsedUnion::B(Some(42))), Union::B(Some(42)));
        assert_eq!(Union::from(ParsedUnion::B(None)), Union::B(None));
    }
}
