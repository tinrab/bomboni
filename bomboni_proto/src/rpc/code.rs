#[cfg(feature = "tonic")]
use crate::google::rpc::Code;
#[cfg(feature = "tonic")]
use http::StatusCode;

impl Code {
    #[cfg(feature = "tonic")]
    pub fn to_status_code(&self) -> StatusCode {
        // https://cloud.google.com/apis/design/errors#generating_errors
        match self {
            Code::InvalidArgument | Code::FailedPrecondition | Code::OutOfRange => {
                StatusCode::BAD_REQUEST
            }
            Code::Unauthenticated => StatusCode::UNAUTHORIZED,
            Code::PermissionDenied => StatusCode::FORBIDDEN,
            Code::NotFound => StatusCode::NOT_FOUND,
            Code::Aborted | Code::AlreadyExists => StatusCode::CONFLICT,
            Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
            Code::Cancelled => StatusCode::from_u16(499u16).unwrap(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "tonic")]
impl From<tonic::Code> for Code {
    fn from(code: tonic::Code) -> Self {
        let value = code as i32;
        Code::try_from(value).unwrap()
    }
}

#[cfg(feature = "tonic")]
impl From<Code> for tonic::Code {
    fn from(code: Code) -> Self {
        let value = code as i32;
        tonic::Code::from(value)
    }
}
