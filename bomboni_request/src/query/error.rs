use bomboni_proto::google::rpc::Code;
use thiserror::Error;

use crate::{error::GenericError, filter::error::FilterError, ordering::error::OrderingError};

/// Query processing errors.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum QueryError {
    /// Filter error.
    #[error("filter error: {0}")]
    FilterError(FilterError),
    /// Filter is too long.
    #[error("filter is too long")]
    FilterTooLong,
    /// Filter schema mismatch.
    #[error("filter schema mismatch")]
    FilterSchemaMismatch,
    /// Ordering error.
    #[error("ordering error: {0}")]
    OrderingError(OrderingError),
    /// Ordering is too long.
    #[error("ordering is too long")]
    OrderingTooLong,
    /// Ordering schema mismatch.
    #[error("ordering schema mismatch")]
    OrderingSchemaMismatch,
    /// Query is too long.
    #[error("query is too long")]
    QueryTooLong,
    /// Page token is invalid.
    #[error("page token is invalid")]
    InvalidPageToken,
    /// Page token could not be built.
    #[error("page token could not be built")]
    PageTokenFailure,
    /// Page size specified is invalid.
    #[error("page size specified is invalid")]
    InvalidPageSize,
}

/// Result type for query operations.
pub type QueryResult<T> = Result<T, QueryError>;

impl QueryError {
    /// Gets the name of the field that caused the error.
    pub const fn get_violating_field_name(&self) -> &'static str {
        match self {
            Self::FilterError(_) | Self::FilterTooLong | Self::FilterSchemaMismatch => "filter",
            Self::OrderingError(_) | Self::OrderingTooLong | Self::OrderingSchemaMismatch => {
                "order_by"
            }
            Self::QueryTooLong => "query",
            Self::InvalidPageToken | Self::PageTokenFailure => "page_token",
            Self::InvalidPageSize => "page_size",
        }
    }
}

impl From<FilterError> for QueryError {
    fn from(err: FilterError) -> Self {
        Self::FilterError(err)
    }
}

impl From<OrderingError> for QueryError {
    fn from(err: OrderingError) -> Self {
        Self::OrderingError(err)
    }
}

impl GenericError for QueryError {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn code(&self) -> Code {
        Code::InvalidArgument
    }
}
