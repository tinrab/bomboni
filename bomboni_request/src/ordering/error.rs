use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum OrderingError {
    #[error("duplicate ordering field `{0}`")]
    DuplicateField(String),
}

pub type OrderingResult<T> = Result<T, OrderingError>;
