use bomboni_common::date_time::UtcDateTime;
use bomboni_common::id::Id;
#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
use wasm_bindgen::prelude::*;

use crate::{error::RequestResult, string::String};

pub mod helpers;

pub trait RequestParse<T>: Sized {
    fn parse(value: T) -> RequestResult<Self>;
}

pub trait RequestParseInto<T>: Sized {
    fn parse_into(self) -> RequestResult<T>;
}

impl<T, U> RequestParseInto<U> for T
where
    U: RequestParse<T>,
{
    fn parse_into(self) -> RequestResult<U> {
        U::parse(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(
    all(
        target_family = "wasm",
        not(any(target_os = "emscripten", target_os = "wasi")),
        feature = "wasm"
    ),
    wasm_bindgen(getter_with_clone, inspectable)
)]
pub struct ParsedResource {
    pub name: String,
    pub create_time: Option<UtcDateTime>,
    pub update_time: Option<UtcDateTime>,
    pub delete_time: Option<UtcDateTime>,
    pub deleted: bool,
    pub etag: Option<String>,
    pub revision_id: Option<Id>,
    pub revision_create_time: Option<UtcDateTime>,
}

#[cfg(feature = "testing")]
#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};
    use std::fmt::Debug;
    use std::marker::PhantomData;

    use crate::format_string;
    use crate::schema::FunctionSchemaMap;
    use crate::{
        error::{CommonError, PathError, PathErrorStep, RequestError, RequestResult},
        filter::Filter,
        ordering::{Ordering, OrderingDirection, OrderingTerm},
        parse::helpers::id_convert,
        query::{
            list::{ListQuery, ListQueryBuilder, ListQueryConfig},
            page_token::{plain::PlainPageTokenBuilder, FilterPageToken, PageTokenBuilder},
            search::{SearchQuery, SearchQueryBuilder, SearchQueryConfig},
        },
        testing::{bomboni, schema::UserItem},
    };
    use bomboni_common::{btree_map, btree_map_into, hash_map_into};
    use bomboni_proto::google::protobuf::{
        FloatValue, Int32Value, Int64Value, StringValue, Timestamp, UInt32Value, UInt64Value,
    };
    use bomboni_request_derive::{derived_map, parse_resource_name, Parse};
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
    #[repr(i32)]
    enum DataTypeEnum {
        #[default]
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
            default_default_nested: Option<NestedItem>,
            enum_value: i32,
            oneof: Option<OneofKind>,
            kept_nested: Option<NestedItem>,
            optional_box: Option<Box<i32>>,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItem {
            #[parse(source = "string")]
            s: String,
            #[parse(source = "optional_string?")]
            opt_s: Option<String>,
            required_string: String,
            #[parse(extract = [Unwrap])]
            required_string_optional: String,
            nested: ParsedNestedItem,
            optional_nested: Option<ParsedNestedItem>,
            #[parse(extract = [UnwrapOr(NestedItem::default())])]
            default_nested: ParsedNestedItem,
            #[parse(extract = [UnwrapOrDefault])]
            default_default_nested: ParsedNestedItem,
            #[parse(enumeration)]
            enum_value: DataTypeEnum,
            #[parse(oneof, extract = [Unwrap])]
            oneof: ParsedOneofKind,
            #[parse(keep_primitive)]
            kept_nested: Option<NestedItem>,
            #[parse(extract = [Unwrap, Unbox])]
            optional_box: Option<i32>,
            #[parse(skip)]
            extra: i32,
        }

        impl Default for Item {
            fn default() -> Self {
                Self {
                    string: "string".into(),
                    optional_string: Some("optional_string".into()),
                    required_string: "required_string".into(),
                    required_string_optional: Some("required_string_optional".into()),
                    nested: Some(NestedItem {}),
                    optional_nested: Some(NestedItem {}),
                    default_nested: Some(NestedItem {}),
                    default_default_nested: Some(NestedItem {}),
                    enum_value: 1,
                    oneof: Some(OneofKind::String("oneof".into())),
                    kept_nested: Some(NestedItem {}),
                    optional_box: Some(Box::new(1)),
                }
            }
        }

        impl Default for ParsedItem {
            fn default() -> Self {
                Self {
                    s: "string".into(),
                    opt_s: Some("optional_string".into()),
                    required_string: "required_string".into(),
                    required_string_optional: "required_string_optional".into(),
                    nested: ParsedNestedItem {},
                    optional_nested: Some(ParsedNestedItem {}),
                    default_nested: ParsedNestedItem {},
                    default_default_nested: ParsedNestedItem {},
                    enum_value: DataTypeEnum::String,
                    oneof: ParsedOneofKind::String("oneof".into()),
                    kept_nested: Some(NestedItem {}),
                    optional_box: Some(1),
                    extra: 0,
                }
            }
        }

        #[derive(Debug, PartialEq, Default)]
        struct NestedItem {}

        #[derive(Parse, Debug, Default, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = NestedItem, write)]
        struct ParsedNestedItem {}

        #[derive(Debug, PartialEq)]
        #[allow(dead_code)]
        enum OneofKind {
            String(String),
            Boolean(bool),
            Nested(NestedItem),
        }

        #[derive(Parse, Debug, PartialEq)]
        #[allow(dead_code)]
        #[parse(bomboni_crate = bomboni, source = OneofKind, write)]
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

        impl Default for OneofKind {
            fn default() -> Self {
                Self::Boolean(false)
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
                if let RequestError::Path(error) = err {
                    assert_eq!(
                        error.error.as_any().downcast_ref::<CommonError>().unwrap(),
                        $err
                    );
                    assert_eq!(error.path_to_string(), $field);
                } else {
                    panic!("expected PathError, got {:?}", err);
                }
            }};
        }

        assert_parse_field_err!(
            Item {
                string: String::default(),
                ..Default::default()
            },
            "string",
            &CommonError::RequiredFieldMissing
        );
        assert_parse_field_err!(
            Item {
                required_string: String::default(),
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
                required_string_optional: Some(String::default()),
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
                default_default_nested: None,
                enum_value: 1,
                oneof: Some(OneofKind::String("abc".into())),
                kept_nested: Some(NestedItem {}),
                optional_box: Some(Box::new(42)),
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
                default_default_nested: ParsedNestedItem {},
                enum_value: DataTypeEnum::String,
                oneof: ParsedOneofKind::String("abc".into()),
                kept_nested: Some(NestedItem {}),
                optional_box: Some(42),
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
                default_default_nested: ParsedNestedItem {},
                enum_value: DataTypeEnum::Boolean,
                oneof: ParsedOneofKind::String("abc".into()),
                kept_nested: Some(NestedItem {}),
                optional_box: Some(42),
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
                default_default_nested: Some(NestedItem {}),
                enum_value: 2,
                oneof: Some(OneofKind::String("abc".into())),
                kept_nested: Some(NestedItem {}),
                optional_box: Some(Box::new(42)),
            }
        );
    }

    #[test]
    fn extracts() {
        #[derive(Debug, PartialEq)]
        struct Item {
            value: i32,
            required_value: Option<i32>,
            optional_value: Option<i32>,
            inner_optional_value: Option<Box<Option<Box<i32>>>>,
            required_boxed_value: Option<Box<i32>>,
            optional_boxed_value: Option<Box<i32>>,
            default_value: Option<i32>,
            nested: Option<NestedItem>,
            source_box: Box<i32>,
            keep_box: Box<i32>,
            oneof: Option<Oneof>,
        }

        #[derive(Debug, PartialEq, Clone)]
        struct NestedItem {
            name: String,
            description: Option<String>,
        }

        #[derive(Debug, PartialEq)]
        struct Oneof {
            kind: Option<OneofKind>,
        }

        #[derive(Debug, PartialEq)]
        enum OneofKind {
            Value(i32),
        }

        impl OneofKind {
            pub fn get_variant_name(&self) -> &'static str {
                match self {
                    Self::Value(_) => "value",
                }
            }
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItem {
            value: i32,
            #[parse(try_from = i32, extract = [Unwrap])]
            required_value: i64,
            #[parse(try_from = i32)]
            optional_value: Option<i64>,
            #[parse(try_from = i32, extract = [Unwrap, Unbox, Unwrap, Unbox])]
            inner_optional_value: Option<i64>,
            #[parse(try_from = i32, extract = [Unwrap, Unbox])]
            required_boxed_value: Box<i64>,
            #[parse(try_from = i32, extract = [Unwrap, Unbox])]
            optional_boxed_value: Option<i64>,
            #[parse(extract = [UnwrapOr(42i32)])]
            default_value: i32,
            nested: ParsedNested,
            #[parse(source = "nested?.name")]
            name: String,
            #[parse(source = "nested?.description?")]
            description: Option<String>,
            #[parse(extract = [Unbox])]
            source_box: i32,
            keep_box: Box<i32>,
            #[parse(oneof, extract = [Unwrap])]
            oneof: ParsedOneof,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = NestedItem, write)]
        struct ParsedNested {
            name: String,
            description: Option<String>,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = Oneof, tagged_union { oneof = OneofKind, field = kind }, write)]
        enum ParsedOneof {
            Value(i32),
        }

        impl Default for Item {
            fn default() -> Self {
                Self {
                    value: 1,
                    required_value: Some(1),
                    optional_value: Some(1),
                    inner_optional_value: Some(Box::new(Some(Box::new(1)))),
                    required_boxed_value: Some(Box::new(1)),
                    optional_boxed_value: Some(Box::new(1)),
                    default_value: Some(1),
                    nested: Some(NestedItem::default()),
                    source_box: Box::new(1),
                    keep_box: Box::new(1),
                    oneof: Some(Oneof {
                        kind: Some(OneofKind::Value(1)),
                    }),
                }
            }
        }

        impl Default for NestedItem {
            fn default() -> Self {
                Self {
                    name: "abc".into(),
                    description: Some("abc".into()),
                }
            }
        }

        impl Default for ParsedItem {
            fn default() -> Self {
                Self {
                    value: 1,
                    required_value: 1,
                    optional_value: Some(1),
                    inner_optional_value: Some(1),
                    required_boxed_value: Box::new(1),
                    optional_boxed_value: Some(1),
                    default_value: 1,
                    nested: ParsedNested::default(),
                    name: "abc".into(),
                    description: Some("abc".into()),
                    source_box: 1,
                    keep_box: Box::new(1),
                    oneof: ParsedOneof::Value(1),
                }
            }
        }

        impl Default for ParsedNested {
            fn default() -> Self {
                Self {
                    name: "abc".into(),
                    description: Some("abc".into()),
                }
            }
        }

        assert_eq!(
            ParsedItem::parse(Item {
                value: 42,
                required_value: Some(42),
                optional_value: Some(42),
                inner_optional_value: Some(Box::new(Some(Box::new(42)))),
                required_boxed_value: Some(Box::new(42)),
                optional_boxed_value: Some(Box::new(42)),
                default_value: Some(42),
                nested: Some(NestedItem {
                    name: "name".into(),
                    description: Some("description".into()),
                }),
                source_box: Box::new(42),
                keep_box: Box::new(42),
                oneof: Some(Oneof {
                    kind: Some(OneofKind::Value(42)),
                }),
            })
            .unwrap(),
            ParsedItem {
                value: 42,
                optional_value: Some(42),
                required_value: 42,
                inner_optional_value: Some(42),
                required_boxed_value: Box::new(42),
                optional_boxed_value: Some(42),
                default_value: 42,
                nested: ParsedNested {
                    name: "name".into(),
                    description: Some("description".into()),
                },
                name: "name".into(),
                description: Some("description".into()),
                source_box: 42,
                keep_box: Box::new(42),
                oneof: ParsedOneof::Value(42),
            }
        );
        assert_eq!(
            Item::from(ParsedItem {
                value: 42,
                optional_value: Some(42),
                inner_optional_value: Some(42),
                required_value: 42,
                required_boxed_value: Box::new(42),
                optional_boxed_value: Some(42),
                default_value: 42,
                nested: ParsedNested {
                    name: "name".into(),
                    description: Some("description".into()),
                },
                name: "name".into(),
                description: Some("description".into()),
                source_box: 42,
                keep_box: Box::new(42),
                oneof: ParsedOneof::Value(1),
            }),
            Item {
                value: 42,
                optional_value: Some(42),
                inner_optional_value: Some(Box::new(Some(Box::new(42)))),
                required_value: Some(42),
                required_boxed_value: Some(Box::new(42)),
                optional_boxed_value: Some(Box::new(42)),
                default_value: Some(42),
                nested: Some(NestedItem {
                    name: "name".into(),
                    description: Some("description".into()),
                }),
                source_box: Box::new(42),
                keep_box: Box::new(42),
                oneof: Some(Oneof {
                    kind: Some(OneofKind::Value(1)),
                }),
            }
        );

        assert_eq!(
            ParsedItem::parse(Item {
                optional_value: None,
                ..Default::default()
            })
            .unwrap(),
            ParsedItem {
                optional_value: None,
                ..Default::default()
            }
        );
        assert_eq!(
            ParsedItem::parse(Item {
                optional_boxed_value: None,
                ..Default::default()
            })
            .unwrap(),
            ParsedItem {
                optional_boxed_value: None,
                ..Default::default()
            }
        );
        assert_eq!(
            ParsedItem::parse(Item {
                default_value: None,
                ..Default::default()
            })
            .unwrap(),
            ParsedItem {
                default_value: 42,
                ..Default::default()
            }
        );

        assert!(matches!(
            ParsedItem::parse(Item {
                required_value: None,
                ..Default::default()
            })
            .unwrap_err(),
            RequestError::Path(PathError { error, path, .. })
            if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::RequiredFieldMissing
            ) && path[0] == PathErrorStep::Field("required_value".into())
        ));
        assert!(matches!(
            ParsedItem::parse(Item {
                required_boxed_value: None,
                ..Default::default()
            })
            .unwrap_err(),
            RequestError::Path(PathError { error, path, .. })
            if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::RequiredFieldMissing
            ) && path[0] == PathErrorStep::Field("required_boxed_value".into())
        ));
    }

    #[test]
    fn parse_strings() {
        #[derive(Debug, PartialEq)]
        struct Item {
            required: String,
            possibly_empty: String,
            regex_validated: String,
            wrapped: StringValue,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItem {
            required: String,
            #[parse(extract = [StringFilterEmpty])]
            possibly_empty: Option<String>,
            #[parse(regex = "^[a-z]+$")]
            regex_validated: String,
            #[parse(wrapper, unspecified)]
            wrapped: String,
        }

        impl Default for Item {
            fn default() -> Self {
                Self {
                    required: "required".into(),
                    possibly_empty: "possibly_empty".into(),
                    regex_validated: "regex".into(),
                    wrapped: StringValue {
                        value: "wrapped".into(),
                    },
                }
            }
        }

        assert_eq!(
            ParsedItem::parse(Item::default()).unwrap(),
            ParsedItem {
                required: "required".into(),
                possibly_empty: Some("possibly_empty".into()),
                regex_validated: "regex".into(),
                wrapped: "wrapped".into(),
            }
        );

        assert!(matches!(
            ParsedItem::parse(Item {
                required: String::default(),
                ..Default::default()
            })
            .unwrap_err(),
            RequestError::Path(PathError {
                error, path, ..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::RequiredFieldMissing
            ) && path[0] == PathErrorStep::Field("required".into())
        ));
        assert!(matches!(
            ParsedItem::parse(Item {
                regex_validated: "123".into(),
                ..Default::default()
            })
            .unwrap_err(),
            RequestError::Path(PathError {
                error, path, ..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::InvalidStringFormat { .. }
            ) && path[0] == PathErrorStep::Field("regex_validated".into())
        ));
    }

    #[test]
    fn parse_enumerations() {
        #[derive(Debug, PartialEq, Default)]
        struct Item {
            required: i32,
            optional: i32,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItem {
            #[parse(enumeration)]
            required: DataTypeEnum,
            #[parse(enumeration, extract = [EnumerationFilterUnspecified])]
            optional: Option<DataTypeEnum>,
        }

        assert_eq!(
            ParsedItem::parse(Item {
                required: 1,
                optional: 0,
            })
            .unwrap(),
            ParsedItem {
                required: DataTypeEnum::String,
                optional: None,
            }
        );
        assert_eq!(
            Item::from(ParsedItem {
                required: DataTypeEnum::String,
                optional: None,
            }),
            Item {
                required: 1,
                optional: 0,
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
            nullable: Option<Vec<i32>>,
        }

        #[derive(Debug, PartialEq)]
        struct Oneof {
            kind: Option<OneofKind>,
        }

        #[derive(Debug, PartialEq)]
        enum OneofKind {
            Derived(String),
            BorrowedDerived(String),
            Extracted(Option<String>),
            BorrowedExtracted(Option<String>),
        }

        impl OneofKind {
            pub fn get_variant_name(&self) -> &'static str {
                match self {
                    Self::Derived(_) => "derived",
                    Self::BorrowedDerived(_) => "borrowed_derived",
                    Self::Extracted(_) => "extracted",
                    Self::BorrowedExtracted(_) => "borrowed_extracted",
                }
            }
        }

        #[derive(Parse, Debug, PartialEq, Default)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItem {
            #[parse(derive { parse = pos_parse, write = pos_write })]
            pos: String,
            name: String,
            #[parse(source = "name", derive { parse = id_parse, write = id_write })]
            id: u64,
            #[parse(source_field, derive = nullable_derive)]
            nullable: Option<Vec<i32>>,
        }

        #[allow(clippy::unnecessary_wraps)]
        fn pos_parse(item: &Item) -> RequestResult<String> {
            Ok(format_string!("{}, {}", item.x, item.y))
        }

        fn pos_write(target: &ParsedItem, source: &mut Item) {
            let parts: Vec<&str> = target.pos.split(", ").collect();
            source.x = parts[0].parse().unwrap();
            source.y = parts[1].parse().unwrap();
        }

        #[allow(clippy::unnecessary_wraps)]
        fn id_parse(id: String) -> RequestResult<u64> {
            if id.is_empty() {
                return Err(CommonError::RequiredFieldMissing.into());
            }
            Ok(id.parse().unwrap())
        }

        fn id_write(id: u64) -> String {
            format_string!("{}", id)
        }

        mod nullable_derive {
            use super::*;

            #[allow(clippy::unnecessary_wraps)]
            pub fn parse(value: Option<Vec<i32>>) -> RequestResult<Option<Vec<i32>>> {
                Ok(value.filter(|v| !v.is_empty()))
            }

            pub fn write(value: Option<Vec<i32>>) -> Option<Vec<i32>> {
                value.filter(|v| !v.is_empty())
            }
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(bomboni_crate = bomboni, source = Oneof, tagged_union { oneof = OneofKind, field = kind }, write)]
        enum ParsedOneof {
            #[parse(derive { parse = derived_oneof_parse, write = derived_oneof_write })]
            Derived(i32),
            #[parse(derive { parse = borrowed_derived_oneof_parse, write = borrowed_derived_oneof_write, source_borrow, target_borrow })]
            BorrowedDerived(i32),
            #[parse(extract = [Unwrap], derive { parse = extracted_oneof_parse, write = extracted_oneof_write })]
            Extracted(i32),
            #[parse(extract = [Unwrap], derive { parse = borrowed_extracted_parse, write = borrowed_extracted_write, source_borrow, target_borrow })]
            BorrowedExtracted(i32),
        }

        #[allow(clippy::unnecessary_wraps)]
        fn derived_oneof_parse(value: String) -> RequestResult<i32> {
            Ok(value
                .parse()
                .map_err(|_| CommonError::InvalidNumericValue)?)
        }

        fn derived_oneof_write(value: i32) -> String {
            format_string!("{}", value)
        }

        fn borrowed_derived_oneof_parse(oneof: &OneofKind) -> Option<RequestResult<ParsedOneof>> {
            match oneof {
                OneofKind::BorrowedDerived(value) => {
                    Some(Ok(ParsedOneof::BorrowedDerived(match value.parse() {
                        Ok(value) => value,
                        Err(_) => return Some(Err(CommonError::InvalidNumericValue.into())),
                    })))
                }
                _ => None,
            }
        }

        fn borrowed_derived_oneof_write(oneof: &ParsedOneof) -> Option<Oneof> {
            match oneof {
                ParsedOneof::BorrowedDerived(value) => Some(Oneof {
                    kind: Some(OneofKind::BorrowedDerived(format_string!("{}", value))),
                }),
                _ => None,
            }
        }

        #[allow(clippy::unnecessary_wraps)]
        fn extracted_oneof_parse(value: String) -> RequestResult<i32> {
            Ok(value
                .parse()
                .map_err(|_| CommonError::InvalidNumericValue)?)
        }

        #[allow(clippy::unnecessary_wraps)]
        fn extracted_oneof_write(value: i32) -> Option<String> {
            Some(format_string!("{}", value))
        }

        fn borrowed_extracted_parse(source: &OneofKind) -> Option<RequestResult<ParsedOneof>> {
            match source {
                OneofKind::BorrowedExtracted(value) => {
                    match value
                        .as_ref()
                        .ok_or(CommonError::RequiredFieldMissing)
                        .and_then(|value| {
                            value.parse().map_err(|_| CommonError::InvalidNumericValue)
                        }) {
                        Ok(value) => Some(Ok(ParsedOneof::BorrowedExtracted(value))),
                        Err(err) => Some(Err(err.into())),
                    }
                }
                _ => None,
            }
        }

        fn borrowed_extracted_write(oneof: &ParsedOneof) -> Option<Oneof> {
            match oneof {
                ParsedOneof::BorrowedExtracted(value) => Some(Oneof {
                    kind: Some(OneofKind::BorrowedExtracted(Some(format_string!(
                        "{}", value
                    )))),
                }),
                _ => None,
            }
        }

        assert_eq!(
            ParsedItem::parse(Item {
                x: 1,
                y: 2,
                name: "42".into(),
                nullable: Some(vec![1, 2]),
            })
            .unwrap(),
            ParsedItem {
                pos: "1, 2".into(),
                id: 42,
                name: "42".into(),
                nullable: Some(vec![1, 2]),
            }
        );
        assert_eq!(
            Item::from(ParsedItem {
                pos: "1, 2".into(),
                id: 42,
                name: "42".into(),
                nullable: Some(Vec::new()),
            }),
            Item {
                x: 1,
                y: 2,
                name: "42".into(),
                nullable: None,
            }
        );

        assert!(matches!(
            ParsedItem::parse(Item {
                name: String::default(),
                ..Default::default()
            }).unwrap_err(),
            RequestError::Path(PathError {
                error,
                path,
                ..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::RequiredFieldMissing
            ) && path[0] == PathErrorStep::Field("name".into())
        ));

        assert_eq!(
            ParsedOneof::parse(Oneof {
                kind: Some(OneofKind::Derived("42".into())),
            })
            .unwrap(),
            ParsedOneof::Derived(42i32)
        );
        assert_eq!(
            Oneof::from(ParsedOneof::Derived(42i32)),
            Oneof {
                kind: Some(OneofKind::Derived("42".into())),
            }
        );
        assert!(matches!(
            ParsedOneof::parse(Oneof {
                kind: Some(OneofKind::Derived("f".into())),
            })
            .unwrap_err(),
            RequestError::Path(PathError {
                error,
                path,
                ..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::InvalidNumericValue
            ) && path[0] == PathErrorStep::Field("derived".into())
        ));

        assert_eq!(
            ParsedOneof::parse(Oneof {
                kind: Some(OneofKind::BorrowedDerived("42".into())),
            })
            .unwrap(),
            ParsedOneof::BorrowedDerived(42i32)
        );
        assert_eq!(
            Oneof::from(ParsedOneof::BorrowedDerived(42i32)),
            Oneof {
                kind: Some(OneofKind::BorrowedDerived("42".into())),
            }
        );

        assert_eq!(
            ParsedOneof::parse(Oneof {
                kind: Some(OneofKind::Extracted(Some("42".into()))),
            })
            .unwrap(),
            ParsedOneof::Extracted(42i32)
        );
        assert_eq!(
            Oneof::from(ParsedOneof::Extracted(42i32)),
            Oneof {
                kind: Some(OneofKind::Extracted(Some("42".into()))),
            }
        );
        assert!(matches!(
            ParsedOneof::parse(Oneof {
                kind: Some(OneofKind::Extracted(None)),
            })
            .unwrap_err(),
            RequestError::Path(PathError {
                error,
                path,
                ..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::RequiredFieldMissing
            ) && path[0] == PathErrorStep::Field("extracted".into())
        ));

        assert_eq!(
            ParsedOneof::parse(Oneof {
                kind: Some(OneofKind::BorrowedExtracted(Some("42".into()))),
            })
            .unwrap(),
            ParsedOneof::BorrowedExtracted(42i32)
        );
        assert_eq!(
            Oneof::from(ParsedOneof::BorrowedExtracted(42i32)),
            Oneof {
                kind: Some(OneofKind::BorrowedExtracted(Some("42".into()))),
            }
        );
        assert!(matches!(
            ParsedOneof::parse(Oneof {
                kind: Some(OneofKind::BorrowedExtracted(None)),
            })
            .unwrap_err(),
            RequestError::Path(PathError {
                error,
                path,
                ..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::RequiredFieldMissing
            ) && path[0] == PathErrorStep::Field("borrowed_extracted".into())
        ));
    }

    #[test]
    fn parse_collections() {
        #[derive(Debug, PartialEq)]
        struct Item {
            values: Vec<i32>,
            strings: Vec<String>,
            items: Vec<NestedItem>,
            values_map: BTreeMap<String, i32>,
            items_map: HashMap<i32, NestedItem>,
            enums: Vec<i32>,
            enum_map: BTreeMap<i32, i32>,
        }

        #[derive(Debug, PartialEq, Default)]
        struct NestedItem {
            value: i32,
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItem {
            values: Vec<i32>,
            #[parse(regex = "^[a-z]$")]
            strings: Vec<String>,
            items: Vec<ParsedNestedItem>,
            values_map: BTreeMap<String, i32>,
            items_map: HashMap<i32, ParsedNestedItem>,
            #[parse(enumeration)]
            enums: Vec<DataTypeEnum>,
            #[parse(enumeration)]
            enum_map: BTreeMap<i32, DataTypeEnum>,
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(bomboni_crate = bomboni, source = NestedItem, write)]
        struct ParsedNestedItem {
            value: i32,
        }

        impl Default for Item {
            fn default() -> Self {
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
                    enum_map: btree_map! {
                        1 => 1,
                    },
                }
            }
        }

        impl Default for ParsedItem {
            fn default() -> Self {
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
                    enum_map: btree_map! {
                        1 => DataTypeEnum::String,
                    },
                }
            }
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
                enum_map: btree_map_into! {
                    1 => 1,
                },
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
                enum_map: btree_map_into! {
                    1 => DataTypeEnum::String,
                },
            }
        );

        assert!(matches!(
            ParsedItem::parse(Item {
                strings: vec!["Hello".into()],
                ..Default::default()
            }).unwrap_err(),
            RequestError::Path(error) if matches!(
                error.error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::InvalidStringFormat { .. }
            ) && error.path_to_string() == "strings[0]"
        ));
        assert!(matches!(ParsedItem::parse(Item {
                enums: vec![99i32],
                ..Default::default()
            })
            .unwrap_err(),
            RequestError::Path(error) if matches!(
                error.error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::InvalidEnumValue
            ) && error.path_to_string() == "enums[0]"
        ));
        assert!(matches!(
            ParsedItem::parse(Item {
                enum_map: btree_map_into! {
                    99 => 99,
                },
                ..Default::default()
            })
            .unwrap_err(),
            RequestError::Path(error) if matches!(
                error.error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::InvalidEnumValue
            ) && error.path_to_string() == "enum_map{99}"
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
                enum_map: btree_map_into! {
                    1 => DataTypeEnum::String,
                },
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
                enum_map: btree_map_into! {
                    1 => 1,
                },
            }
        );
    }

    #[test]
    fn wrapper_types() {
        #[derive(Debug, PartialEq, Default)]
        struct Item {
            value_f32: Option<FloatValue>,
            integer_16: Int32Value,
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItem {
            #[parse(wrapper)]
            value_f32: Option<f32>,
            #[parse(wrapper)]
            integer_16: i16,
        }

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
            USize(UInt64Value),
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
        #[parse(bomboni_crate = bomboni, source = Value, tagged_union { oneof = ValueKind, field = kind }, write)]
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

        assert_eq!(
            ParsedItem::parse(Item {
                value_f32: Some(FloatValue { value: 42.0 }),
                integer_16: Int32Value { value: 42 },
            })
            .unwrap(),
            ParsedItem {
                value_f32: Some(42.0),
                integer_16: 42,
            }
        );
        assert_eq!(
            Item::from(ParsedItem {
                value_f32: Some(42.0),
                integer_16: 42,
            }),
            Item {
                value_f32: Some(FloatValue { value: 42.0 }),
                integer_16: Int32Value { value: 42 },
            }
        );
    }

    #[test]
    fn parse_oneof() {
        #[derive(Debug, Clone, PartialEq)]
        enum Item {
            String(String),
            Data(Data),
            DataValue(Data),
            Null(()),
            Empty,
            Dropped(i32),
        }

        impl Item {
            pub fn get_variant_name(&self) -> &'static str {
                match self {
                    Self::String(_) => "string",
                    Self::Data(_) => "data",
                    Self::DataValue(_) => "data_value",
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
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        enum ParsedItem {
            String(String),
            Data(ParsedData),
            #[parse(extract = [Field("value")])]
            DataValue(i32),
            Null(()),
            Empty,
            #[parse(source_unit)]
            Dropped,
        }

        #[derive(Debug, Clone, Default, PartialEq, Parse)]
        #[parse(bomboni_crate = bomboni, source = Data, write)]
        struct ParsedData {
            value: i32,
        }

        #[derive(Debug, Clone, PartialEq, Default)]
        struct NestedItem {
            item: Option<Item>,
        }

        #[derive(Debug, Clone, PartialEq)]
        struct Container {
            item: Item,
            option_item: Option<Item>,
            nested_item: Option<NestedItem>,
        }

        impl Default for Container {
            fn default() -> Self {
                Self {
                    item: Item::String("item".into()),
                    option_item: Some(Item::String("option_item".into())),
                    nested_item: Some(NestedItem {
                        item: Some(Item::String("nested_item".into())),
                    }),
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Parse)]
        #[parse(bomboni_crate = bomboni, source = Container, write)]
        struct ParsedContainer {
            #[parse(oneof)]
            item: ParsedItem,
            #[parse(oneof, extract = [Unwrap])]
            option_item: ParsedItem,
            #[parse(source = "nested_item?.item?", oneof)]
            nested_item: ParsedItem,
        }

        assert_eq!(
            ParsedContainer::parse(Container {
                item: Item::Data(Data { value: 42 }),
                option_item: Some(Item::DataValue(Data { value: 42 })),
                nested_item: Some(NestedItem {
                    item: Some(Item::Data(Data { value: 42 })),
                }),
            })
            .unwrap(),
            ParsedContainer {
                item: ParsedItem::Data(ParsedData { value: 42 }),
                option_item: ParsedItem::DataValue(42),
                nested_item: ParsedItem::Data(ParsedData { value: 42 }),
            }
        );
        assert_eq!(
            Container::from(ParsedContainer {
                item: ParsedItem::Data(ParsedData { value: 42 }),
                option_item: ParsedItem::DataValue(42),
                nested_item: ParsedItem::Data(ParsedData { value: 42 }),
            }),
            Container {
                item: Item::Data(Data { value: 42 }),
                option_item: Some(Item::DataValue(Data { value: 42 })),
                nested_item: Some(NestedItem {
                    item: Some(Item::Data(Data { value: 42 })),
                }),
            }
        );

        assert!(matches!(
            ParsedContainer::parse(Container {
                nested_item: None,
                ..Default::default()
            })
            .unwrap_err(),
            RequestError::Path(PathError {
                error,
                path,
                ..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::RequiredFieldMissing
            ) && path[0] == PathErrorStep::Field("nested_item".into())
        ));

        assert_eq!(
            ParsedItem::parse(Item::String("abc".into())).unwrap(),
            ParsedItem::String("abc".into())
        );
        assert_eq!(
            Item::from(ParsedItem::String("abc".into())),
            Item::String("abc".into())
        );

        assert!(matches!(
            ParsedItem::parse(Item::String(String::default())).unwrap_err(),
            RequestError::Path(PathError {
                error,
                path,
                ..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::RequiredFieldMissing
            ) && path[0] == PathErrorStep::Field("string".into())
        ));

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
    fn parse_tagged_union() {
        #[derive(Debug, Clone, PartialEq, Default)]
        struct Value {
            kind: Option<ValueKind>,
        }

        #[derive(Debug, Clone, PartialEq)]
        enum ValueKind {
            Number(i32),
            Inner(Box<Value>),
            Nested(NestedValue),
            OptionalNested(Option<NestedValue>),
            DefaultNested(Option<NestedValue>),
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
                    Self::OptionalNested(_) => "optional_nested",
                    Self::DefaultNested(_) => "default_nested",
                }
            }
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(bomboni_crate = bomboni, source = Value, tagged_union { oneof = ValueKind, field = kind }, write)]
        enum ParsedValue {
            Number(i32),
            #[parse(extract = [Unbox])]
            Inner(Box<ParsedValue>),
            Nested(ParsedNestedValue),
            OptionalNested(Option<ParsedNestedValue>),
            #[parse(extract = [UnwrapOrDefault])]
            DefaultNested(ParsedNestedValue),
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(bomboni_crate = bomboni, source = NestedValue, write)]
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

        assert_eq!(
            ParsedValue::parse(Value {
                kind: Some(ValueKind::OptionalNested(Some(NestedValue { value: 42 }))),
            })
            .unwrap(),
            ParsedValue::OptionalNested(Some(ParsedNestedValue { value: 42 }))
        );
        assert_eq!(
            Value::from(ParsedValue::OptionalNested(Some(ParsedNestedValue {
                value: 42
            }))),
            Value {
                kind: Some(ValueKind::OptionalNested(Some(NestedValue { value: 42 }))),
            }
        );
        assert_eq!(
            ParsedValue::parse(Value {
                kind: Some(ValueKind::OptionalNested(None)),
            })
            .unwrap(),
            ParsedValue::OptionalNested(None)
        );
        assert_eq!(
            Value::from(ParsedValue::OptionalNested(None)),
            Value {
                kind: Some(ValueKind::OptionalNested(None)),
            }
        );
    }

    #[test]
    fn parse_keep() {
        #[derive(Debug, Clone, PartialEq, Default)]
        struct Item {
            value: i32,
            item: Option<NestedItem>,
            item_primitive: Option<NestedItem>,
        }

        #[derive(Debug, Clone, PartialEq, Default)]
        struct NestedItem {
            value: i32,
        }

        #[derive(Debug, Clone, PartialEq)]
        enum Oneof {
            Item(NestedItem),
            Vec(Vec<NestedItem>),
        }

        impl Oneof {
            pub fn get_variant_name(&self) -> &'static str {
                match self {
                    Self::Item(_) => "item",
                    Self::Vec(_) => "vec",
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Default, Parse)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItem {
            #[parse(keep, source = "value")]
            x: i32,
            #[parse(keep_primitive)]
            item: Option<NestedItem>,
            #[parse(keep_primitive, extract = [Unwrap])]
            item_primitive: NestedItem,
        }

        #[derive(Debug, Clone, PartialEq, Parse)]
        #[parse(bomboni_crate = bomboni, source = Oneof, write)]
        enum ParsedOneof {
            #[parse(keep)]
            Item(NestedItem),
            #[parse(keep_primitive)]
            Vec(Vec<NestedItem>),
        }

        assert_eq!(
            ParsedItem::parse(Item {
                value: 42,
                item: Some(NestedItem { value: 24 }),
                item_primitive: Some(NestedItem { value: 24 }),
            })
            .unwrap(),
            ParsedItem {
                x: 42,
                item: Some(NestedItem { value: 24 }),
                item_primitive: NestedItem { value: 24 },
            }
        );
        assert_eq!(
            Item::from(ParsedItem {
                x: 42,
                item: Some(NestedItem { value: 24 }),
                item_primitive: NestedItem { value: 24 },
            }),
            Item {
                value: 42,
                item: Some(NestedItem { value: 24 }),
                item_primitive: Some(NestedItem { value: 24 }),
            }
        );

        assert_eq!(
            ParsedOneof::parse(Oneof::Item(NestedItem { value: 42 })).unwrap(),
            ParsedOneof::Item(NestedItem { value: 42 })
        );
        assert_eq!(
            Oneof::from(ParsedOneof::Item(NestedItem { value: 42 })),
            Oneof::Item(NestedItem { value: 42 })
        );
        assert_eq!(
            ParsedOneof::parse(Oneof::Vec(vec![NestedItem { value: 42 }])).unwrap(),
            ParsedOneof::Vec(vec![NestedItem { value: 42 }])
        );
        assert_eq!(
            Oneof::from(ParsedOneof::Vec(vec![NestedItem { value: 42 }])),
            Oneof::Vec(vec![NestedItem { value: 42 }])
        );
    }

    #[test]
    fn parse_generics() {
        #[derive(Debug, Clone, PartialEq, Default)]
        struct Item<T: Debug + Clone> {
            value: T,
        }

        #[derive(Debug, Clone, PartialEq, Default, Parse)]
        #[parse(bomboni_crate = bomboni, source = Item::<TSource>, write)]
        struct ParsedItem<T, TSource, S = String>
        where
            T: Default + Debug + Clone + Into<TSource>,
            TSource: Default + Debug + Clone + RequestParseInto<T>,
            S: Default + Debug + Clone,
        {
            value: T,
            #[parse(skip)]
            skipped: S,
            _ts: PhantomData<TSource>,
        }

        #[derive(Debug, Clone, PartialEq, Default, Parse)]
        #[parse(bomboni_crate = bomboni, source = Item::<i32>, write)]
        struct ParsedItemI32<T>
        where
            T: Default + Debug + Clone + RequestParse<i32> + Into<i32>,
        {
            value: T,
        }

        impl RequestParse<i32> for i32 {
            fn parse(value: i32) -> RequestResult<Self> {
                Ok(value)
            }
        }

        assert_eq!(
            ParsedItem::<i32, i32, String>::parse(Item { value: 42 }).unwrap(),
            ParsedItem::<i32, i32, String> {
                value: 42,
                skipped: String::default(),
                _ts: PhantomData,
            }
        );
        assert_eq!(
            Item::from(ParsedItem::<i32, i32, String> {
                value: 42,
                skipped: String::default(),
                _ts: PhantomData,
            }),
            Item { value: 42 }
        );

        assert_eq!(
            ParsedItemI32::<i32>::parse(Item { value: 42 }).unwrap(),
            ParsedItemI32::<i32> { value: 42 }
        );
    }

    #[test]
    fn parse_query() {
        #[derive(Debug, PartialEq, Default, Clone)]
        struct Item {
            query: String,
            page_size: Option<u32>,
            page_token: Option<String>,
            filter: Option<String>,
            order_by: Option<String>,
            order: Option<String>,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedListQuery {
            #[parse(list_query)]
            list_query: ListQuery,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedNoFilter {
            #[parse(list_query { filter = false })]
            query: ListQuery,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedCustomToken {
            #[parse(list_query)]
            query: ListQuery<u64>,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedSearchQuery {
            #[parse(search_query)]
            search_query: SearchQuery,
        }

        fn get_list_query_builder() -> &'static ListQueryBuilder<PlainPageTokenBuilder> {
            use std::sync::OnceLock;
            static SINGLETON: OnceLock<ListQueryBuilder<PlainPageTokenBuilder>> = OnceLock::new();
            SINGLETON.get_or_init(|| {
                ListQueryBuilder::<PlainPageTokenBuilder>::new(
                    UserItem::get_schema(),
                    FunctionSchemaMap::new(),
                    ListQueryConfig {
                        max_page_size: Some(20),
                        default_page_size: 10,
                        primary_ordering_term: Some(OrderingTerm {
                            name: "id".into(),
                            direction: OrderingDirection::Descending,
                        }),
                        max_filter_length: Some(50),
                        max_ordering_length: Some(50),
                    },
                    PlainPageTokenBuilder {},
                )
            })
        }

        fn get_search_query_builder() -> &'static SearchQueryBuilder<PlainPageTokenBuilder> {
            use std::sync::OnceLock;
            static SINGLETON: OnceLock<SearchQueryBuilder<PlainPageTokenBuilder>> = OnceLock::new();
            SINGLETON.get_or_init(|| {
                SearchQueryBuilder::<PlainPageTokenBuilder>::new(
                    UserItem::get_schema(),
                    FunctionSchemaMap::new(),
                    SearchQueryConfig {
                        max_query_length: Some(50),
                        max_page_size: Some(20),
                        default_page_size: 10,
                        primary_ordering_term: Some(OrderingTerm {
                            name: "id".into(),
                            direction: OrderingDirection::Descending,
                        }),
                        max_filter_length: Some(50),
                        max_ordering_length: Some(50),
                    },
                    PlainPageTokenBuilder {},
                )
            })
        }

        #[derive(Clone)]
        struct CustomPageTokenBuilder {}

        impl PageTokenBuilder for CustomPageTokenBuilder {
            type PageToken = u64;

            fn parse(
                &self,
                _filter: &Filter,
                _ordering: &Ordering,
                _salt: &[u8],
                page_token: &str,
            ) -> crate::query::error::QueryResult<Self::PageToken> {
                Ok(page_token.parse().unwrap())
            }

            fn build_next<T: crate::schema::SchemaMapped>(
                &self,
                _filter: &Filter,
                _ordering: &Ordering,
                _salt: &[u8],
                _next_item: &T,
            ) -> crate::query::error::QueryResult<String> {
                Ok("24".into())
            }
        }

        let item = Item {
            query: "hello".into(),
            page_size: Some(42),
            page_token: Some("true".into()),
            filter: Some("true".into()),
            order_by: Some("id".into()),
            order: Some("id desc".into()),
        };

        assert_eq!(
            ParsedListQuery::parse_list_query(item.clone(), get_list_query_builder()).unwrap(),
            ParsedListQuery {
                list_query: ListQuery {
                    page_size: 20,
                    page_token: Some(FilterPageToken {
                        filter: Filter::parse("true").unwrap(),
                    }),
                    filter: Filter::parse("true").unwrap(),
                    ordering: Ordering::new(vec![OrderingTerm {
                        name: "id".into(),
                        direction: OrderingDirection::Ascending,
                    }])
                },
            },
        );
        assert_eq!(
            Item::from(ParsedListQuery {
                list_query: ListQuery {
                    page_size: 20,
                    page_token: Some(FilterPageToken {
                        filter: Filter::parse("true").unwrap(),
                    }),
                    filter: Filter::parse("true").unwrap(),
                    ordering: Ordering::new(vec![OrderingTerm {
                        name: "id".into(),
                        direction: OrderingDirection::Ascending,
                    }])
                },
            }),
            Item {
                query: String::default(),
                page_size: Some(20),
                page_token: Some("true".into()),
                filter: Some("true".into()),
                order_by: Some("id asc".into()),
                order: None,
            },
        );

        assert_eq!(
            ParsedNoFilter::parse_list_query(item.clone(), get_list_query_builder()).unwrap(),
            ParsedNoFilter {
                query: ListQuery {
                    page_size: 20,
                    page_token: Some(FilterPageToken {
                        filter: Filter::parse("true").unwrap(),
                    }),
                    filter: Filter::default(),
                    ordering: Ordering::new(vec![OrderingTerm {
                        name: "id".into(),
                        direction: OrderingDirection::Ascending,
                    }])
                },
            },
        );
        assert_eq!(
            Item::from(ParsedNoFilter {
                query: ListQuery {
                    page_size: 20,
                    page_token: Some(FilterPageToken {
                        filter: Filter::parse("true").unwrap(),
                    }),
                    filter: Filter::default(),
                    ordering: Ordering::new(vec![OrderingTerm {
                        name: "id".into(),
                        direction: OrderingDirection::Ascending,
                    }])
                },
            }),
            Item {
                query: String::default(),
                page_size: Some(20),
                page_token: Some("true".into()),
                filter: None,
                order_by: Some("id asc".into()),
                order: None,
            },
        );

        assert_eq!(
            ParsedCustomToken::parse_list_query(
                Item {
                    page_token: Some("42".into()),
                    ..item.clone()
                },
                &ListQueryBuilder::<CustomPageTokenBuilder>::new(
                    UserItem::get_schema(),
                    FunctionSchemaMap::new(),
                    ListQueryConfig {
                        max_page_size: Some(20),
                        default_page_size: 10,
                        primary_ordering_term: Some(OrderingTerm {
                            name: "id".into(),
                            direction: OrderingDirection::Descending,
                        }),
                        max_filter_length: Some(50),
                        max_ordering_length: Some(50),
                    },
                    CustomPageTokenBuilder {},
                )
            )
            .unwrap(),
            ParsedCustomToken {
                query: ListQuery {
                    page_size: 20,
                    page_token: Some(42),
                    filter: Filter::parse("true").unwrap(),
                    ordering: Ordering::new(vec![OrderingTerm {
                        name: "id".into(),
                        direction: OrderingDirection::Ascending,
                    }])
                },
            },
        );
        assert_eq!(
            Item::from(ParsedCustomToken {
                query: ListQuery {
                    page_size: 20,
                    page_token: Some(42),
                    filter: Filter::parse("true").unwrap(),
                    ordering: Ordering::new(vec![OrderingTerm {
                        name: "id".into(),
                        direction: OrderingDirection::Ascending,
                    }])
                },
            }),
            Item {
                query: String::default(),
                page_size: Some(20),
                page_token: Some("42".into()),
                filter: Some("true".into()),
                order_by: Some("id asc".into()),
                order: None,
            },
        );

        assert_eq!(
            ParsedSearchQuery::parse_search_query(item.clone(), get_search_query_builder())
                .unwrap(),
            ParsedSearchQuery {
                search_query: SearchQuery {
                    query: "hello".into(),
                    page_size: 20,
                    page_token: Some(FilterPageToken {
                        filter: Filter::parse("true").unwrap(),
                    }),
                    filter: Filter::parse("true").unwrap(),
                    ordering: Ordering::new(vec![OrderingTerm {
                        name: "id".into(),
                        direction: OrderingDirection::Ascending,
                    }])
                },
            },
        );
        assert_eq!(
            Item::from(ParsedSearchQuery {
                search_query: SearchQuery {
                    query: "hello".into(),
                    page_size: 20,
                    page_token: Some(FilterPageToken {
                        filter: Filter::parse("true").unwrap(),
                    }),
                    filter: Filter::parse("true").unwrap(),
                    ordering: Ordering::new(vec![OrderingTerm {
                        name: "id".into(),
                        direction: OrderingDirection::Ascending,
                    }])
                },
            }),
            Item {
                query: "hello".into(),
                page_size: Some(20),
                page_token: Some("true".into()),
                filter: Some("true".into()),
                order_by: Some("id asc".into()),
                order: None,
            },
        );
    }

    #[test]
    fn wrap_request_message() {
        #[derive(Debug, Clone, PartialEq, Default)]
        struct Request {
            value: Option<i32>,
        }

        impl Request {
            pub const NAME: &'static str = "Request";
        }

        #[derive(Debug, Clone, PartialEq, Default, Parse)]
        #[parse(bomboni_crate = bomboni, source = Request, request, write)]
        struct ParsedRequest {
            #[parse(source = "value?")]
            value: i32,
        }

        #[derive(Debug, Clone, PartialEq, Default, Parse)]
        #[parse(bomboni_crate = bomboni, source = Request, request { name = "Test" }, write)]
        struct ParsedCustomNameRequest {
            #[parse(source = "value?")]
            value: i32,
        }

        assert!(matches!(
            ParsedRequest::parse(Request { value: None }).unwrap_err(),
            RequestError::BadRequest { name, violations }
            if name == Request::NAME && matches!(
                violations.first().unwrap(),
                error if matches!(
                    error.error.as_any().downcast_ref::<CommonError>().unwrap(),
                    CommonError::RequiredFieldMissing { .. }
                ) && error.path_to_string() == "value"
            )
        ));

        assert!(matches!(
            ParsedCustomNameRequest::parse(Request { value: None }).unwrap_err(),
            RequestError::BadRequest { name, violations }
            if name == "Test" && matches!(
                violations.first().unwrap(),
                error if matches!(
                    error.error.as_any().downcast_ref::<CommonError>().unwrap(),
                    CommonError::RequiredFieldMissing { .. }
                ) && error.path_to_string() == "value"
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
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItem {
            #[parse(resource {
                fields {
                    name = true,
                    update_time {
                        source = update_time,
                        write = false,
                    }
                }
            })]
            resource: ParsedResource,
        }

        #[derive(Parse, Debug, PartialEq)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItemDefaultResource {
            #[parse(resource)]
            resource: ParsedResource,
        }

        assert_eq!(
            ParsedItem::parse(Item {
                name: "items/42".into(),
                create_time: Some(UtcDateTime::UNIX_EPOCH.into()),
                deleted: true,
                etag: Some("abc".into()),
                ..Default::default()
            })
            .unwrap(),
            ParsedItem {
                resource: ParsedResource {
                    name: "items/42".into(),
                    create_time: Some(UtcDateTime::UNIX_EPOCH),
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
                    create_time: Some(UtcDateTime::UNIX_EPOCH),
                    deleted: true,
                    ..Default::default()
                },
            }),
            Item {
                name: "items/42".into(),
                create_time: Some(UtcDateTime::UNIX_EPOCH.into()),
                deleted: true,
                ..Default::default()
            }
        );

        assert_eq!(
            ParsedItemDefaultResource::parse(Item {
                name: "items/42".into(),
                create_time: Some(UtcDateTime::UNIX_EPOCH.into()),
                deleted: true,
                ..Default::default()
            })
            .unwrap(),
            ParsedItemDefaultResource {
                resource: ParsedResource {
                    name: "items/42".into(),
                    create_time: Some(UtcDateTime::UNIX_EPOCH),
                    deleted: true,
                    ..Default::default()
                }
            }
        );
    }

    #[test]
    fn parse_convert() {
        #[derive(Debug, PartialEq, Default)]
        struct Item {
            value: i32,
            id: Option<String>,
            nested: Vec<i32>,
        }

        #[derive(Debug, PartialEq, Parse)]
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItem {
            #[parse(try_from = i32)]
            value: u64,
            #[parse(convert = id_convert)]
            id: Option<Id>,
            #[parse(convert { parse = parse_nested, write = write_nested })]
            nested: ParsedNestedItem,
        }

        #[derive(Debug, PartialEq)]
        struct ParsedNestedItem {
            values: Vec<i32>,
        }

        #[allow(clippy::unnecessary_wraps)]
        pub fn parse_nested(item: Vec<i32>) -> RequestResult<ParsedNestedItem> {
            Ok(ParsedNestedItem { values: item })
        }

        pub fn write_nested(item: ParsedNestedItem) -> Vec<i32> {
            item.values
        }

        assert_eq!(
            ParsedItem::parse(Item {
                value: 42,
                id: Some("0000000000000000000000000000002A".into()),
                nested: vec![1, 2, 3],
            })
            .unwrap(),
            ParsedItem {
                value: 42,
                id: Some(Id::new(42)),
                nested: ParsedNestedItem {
                    values: vec![1, 2, 3],
                },
            }
        );
        assert_eq!(
            Item::from(ParsedItem {
                value: 42,
                id: Some(Id::new(42)),
                nested: ParsedNestedItem {
                    values: vec![1, 2, 3],
                },
            }),
            Item {
                value: 42,
                id: Some("0000000000000000000000000000002A".into()),
                nested: vec![1, 2, 3],
            }
        );
    }

    #[test]
    fn serde_as() {
        #[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
        struct Item {
            value: i32,
        }

        #[derive(Debug, Clone, PartialEq, Parse)]
        #[parse(bomboni_crate = bomboni, source = Item, write, serde_as)]
        struct ParsedItem {
            value: i32,
        }

        let js = serde_json::to_string_pretty(&Item { value: 42 }).unwrap();
        assert_eq!(
            js,
            serde_json::to_string_pretty(&ParsedItem { value: 42 }).unwrap(),
        );
        assert_eq!(serde_json::from_str::<ParsedItem>(&js).unwrap().value, 42);
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
    fn parse_maps() {
        derived_map!(
            pub parse1,
            |item: i32| -> (i32, String) { (item, item.to_string().into()) },
        );
        assert_eq!(
            parse1::parse(vec![1, 2, 3]).unwrap(),
            btree_map_into! {
                1 => "1",
                2 => "2",
                3 => "3",
            }
        );

        derived_map!(
            parse2,
            |item: &'static str| -> RequestResult<(i32, String)> {
                let value: i32 = item.parse().map_err(|_| CommonError::InvalidNumericValue)?;
                Ok((value, item.to_string().into()))
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

        derived_map!(
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
        #[parse(bomboni_crate = bomboni, source = Item)]
        struct ParsedItem {
            #[parse(source = "values", derive = values_parse_hash)]
            values: HashMap<i32, i32>,
        }

        derived_map!(
            values_parse_hash,
            HashMap,
            |item: (i32, i32)| -> (i32, i32) { item },
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
            RequestError::Path(PathError {
                error, ..
            }) if matches!(
                error.as_any().downcast_ref::<CommonError>().unwrap(),
                CommonError::DuplicateValue
            )
        ));
    }
}
