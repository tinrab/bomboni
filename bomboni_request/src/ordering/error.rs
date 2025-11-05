use thiserror::Error;

/// Ordering parsing errors.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum OrderingError {
    /// Invalid term format.
    #[error("invalid ordering term format `{0}`")]
    InvalidTermFormat(String),
    /// Unknown field member.
    #[error("unknown ordering member `{0}`")]
    UnknownMember(String),
    /// Duplicate field in ordering.
    #[error("duplicate ordering field `{0}`")]
    DuplicateField(String),
    /// Invalid sort direction.
    #[error("invalid ordering direction `{0}`")]
    InvalidDirection(String),
    /// Field is not orderable.
    #[error("unordered field `{0}`")]
    UnorderedField(String),
}

/// Ordering result type.
pub type OrderingResult<T> = Result<T, OrderingError>;
