use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use bomboni::common::id::Id;
use bomboni::macros::btree_map_into;
use bomboni::request::{
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

/// Book model representing a book in the bookstore.
///
/// This struct provides a parsed representation of a book with
/// resource metadata and typed fields.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = Book, write)]
pub struct BookModel {
    /// Resource metadata including timestamps and deletion status.
    #[parse(resource)]
    pub resource: ParsedResource,
    /// Unique identifier for the book.
    #[parse(source = "name", convert = book_id_convert)]
    pub id: BookId,
    /// Display title of the book.
    pub display_name: String,
    /// ID of the author who wrote the book.
    #[parse(source = "author", convert = author_id_convert)]
    pub author_id: AuthorId,
    /// ISBN number of the book.
    pub isbn: String,
    /// Description of the book.
    pub description: String,
    /// Price in cents.
    pub price_cents: i64,
    /// Number of pages in the book.
    pub page_count: i32,
}

/// Unique identifier for a book.
///
/// This wraps an internal ID and provides methods for parsing and formatting
/// book resource names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BookId(pub Id);

impl BookModel {
    /// The name pattern for book resources.
    pub const NAME_PATTERN: &str = "books/{book_id}";

    /// Returns the schema for book model fields.
    ///
    /// This defines the available fields for querying and filtering books.
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
    /// Creates a new book ID from any type that can be converted to an Id.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID value to wrap
    pub fn new<T: Into<Id>>(id: T) -> Self {
        Self(id.into())
    }

    /// Parses a book ID from a resource name string.
    ///
    /// # Arguments
    ///
    /// * `name` - The resource name to parse (e.g., "books/123")
    ///
    /// # Returns
    ///
    /// Some(BookId) if parsing succeeds, None otherwise
    pub fn parse_name<S: AsRef<str>>(name: S) -> Option<Self> {
        let id = parse_resource_name!({
            "books": Id,
        })(name.as_ref())?
        .0;
        Some(Self(id))
    }

    /// Converts the book ID to a resource name string.
    ///
    /// # Returns
    ///
    /// The formatted resource name (e.g., "books/123")
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

/// Conversion utilities for book IDs.
pub mod book_id_convert {
    use bomboni::request::error::{CommonError, RequestResult};

    use crate::model::book::{BookId, BookModel};

    /// Parse a book ID from a string.
    ///
    /// # Errors
    ///
    /// Will return [`RequestError`] if the name format is invalid.
    pub fn parse<S: AsRef<str> + ToString>(name: S) -> RequestResult<BookId> {
        BookId::parse_name(name.as_ref()).ok_or_else(|| {
            CommonError::InvalidName {
                expected_format: BookModel::NAME_PATTERN.into(),
                name: name.to_string(),
            }
            .into()
        })
    }

    /// Writes a book ID to its string representation.
    ///
    /// # Arguments
    ///
    /// * `id` - The book ID to convert
    ///
    /// # Returns
    ///
    /// The formatted resource name string
    pub fn write(id: BookId) -> String {
        id.to_name()
    }
}
