use pest::error::InputLocation;
use thiserror::Error;

use crate::{
    filter::{FilterComparator, Rule},
    schema::ValueType,
    string::String,
};

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FilterError {
    #[error("failed to parse filter from `{start}` to `{end}`")]
    Parse { start: usize, end: usize },
    #[error("expected a filter value")]
    ExpectedValue,
    #[error("invalid number `{0}`")]
    InvalidNumber(String),
    #[error("unknown filter member `{0}`")]
    UnknownMember(String),
    #[error("unknown function `{0}`")]
    UnknownFunction(String),
    #[error("invalid number of arguments for function `{name}`, expected {expected}")]
    FunctionInvalidArgumentCount { name: String, expected: usize },
    #[error("invalid result value type of filter")]
    InvalidResultValueType,
    #[error("expected filter type `{expected}`, but got `{actual}`")]
    InvalidType {
        expected: ValueType,
        actual: ValueType,
    },
    #[error("incomparable value type `{0}`")]
    IncomparableType(ValueType),
    #[error("unsuitable comparator `{0}`")]
    UnsuitableComparator(FilterComparator),
}

pub type FilterResult<T> = Result<T, FilterError>;

impl From<pest::error::Error<Rule>> for FilterError {
    fn from(err: pest::error::Error<Rule>) -> Self {
        match err.location {
            InputLocation::Pos(pos) => Self::Parse {
                start: pos,
                end: pos,
            },
            InputLocation::Span(span) => Self::Parse {
                start: span.0,
                end: span.1,
            },
        }
    }
}
