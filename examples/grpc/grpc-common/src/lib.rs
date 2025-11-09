//! Common utilities and types for gRPC services.
//!
//! This crate provides shared functionality for authentication, error handling,
//! and protocol buffer definitions used across gRPC services.

pub mod auth;

/// The common domain used across services.
pub const COMMON_DOMAIN: &str = "common.rabzelj.com";

#[allow(
    unused_qualifications,
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_html_tags
)]
pub mod proto {
    use bomboni_proto::{include_proto, serde::helpers as serde_helpers};

    include_proto!("common");
    include_proto!("common.plus");
}

use bomboni_request::error::CommonError;

use proto::common_error::CommonErrorReason;

/// Converts a common error to its corresponding protobuf reason.
///
/// # Arguments
///
/// * `error` - The common error to convert
///
/// # Returns
///
/// The corresponding protobuf error reason.
pub const fn get_common_error_reason(error: &CommonError) -> CommonErrorReason {
    match error {
        CommonError::ResourceNotFound => CommonErrorReason::ResourceNotFound,
        CommonError::Unauthorized => CommonErrorReason::Unauthorized,
        CommonError::RequiredFieldMissing => CommonErrorReason::RequiredFieldMissing,
        CommonError::InvalidName { .. } | CommonError::InvalidNameAlternative { .. } => {
            CommonErrorReason::InvalidName
        }
        CommonError::InvalidParent { .. } => CommonErrorReason::InvalidParent,
        CommonError::InvalidStringFormat { .. } => CommonErrorReason::InvalidStringFormat,
        CommonError::InvalidId => CommonErrorReason::InvalidId,
        CommonError::DuplicateId => CommonErrorReason::DuplicateId,
        CommonError::InvalidDisplayName => CommonErrorReason::InvalidDisplayName,
        CommonError::InvalidDateTime => CommonErrorReason::InvalidDateTime,
        CommonError::InvalidEnumValue => CommonErrorReason::InvalidEnumValue,
        CommonError::UnknownOneofVariant => CommonErrorReason::UnknownOneofVariant,
        CommonError::InvalidNumericValue => CommonErrorReason::InvalidNumericValue,
        CommonError::FailedConvertValue => CommonErrorReason::FailedConvertValue,
        CommonError::NumericOutOfRange => CommonErrorReason::NumericOutOfRange,
        CommonError::DuplicateValue => CommonErrorReason::DuplicateValue,
        CommonError::AlreadyExists => CommonErrorReason::AlreadyExists,
        CommonError::NotFound => CommonErrorReason::NotFound,
        CommonError::TypeMismatch => CommonErrorReason::TypeMismatch,
    }
}
