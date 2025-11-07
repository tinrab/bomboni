pub mod auth;

#[allow(unused_qualifications, clippy::all, clippy::pedantic)]
pub mod common {
    pub const COMMON_DOMAIN: &str = "common.rabzelj.com";

    pub mod errors {
        use bomboni_proto::include_proto;
        include_proto!("common.errors");
        include_proto!("common.errors.plus");
    }
}

use bomboni_request::error::CommonError;

use common::errors::common_error::CommonErrorReason;

pub fn get_common_error_reason(error: &CommonError) -> CommonErrorReason {
    match error {
        CommonError::ResourceNotFound => CommonErrorReason::ResourceNotFound,
        CommonError::Unauthorized => CommonErrorReason::Unauthorized,
        CommonError::RequiredFieldMissing => CommonErrorReason::RequiredFieldMissing,
        CommonError::InvalidName { .. } => CommonErrorReason::InvalidName,
        CommonError::InvalidNameAlternative { .. } => CommonErrorReason::InvalidName,
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
