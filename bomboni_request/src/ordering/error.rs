use thiserror::Error;

use crate::string::String;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum OrderingError {
    #[error("invalid ordering term format `{0}`")]
    InvalidTermFormat(String),
    #[error("unknown ordering member `{0}`")]
    UnknownMember(String),
    #[error("duplicate ordering field `{0}`")]
    DuplicateField(String),
    #[error("invalid ordering direction `{0}`")]
    InvalidDirection(String),
    #[error("unordered field `{0}`")]
    UnorderedField(String),
}

pub type OrderingResult<T> = Result<T, OrderingError>;
