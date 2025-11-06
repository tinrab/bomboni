use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use bomboni_common::id::Id;
use bomboni_macros::btree_map_into;
use bomboni_request::{
    derive::{Parse, parse_resource_name},
    error::RequestError,
    parse::ParsedResource,
    schema::{FieldMemberSchema, Schema, SchemaMapped, ValueType},
    value::Value,
};

use crate::{
    model::author::{AuthorId, author_id_convert},
    v1::Book,
};

#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = Book, write)]
pub struct BookModel {
    #[parse(resource)]
    pub resource: ParsedResource,
    #[parse(source = "name", convert = book_id_convert)]
    pub id: BookId,
    pub display_name: String,
    #[parse(source = "author", convert = author_id_convert)]
    pub author_id: AuthorId,
    pub isbn: String,
    pub description: String,
    pub price_cents: i64,
    pub page_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BookId(pub Id);

impl BookModel {
    pub const NAME_PATTERN: &str = "books/{book_id}";

    pub fn get_schema() -> Schema {
        Schema {
            members: btree_map_into! {
                "id" => FieldMemberSchema::new_ordered(ValueType::String),
                "display_name" => FieldMemberSchema::new_ordered(ValueType::String),
                "author" => FieldMemberSchema::new_ordered(ValueType::String),
                "isbn" => FieldMemberSchema::new_ordered(ValueType::String),
                "description" => FieldMemberSchema::new_ordered(ValueType::String),
                "price_cents" => FieldMemberSchema::new_ordered(ValueType::Integer),
                "page_count" => FieldMemberSchema::new_ordered(ValueType::Integer),
            },
        }
    }
}

impl SchemaMapped for BookModel {
    fn get_field(&self, name: &str) -> Value {
        match name {
            "id" => self.id.0.to_string().into(),
            "display_name" => self.display_name.clone().into(),
            "author" => self.author_id.0.to_string().into(),
            "isbn" => self.isbn.clone().into(),
            "description" => self.description.clone().into(),
            "price_cents" => self.price_cents.into(),
            "page_count" => self.page_count.into(),
            _ => unimplemented!("SchemaMapped for BookModel::{name}"),
        }
    }
}

impl BookId {
    pub fn new<T: Into<Id>>(id: T) -> Self {
        Self(id.into())
    }

    pub fn parse_name<S: AsRef<str>>(name: S) -> Option<Self> {
        let id = parse_resource_name!({
            "books": Id,
        })(name.as_ref())?
        .0;
        Some(Self(id))
    }

    pub fn to_name(&self) -> String {
        format!("books/{}", self.0)
    }
}

impl Display for BookId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_name().fmt(f)
    }
}

impl FromStr for BookId {
    type Err = RequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        book_id_convert::parse(s)
    }
}

pub mod book_id_convert {
    use bomboni_request::error::{CommonError, RequestResult};

    use crate::model::book::{BookId, BookModel};

    pub fn parse<S: AsRef<str> + ToString>(name: S) -> RequestResult<BookId> {
        BookId::parse_name(name.as_ref()).ok_or_else(|| {
            CommonError::InvalidName {
                expected_format: BookModel::NAME_PATTERN.into(),
                name: name.to_string(),
            }
            .into()
        })
    }

    pub fn write(id: BookId) -> String {
        id.to_name()
    }
}
