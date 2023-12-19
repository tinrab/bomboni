use bomboni_proto::google::rpc::Code;
use thiserror::Error;

use crate::{error::DomainError, filter::error::FilterError, ordering::error::OrderingError};

#[derive(Error, Debug, Clone, PartialEq, Eq)]
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

impl QueryError {
    pub fn get_violating_field_name(&self) -> &'static str {
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

impl DomainError for QueryError {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn code(&self) -> Code {
        Code::InvalidArgument
    }
}
