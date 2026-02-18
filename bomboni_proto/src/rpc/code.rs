use crate::google::rpc::Code;
#[cfg(feature = "tonic")]
use http::StatusCode;

impl Code {
    #[cfg(feature = "tonic")]
    #[must_use]
    /// Converts to HTTP status code.
    ///
    /// # Panics
    ///
    /// Panics if status code creation fails.
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
impl TryFrom<tonic::Code> for Code {
    type Error = ();

    fn try_from(code: tonic::Code) -> Result<Self, Self::Error> {
        let value = code as i32;
        Self::try_from(value).map_err(|_| ())
    }
}

#[cfg(feature = "tonic")]
impl TryFrom<Code> for tonic::Code {
    type Error = Self;

    fn try_from(code: Code) -> Result<Self, Self::Error> {
        let value = code as i32;
        Ok(Self::from(value))
    }
}
