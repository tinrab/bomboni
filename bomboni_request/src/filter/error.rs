use pest::error::InputLocation;
use thiserror::Error;

use crate::schema::ValueType;

use super::parser::Rule;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FilterError {
    #[error("failed to parse filter from `{start}` to `{end}`")]
    Parse { start: usize, end: usize },
    #[error("invalid filter value `{0}`")]
    InvalidNumber(String),
    #[error("expected filter type `{expected}`, but got `{actual}`")]
    InvalidType {
        expected: ValueType,
        actual: ValueType,
    },
    #[error("incomparable filter type `{0}`")]
    IncomparableType(ValueType),
    #[error("expected a filter value")]
    ExpectedValue,
}

pub type FilterResult<T> = Result<T, FilterError>;

impl From<pest::error::Error<Rule>> for FilterError {
    fn from(err: pest::error::Error<Rule>) -> Self {
        match err.location {
            InputLocation::Pos(pos) => FilterError::Parse {
                start: pos,
                end: pos,
            },
            InputLocation::Span(span) => FilterError::Parse {
                start: span.0,
                end: span.1,
            },
        }
    }
}
