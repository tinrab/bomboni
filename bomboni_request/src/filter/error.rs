use pest::error::InputLocation;
use thiserror::Error;

use crate::{
    filter::{FilterComparator, Rule},
    schema::ValueType,
};

/// Filter parsing and evaluation errors.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FilterError {
    /// Parse error with location.
    #[error("failed to parse filter from `{start}` to `{end}`")]
    Parse {
        /// Start position.
        start: usize,
        /// End position.
        end: usize,
    },
    /// Expected a value.
    #[error("expected a filter value")]
    ExpectedValue,
    /// Invalid number format.
    #[error("invalid number `{0}`")]
    InvalidNumber(String),
    /// Unknown field member.
    #[error("unknown filter member `{0}`")]
    UnknownMember(String),
    /// Unknown function.
    #[error("unknown function `{0}`")]
    UnknownFunction(String),
    /// Invalid function argument count.
    #[error("invalid number of arguments for function `{name}`, expected {expected}")]
    FunctionInvalidArgumentCount {
        /// Function name.
        name: String,
        /// Expected count.
        expected: usize,
    },
    /// Invalid result value type.
    #[error("invalid result value type of filter")]
    InvalidResultValueType,
    /// Type mismatch error.
    #[error("expected filter type `{expected}`, but got `{actual}`")]
    InvalidType {
        /// Expected type.
        expected: ValueType,
        /// Actual type.
        actual: ValueType,
    },
    /// Incomparable value type.
    #[error("incomparable value type `{0}`")]
    IncomparableType(ValueType),
    /// Unsuitable comparator.
    #[error("unsuitable comparator `{0}`")]
    UnsuitableComparator(FilterComparator),
}

/// Filter result type.
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
