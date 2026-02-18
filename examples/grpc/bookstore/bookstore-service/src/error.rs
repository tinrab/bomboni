use bomboni_request::error::{CommonError, RequestError};
use bookstore_api::model::error::{AuthorError, BookError, BookstoreError};
use thiserror::Error;
use tonic::{Code, Status, transport};
use tracing::error;

/// Application error types.
///
/// Represents all possible errors that can occur in the bookstore service.
#[derive(Debug, Error)]
pub enum AppError {
    /// Internal application error.
    ///
    /// Represents unexpected internal errors that occur during service operation.
    #[error("internal error: {0}")]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// Request processing error.
    ///
    /// Represents errors that occur during request validation and processing.
    #[error("request error: {0}")]
    Request(#[from] RequestError),

    /// gRPC status error.
    ///
    /// Represents gRPC protocol-level errors.
    #[error("status error: {0}")]
    Status(#[from] Status),
}

/// Application result type.
///
/// Type alias for Result with `AppError` as the error type.
/// Used throughout the application for consistent error handling.
pub type AppResult<T> = Result<T, AppError>;

macro_rules! impl_internal_errors {
    ( $( $type:ty ),* $(,)? ) => {
        $(
        impl From<$type> for AppError {
            fn from(err: $type) -> Self {
                AppError::Internal(Box::new(err))
            }
        }
        )*
    };
}
impl_internal_errors!(config::ConfigError, transport::Error);

macro_rules! impl_request_errors {
    ( $( $type:ty ),* $(,)? ) => {
        $(
        impl From<$type> for AppError {
            fn from(err: $type) -> Self {
                RequestError::from(err).into()
            }
        }
        )*
    };
}
impl_request_errors!(BookstoreError, BookError, AuthorError, CommonError);

impl From<AppError> for Status {
    /// Converts application errors to gRPC status codes.
    ///
    /// Maps internal errors to appropriate gRPC status codes for client responses.
    fn from(err: AppError) -> Self {
        match err {
            AppError::Request(err) => err.into(),
            AppError::Status(status) => status,
            AppError::Internal(_) => {
                error!("internal service error: {}", err);
                Self::internal(Code::Internal.description())
            }
        }
    }
}
