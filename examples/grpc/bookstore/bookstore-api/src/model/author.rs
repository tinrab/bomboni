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

use crate::v1::Author;

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = Author, write)]
pub struct AuthorModel {
    #[parse(resource)]
    pub resource: ParsedResource,
    #[parse(source = "name", convert = author_id_convert)]
    pub id: AuthorId,
    pub display_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorId(pub Id);

impl AuthorModel {
    pub const NAME_PATTERN: &str = "authors/{author_id}";

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
    pub fn new<T: Into<Id>>(id: T) -> Self {
        Self(id.into())
    }

    pub fn parse_name<S: AsRef<str>>(name: S) -> Option<Self> {
        let id = parse_resource_name!({
            "authors": Id,
        })(name.as_ref())?
        .0;
        Some(Self(id))
    }

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

pub mod author_id_convert {
    use bomboni_request::error::{CommonError, RequestResult};

    use crate::model::author::{AuthorId, AuthorModel};

    /// Parse an author ID from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the name format is invalid.
    pub fn parse<S: AsRef<str> + ToString>(name: S) -> RequestResult<AuthorId> {
        AuthorId::parse_name(name.as_ref()).ok_or_else(|| {
            CommonError::InvalidName {
                expected_format: AuthorModel::NAME_PATTERN.into(),
                name: name.to_string(),
            }
            .into()
        })
    }

    pub fn write(id: AuthorId) -> String {
        id.to_name()
    }
}
