use crate::google::rpc::Code;
#[cfg(feature = "tonic")]
use http::StatusCode;

impl Code {
    #[cfg(feature = "tonic")]
    #[must_use]
    pub fn to_status_code(&self) -> StatusCode {
        // https://cloud.google.com/apis/design/errors#generating_errors
        match self {
            Self::InvalidArgument | Self::FailedPrecondition | Self::OutOfRange => {
                StatusCode::BAD_REQUEST
            }
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::PermissionDenied => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Aborted | Self::AlreadyExists => StatusCode::CONFLICT,
            Self::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
            Self::Cancelled => StatusCode::from_u16(499u16).unwrap(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "tonic")]
impl From<tonic::Code> for Code {
    fn from(code: tonic::Code) -> Self {
        let value = code as i32;
        Self::try_from(value).unwrap()
    }
}

#[cfg(feature = "tonic")]
impl From<Code> for tonic::Code {
    fn from(code: Code) -> Self {
        let value = code as i32;
        Self::from(value)
    }
}
