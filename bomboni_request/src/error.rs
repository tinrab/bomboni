use thiserror::Error;

use crate::query::error::QueryError;
use bomboni_proto::google::protobuf::Any;
use bomboni_proto::google::rpc::bad_request::FieldViolation;
use bomboni_proto::google::rpc::BadRequest;
use bomboni_proto::google::rpc::{Code, Status};
use itertools::Itertools;
use prost::{DecodeError, EncodeError};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("invalid `{name}` request")]
    BadRequest {
        name: String,
        violations: Vec<FieldError>,
    },
    #[error(transparent)]
    Field(FieldError),
    #[error("{0}")]
    Domain(DomainErrorBox),
    #[error("encode error: {0}")]
    Encode(#[from] EncodeError),
    #[error("decode error: {0}")]
    Decode(#[from] DecodeError),
}

pub type RequestResult<T> = Result<T, RequestError>;

#[derive(Debug)]
pub struct FieldError {
    pub field: String,
    pub error: DomainErrorBox,
    pub index: Option<usize>,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CommonError {
    #[error("requested entity was not found")]
    ResourceNotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("no value provided for required field")]
    RequiredFieldMissing,
    #[error("expected `{expected_format}`, but got `{name}`.")]
    InvalidName {
        expected_format: String,
        name: String,
    },
    #[error(
        "expected either `{expected_format}` or `{alternative_expected_format}`, but got `{name}`"
    )]
    InvalidNameAlternative {
        expected_format: String,
        alternative_expected_format: String,
        name: String,
    },
    #[error("expected resource parent `{expected}`, but got `{parent}`")]
    InvalidParent { expected: String, parent: String },
    #[error("expected a string in format `{expected}`")]
    InvalidStringFormat { expected: String },
    #[error("invalid ID format")]
    InvalidId,
    #[error("duplicate ID")]
    DuplicateId,
    #[error("invalid display name format")]
    InvalidDisplayName,
    #[error("invalid date time format")]
    InvalidDateTime,
    #[error("invalid enum value")]
    InvalidEnumValue,
    #[error("unknown oneof variant")]
    UnknownOneofVariant,
    #[error("invalid numeric value")]
    InvalidNumericValue,
    #[error("failed to convert value")]
    FailedConvertValue,
    #[error("out of range")]
    NumericOutOfRange,
    #[error("duplicate value")]
    DuplicateValue,
    #[error("already exists")]
    AlreadyExists,
    #[error("not found")]
    NotFound,
    #[error("type mismatch")]
    TypeMismatch,
}

pub trait DomainError: Error {
    fn as_any(&self) -> &dyn std::any::Any;

    fn code(&self) -> Code {
        Code::InvalidArgument
    }

    fn details(&self) -> Vec<Any> {
        Vec::default()
    }
}

pub type DomainErrorBox = Box<dyn DomainError + Send + Sync>;

