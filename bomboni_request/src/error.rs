use thiserror::Error;

use crate::query::error::QueryError;
use bomboni_proto::google::protobuf::Any;
use bomboni_proto::google::rpc::bad_request::FieldViolation;
use bomboni_proto::google::rpc::BadRequest;
use bomboni_proto::google::rpc::{Code, Status};
use itertools::Itertools;
use prost::{DecodeError, EncodeError};
use std::error::Error;

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

#[derive(Error, Debug)]
#[error("field `{field}` error: {error}")]
pub struct FieldError {
    pub field: String,
    pub error: DomainErrorBox,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CommonError {
    #[error(transparent)]
    Query(#[from] QueryError),
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
    pub fn bad_request<V, F, E>(name: &str, violations: V) -> Self
    where
        V: IntoIterator<Item = (F, E)>,
        F: ToString,
        E: Into<DomainErrorBox>,
    {
        Self::BadRequest {
            name: name.into(),
            violations: violations
                .into_iter()
                .map(|(field, error)| FieldError {
                    field: field.to_string(),
                    error: error.into(),
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
        F: ToString,
        E: Into<DomainErrorBox>,
    {
        FieldError {
            field: field.to_string(),
            error: error.into(),
        }
        .into()
    }

    #[must_use]
    pub fn field_path<P, F, E>(field_path: P, error: E) -> Self
    where
        P: IntoIterator<Item = F>,
        F: ToString,
        E: Into<DomainErrorBox>,
    {
        FieldError {
            field: field_path
                .into_iter()
                .map(|step| step.to_string())
                .join("."),
            error: error.into(),
        }
        .into()
    }

    #[must_use]
    pub fn wrap(self, root_field: &str) -> Self {
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
    pub fn wrap_field_path<P, F>(self, field_path: P) -> Self
    where
        P: IntoIterator<Item = F>,
        F: ToString,
    {
        match self {
            Self::Field(error) => FieldError {
                field: format!(
                    "{}.{}",
                    field_path
                        .into_iter()
                        .map(|step| step.to_string())
                        .join("."),
                    error.field
                ),
                ..error
            }
            .into(),
            err => err,
        }
    }

    #[must_use]
    pub fn wrap_request(self, name: &str) -> Self {
        match self {
            Self::Field(error) => Self::bad_request(name, [(error.field, error.error)]),
            err => err,
        }
    }

    #[must_use]
    pub fn wrap_request_nested<P, F>(self, name: &str, root_path: P) -> Self
    where
        P: IntoIterator<Item = F>,
        F: ToString,
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
            err => err,
        }
    }

    pub fn downcast_domain_ref<T: std::any::Any>(&self) -> Option<&T> {
        if let Self::Domain(err) = self {
            err.as_any().downcast_ref::<T>()
        } else {
            None
        }
    }

    pub fn downcast_domain<T: 'static + Clone>(&self) -> Option<T> {
        if let Self::Domain(err) = self {
            err.as_any().downcast_ref::<T>().cloned()
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
}
