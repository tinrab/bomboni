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

use crate::v1::Author;

/// Author model representing an author in the bookstore.
///
/// This struct provides a parsed representation of an author with
/// resource metadata and typed fields.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = Author, write)]
pub struct AuthorModel {
    /// Resource metadata including timestamps and deletion status.
    #[parse(resource)]
    pub resource: ParsedResource,
    /// Unique identifier for the author.
    #[parse(source = "name", convert = author_id_convert)]
    pub id: AuthorId,
    /// Display name of the author.
    pub display_name: String,
}

/// Unique identifier for an author.
///
/// This wraps an internal ID and provides methods for parsing and formatting
/// author resource names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorId(pub Id);

impl AuthorModel {
    /// The name pattern for author resources.
    pub const NAME_PATTERN: &str = "authors/{author_id}";

    /// Returns the schema for author model fields.
    ///
    /// This defines the available fields for querying and filtering authors.
    pub fn get_schema() -> Schema {
        Schema {
            members: btree_map_into! {
                "id" => FieldMemberSchema::new_ordered(ValueType::String),
                "display_name" => FieldMemberSchema::new_ordered(ValueType::String),
            },
        }
    }
}

impl SchemaMapped for AuthorModel {
    fn get_field(&self, name: &str) -> Value {
        match name {
            "id" => self.id.0.to_string().into(),
            "display_name" => self.display_name.clone().into(),
            _ => unimplemented!("SchemaMapped for AuthorModel::{name}"),
        }
    }
}

impl AuthorId {
    /// Creates a new author ID from any type that can be converted to an Id.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID value to wrap
    pub fn new<T: Into<Id>>(id: T) -> Self {
        Self(id.into())
    }

    /// Parses an author ID from a resource name string.
    ///
    /// # Arguments
    ///
    /// * `name` - The resource name to parse (e.g., "authors/123")
    ///
    /// # Returns
    ///
    /// Some(AuthorId) if parsing succeeds, None otherwise
    pub fn parse_name<S: AsRef<str>>(name: S) -> Option<Self> {
        let id = parse_resource_name!({
            "authors": Id,
        })(name.as_ref())?
        .0;
        Some(Self(id))
    }

    /// Converts the author ID to a resource name string.
    ///
    /// # Returns
    ///
    /// The formatted resource name (e.g., "authors/123")
    pub fn to_name(&self) -> String {
        format!("authors/{}", self.0)
    }
}

impl Display for AuthorId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_name().fmt(f)
    }
}

impl FromStr for AuthorId {
    type Err = RequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        author_id_convert::parse(s)
    }
}

/// Conversion utilities for author IDs.
pub mod author_id_convert {
    use bomboni::request::error::{CommonError, RequestResult};

    use crate::model::author::{AuthorId, AuthorModel};

    /// Parse an author ID from a string.
    ///
    /// # Errors
    ///
    /// Will return [`RequestError`] if the name format is invalid.
    pub fn parse<S: AsRef<str> + ToString>(name: S) -> RequestResult<AuthorId> {
        AuthorId::parse_name(name.as_ref()).ok_or_else(|| {
            CommonError::InvalidName {
                expected_format: AuthorModel::NAME_PATTERN.into(),
                name: name.to_string(),
            }
            .into()
        })
    }

    /// Writes an author ID to its string representation.
    ///
    /// # Arguments
    ///
    /// * `id` - The author ID to convert
    ///
    /// # Returns
    ///
    /// The formatted resource name string
    pub fn write(id: AuthorId) -> String {
        id.to_name()
    }
}