impl RequestError {
    #[must_use]
    pub fn bad_request<N, V, F, E>(name: N, violations: V) -> Self
    where
        N: Display,
        V: IntoIterator<Item = (F, E)>,
        F: Display,
        E: Into<DomainErrorBox>,
    {
        Self::BadRequest {
            name: name.to_string(),
            violations: violations
                .into_iter()
                .map(|(field, error)| FieldError {
                    field: field.to_string(),
                    error: error.into(),
                    index: None,
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn domain<E: Into<DomainErrorBox>>(error: E) -> Self {
        Self::Domain(error.into())
    }

    #[must_use]
    pub fn field<F, E>(field: F, error: E) -> Self
    where
        F: Display,
        E: Into<DomainErrorBox>,
    {
        FieldError {
            field: field.to_string(),
            error: error.into(),
            index: None,
        }
        .into()
    }

    #[must_use]
    pub fn field_index<F, I, E>(field: F, index: I, error: E) -> Self
    where
        F: Display,
        I: Into<usize>,
        E: Into<DomainErrorBox>,
    {
        FieldError {
            field: field.to_string(),
            error: error.into(),
            index: Some(index.into()),
        }
        .into()
    }

    #[must_use]
    pub fn wrap<F: Display>(self, root_field: F) -> Self {
        match self {
            Self::Field(error) => FieldError {
                field: format!("{}.{}", root_field, error.field),
                ..error
            }
            .into(),
            Self::Domain(error) => Self::field(root_field, error),
            // TODO: skip or panic?
            err => err,
            // _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn wrap_index<F, I>(self, root_field: F, root_index: I) -> Self
    where
        F: Display,
        I: Display + Into<usize>,
    {
        match self {
            Self::Field(error) => FieldError {
                field: format!("{}[{}].{}", root_field, root_index, error.field),
                ..error
            }
            .into(),
            Self::Domain(error) => Self::field_index(root_field, root_index, error),
            // TODO: skip or panic?
            err => err,
            // _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn wrap_request<N: Display>(self, name: N) -> Self {
        match self {
            Self::Field(error) => Self::bad_request(name, [(error.field, error.error)]),
            Self::Domain(error) => {
                if let Some(error) = error.as_any().downcast_ref::<QueryError>() {
                    #[allow(trivial_casts)]
                    Self::bad_request(
                        name,
                        [(
                            error.get_violating_field_name(),
                            Box::new(error.clone()) as DomainErrorBox,
                        )],
                    )
                } else {
                    RequestError::Domain(error)
                }
            }
            error => error,
        }
    }

    #[must_use]
    pub fn wrap_request_nested<N, P, F>(self, name: N, root_path: P) -> Self
    where
        N: Display,
        P: IntoIterator<Item = F>,
        F: Display,
    {
        match self {
            Self::Field(error) => Self::bad_request(
                name,
                [(
                    format!(
                        "{}.{}",
                        root_path.into_iter().map(|step| step.to_string()).join("."),
                        error.field
                    ),
                    error.error,
                )],
            ),
            Self::Domain(error) => {
                if let Some(error) = error.as_any().downcast_ref::<QueryError>() {
                    #[allow(trivial_casts)]
                    Self::bad_request(
                        name,
                        [(
                            format!(
                                "{}.{}",
                                root_path.into_iter().map(|step| step.to_string()).join("."),
                                error.get_violating_field_name()
                            ),
                            Box::new(error.clone()) as DomainErrorBox,
                        )],
                    )
                } else {
                    RequestError::Domain(error)
                }
            }
            error => error,
        }
    }

    pub fn downcast_domain_ref<T: std::any::Any>(&self) -> Option<&T> {
        if let Self::Domain(error) = self {
            error.as_any().downcast_ref::<T>()
        } else {
            None
        }
    }

    pub fn downcast_domain<T: 'static + Clone>(&self) -> Option<T> {
        if let Self::Domain(error) = self {
            error.as_any().downcast_ref::<T>().cloned()
        } else {
            None
        }
    }

    pub fn code(&self) -> Code {
        match self {
            Self::Encode(_) | Self::Decode(_) | Self::BadRequest { .. } => Code::InvalidArgument,
            Self::Field(error) => error.code(),
            Self::Domain(error) => error.code(),
        }
    }

    pub fn details(&self) -> Vec<Any> {
        match self {
            Self::BadRequest { violations, .. } => vec![BadRequest {
                field_violations: violations
                    .iter()
                    .map(|error| FieldViolation {
                        field: error.field.clone(),
                        description: error.error.to_string(),
                    })
                    .collect(),
            }
            .try_into()
            .unwrap()],
            Self::Field(error) => error.details(),
            Self::Domain(error) => error.details(),
            _ => Vec::new(),
        }
    }
}

impl From<RequestError> for Status {
    fn from(err: RequestError) -> Self {
        Status::from(&err)
    }
}

impl From<&RequestError> for Status {
    fn from(err: &RequestError) -> Self {
        Status::new(err.code(), err.to_string(), err.details())
    }
}

#[cfg(feature = "tonic")]
impl From<RequestError> for tonic::Status {
    fn from(err: RequestError) -> Self {
        Status::from(&err).into()
    }
}

#[cfg(feature = "tonic")]
impl From<&RequestError> for tonic::Status {
    fn from(err: &RequestError) -> Self {
        Status::from(err).into()
    }
}

impl FieldError {
    pub fn code(&self) -> Code {
        self.error.code()
    }

    pub fn details(&self) -> Vec<Any> {
        self.error.details()
    }
}

impl Error for FieldError {}

impl Display for FieldError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(index) = self.index {
            write!(
                f,
                "field `{}[{}]` error: `{}`",
                self.field, index, self.error
            )
        } else {
            write!(f, "field `{}` error: `{}`", self.field, self.error)
        }
    }
}

impl From<FieldError> for RequestError {
    fn from(err: FieldError) -> Self {
        RequestError::Field(err)
    }
}

impl DomainError for CommonError {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn code(&self) -> Code {
        match self {
            Self::ResourceNotFound | Self::NotFound => Code::NotFound,
            Self::AlreadyExists => Code::AlreadyExists,
            Self::Unauthorized => Code::PermissionDenied,
            _ => Code::InvalidArgument,
        }
    }
}

impl<T> From<T> for DomainErrorBox
where
    T: 'static + DomainError + Send + Sync,
{
    fn from(err: T) -> Self {
        Box::new(err)
    }
}

impl<T: 'static + DomainError + Send + Sync> From<T> for RequestError {
    fn from(err: T) -> Self {
        RequestError::Domain(Box::new(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let err = RequestError::bad_request("Test", [("x", CommonError::InvalidId)]);
        assert_eq!(err.to_string(), "invalid `Test` request");
        assert_eq!(
            err.details().remove(0).unpack_into::<BadRequest>().unwrap(),
            BadRequest {
                field_violations: vec![FieldViolation {
                    field: "x".into(),
                    description: "invalid ID format".into(),
                }]
            }
        );
    }

    #[test]
    fn query_error_metadata() {
        assert_eq!(
            serde_json::to_value(Status::from(
                RequestError::from(QueryError::InvalidPageSize).wrap_request("List"),
            ))
            .unwrap(),
            serde_json::from_str::<serde_json::Value>(
                r#"{
                "code": "INVALID_ARGUMENT",
                "message": "invalid `List` request",
                "details": [
                    {
                        "@type": "type.googleapis.com/google.rpc.BadRequest",
                        "fieldViolations": [
                            {
                                "field": "page_size",
                                "description": "page size specified is invalid"
                            }
                        ]
                    }
                ]
            }"#
            )
            .unwrap()
        );
    }
}
