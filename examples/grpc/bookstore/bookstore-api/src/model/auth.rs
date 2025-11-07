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

use crate::v1::User;

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = User, write)]
pub struct UserModel {
    #[parse(resource)]
    pub resource: ParsedResource,
    #[parse(source = "name", convert = user_id_convert)]
    pub id: UserId,
    #[parse(source = "display_name")]
    pub display_name: String,
    #[parse(source = "email")]
    pub email: String,
    #[parse(source = "active")]
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(pub Id);

impl UserModel {
    pub const NAME_PATTERN: &str = "users/{user_id}";

    pub fn get_schema() -> Schema {
        Schema {
            members: btree_map_into! {
                "id" => FieldMemberSchema::new_ordered(ValueType::String),
                "display_name" => FieldMemberSchema::new_ordered(ValueType::String),
                "email" => FieldMemberSchema::new_ordered(ValueType::String),
                "active" => FieldMemberSchema::new_ordered(ValueType::Boolean),
            },
        }
    }
}

impl SchemaMapped for UserModel {
    fn get_field(&self, name: &str) -> Value {
        match name {
            "id" => self.id.0.to_string().into(),
            "display_name" => self.display_name.clone().into(),
            "email" => self.email.clone().into(),
            "active" => self.active.into(),
            _ => unimplemented!("SchemaMapped for UserModel::{name}"),
        }
    }
}

impl UserId {
    pub fn new<T: Into<Id>>(id: T) -> Self {
        Self(id.into())
    }

    pub fn parse_name<S: AsRef<str>>(name: S) -> Option<Self> {
        let id = parse_resource_name!({
            "users": Id,
        })(name.as_ref())?
        .0;
        Some(Self(id))
    }

    pub fn to_name(&self) -> String {
        format!("users/{}", self.0)
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_name().fmt(f)
    }
}

impl FromStr for UserId {
    type Err = RequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        user_id_convert::parse(s)
    }
}

pub mod user_id_convert {
    use bomboni_request::error::{CommonError, RequestResult};

    use crate::model::auth::{UserId, UserModel};

    /// Parse a user ID from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the name format is invalid.
    pub fn parse<S: AsRef<str> + ToString>(name: S) -> RequestResult<UserId> {
        UserId::parse_name(name.as_ref()).ok_or_else(|| {
            CommonError::InvalidName {
                expected_format: UserModel::NAME_PATTERN.into(),
                name: name.to_string(),
            }
            .into()
        })
    }

    pub fn write(id: UserId) -> String {
        id.to_name()
    }
}
