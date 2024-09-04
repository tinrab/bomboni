//! # Utilities for working with API requests.
//!
//! This crate provides utilities for working API requests based on Google's AIP resource-oriented gRPC API conventions [1].
//!
//! [1]: https://google.aip.dev

#[allow(unused_extern_crates)]
extern crate regex;

pub mod error;
pub mod filter;
pub mod ordering;
pub mod parse;
pub mod query;
pub mod schema;
pub mod sql;
pub mod string;
pub mod value;

#[cfg(feature = "testing")]
pub mod testing;

#[cfg(feature = "derive")]
pub mod derive {
    pub use bomboni_request_derive::*;
}

#[doc(hidden)]
#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
pub mod bomboni {
    pub mod wasm {
        pub use bomboni_wasm::*;
    }
}

#[cfg(test)]
mod tests {
    // use crate::parse::RequestParse;
    // use crate::parse::RequestParseInto;
    use crate::parse::RequestParseInto;
    use bomboni_request_derive::Parse;
    use filter::Filter;
    use std::time::Instant;
    use string::String;

    use super::*;

    pub mod bomboni {
        pub mod proto {
            pub use bomboni_proto::*;
        }

        pub mod request {
            pub use crate::*;
        }
    }

    #[test]
    fn it_works() {
        const N: usize = 10_000_000;

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
                    required_string: "required_string".repeat(20).into(),
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
        // #[derive(Debug, PartialEq)]
        // #[allow(dead_code)]
        // enum ParsedOneofKind {
        //     String(String),
        //     Boolean(bool),
        //     Nested(ParsedNestedItem),
        // }

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

        for _ in 0..(N / 10) {
            let source = Item::default();
            let parsed: ParsedItem = source.parse_into().unwrap();
            let _: Item = parsed.into();
        }

        let now = Instant::now();
        for _ in 0..N {
            let source = Item::default();
            let parsed: ParsedItem = source.parse_into().unwrap();
            let _: Item = parsed.into();
        }
        println!("{}ms", now.elapsed().as_millis());
    }
}
