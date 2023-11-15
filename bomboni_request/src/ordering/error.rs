use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum OrderingError {
    #[error("duplicate ordering field `{0}`")]
    DuplicateField(String),
}

pub type OrderingResult<T> = Result<T, OrderingError>;
