use thiserror::Error;

use crate::{filter::error::FilterError, ordering::error::OrderingError};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum QueryError {
    #[error("filter error: {0}")]
    FilterError(FilterError),
    #[error("filter is too long")]
    FilterTooLong,
    #[error("filter schema mismatch")]
    FilterSchemaMismatch,
    #[error("ordering error: {0}")]
    OrderingError(OrderingError),
    #[error("ordering is too long")]
    OrderingTooLong,
    #[error("ordering schema mismatch")]
    OrderingSchemaMismatch,
    #[error("query is too long")]
    QueryTooLong,
    #[error("page token is invalid")]
    InvalidPageToken,
    #[error("page token could not be built")]
    PageTokenFailure,
    #[error("page size specified is invalid")]
    InvalidPageSize,
}

pub type QueryResult<T> = Result<T, QueryError>;

impl From<FilterError> for QueryError {
    fn from(err: FilterError) -> Self {
        QueryError::FilterError(err)
    }
}

impl From<OrderingError> for QueryError {
    fn from(err: OrderingError) -> Self {
        QueryError::OrderingError(err)
    }
}
