use crate::query::error::QueryError;
use bomboni_proto::google::protobuf::Any;
use bomboni_proto::google::rpc::bad_request::FieldViolation;
use bomboni_proto::google::rpc::BadRequest;
use bomboni_proto::google::rpc::{Code, Status};
use prost::{DecodeError, EncodeError};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
#[cfg_attr(
    all(
        target_family = "wasm",
        not(any(target_os = "emscripten", target_os = "wasi")),
        feature = "wasm",
    ),
    derive(bomboni_wasm::Wasm),
    wasm(
        bomboni_crate = crate::bomboni,
        into_wasm_abi,
        proxy { source = Status, try_from = RequestParse::parse },
    )
)]
pub enum RequestError {
    #[error("invalid `{name}` request")]
    BadRequest {
        name: String,
        violations: Vec<PathError>,
    },
    #[error(transparent)]
    Path(PathError),
    #[error("{0}")]
    Generic(GenericErrorBox),
    #[error("encode error: {0}")]
    Encode(#[from] EncodeError),
    #[error("decode error: {0}")]
    Decode(#[from] DecodeError),
}

pub type RequestResult<T> = Result<T, RequestError>;

#[derive(Debug)]
pub struct PathError {
    pub path: Vec<PathErrorStep>,
    pub error: GenericErrorBox,
}

#[derive(Debug, PartialEq)]
pub enum PathErrorStep {
    Field(String),
    Index(usize),
    Key(String),
}

#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
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

pub trait GenericError: Error {
    fn as_any(&self) -> &dyn std::any::Any;

    fn code(&self) -> Code {
        Code::InvalidArgument
    }

    fn details(&self) -> Vec<Any> {
        Vec::default()
    }
}

pub type GenericErrorBox = Box<dyn GenericError + Send + Sync>;

impl RequestError {
    #[must_use]
    pub fn bad_request<N, V, F, E>(name: N, violations: V) -> Self
    where
        N: Display,
        V: IntoIterator<Item = (F, E)>,
        F: Display,
        E: Into<GenericErrorBox>,
    {
        Self::BadRequest {
            name: name.to_string(),
            violations: violations
                .into_iter()
                .map(|(field, error)| PathError {
                    path: vec![PathErrorStep::Field(field.to_string())],
                    error: error.into(),
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn generic<E: Into<GenericErrorBox>>(error: E) -> Self {
        Self::Generic(error.into())
    }

    #[must_use]
    pub fn path<P, E>(path: P, error: E) -> Self
    where
        P: IntoIterator<Item = PathErrorStep>,
        E: Into<GenericErrorBox>,
    {
        Self::Path(PathError {
            path: path.into_iter().collect(),
            error: error.into(),
        })
    }

    #[must_use]
    pub fn field<F, E>(field: F, error: E) -> Self
    where
        F: Display,
        E: Into<GenericErrorBox>,
    {
        Self::path([PathErrorStep::Field(field.to_string())], error)
    }

    #[must_use]
    pub fn field_index<F, E>(field: F, index: usize, error: E) -> Self
    where
        F: Display,
        E: Into<GenericErrorBox>,
    {
        Self::path(
            [
                PathErrorStep::Field(field.to_string()),
                PathErrorStep::Index(index),
            ],
            error,
        )
    }

    #[must_use]
    pub fn field_key<F, K, E>(field: F, key: K, error: E) -> Self
    where
        F: Display,
        K: Display,
        E: Into<GenericErrorBox>,
    {
        Self::path(
            [
                PathErrorStep::Field(field.to_string()),
                PathErrorStep::Key(key.to_string()),
            ],
            error,
        )
    }

    #[must_use]
    pub fn field_parse<F, E>(field: F, error: E) -> Self
    where
        F: Display,
        E: Into<GenericErrorBox>,
    {
        Self::path(PathError::parse_path(field.to_string()), error)
    }

    #[must_use]
    pub fn index<E>(index: usize, error: E) -> Self
    where
        E: Into<GenericErrorBox>,
    {
        Self::path([PathErrorStep::Index(index)], error)
    }

    #[must_use]
    pub fn key<K, E>(key: K, error: E) -> Self
    where
        K: Display,
        E: Into<GenericErrorBox>,
    {
        Self::path([PathErrorStep::Key(key.to_string())], error)
    }

    #[must_use]
    pub fn wrap_path<P>(self, path: P) -> Self
    where
        P: IntoIterator<Item = PathErrorStep>,
    {
        let mut path: Vec<_> = path.into_iter().collect();
        match self {
            Self::Path(error) => PathError {
                path: {
                    path.extend(error.path);
                    path
                },
                error: error.error,
            }
            .into(),
            Self::Generic(error) => Self::path(path, error),
            err => panic!("cannot wrap error path `{path:?}` for: {err:?}"),
        }
    }

    #[must_use]
    pub fn insert_path<P>(self, path: P, index: usize) -> Self
    where
        P: IntoIterator<Item = PathErrorStep>,
    {
        let path: Vec<_> = path.into_iter().collect();
        match self {
            Self::Path(mut error) => PathError {
                path: {
                    let tail: Vec<_> = error.path.splice(index.., path).collect();
                    error.path.extend(tail);
                    error.path
                },
                error: error.error,
            }
            .into(),
            Self::Generic(error) => Self::path(path, error),
            err => panic!("cannot insert error path `{path:?}` for: {err:?}"),
        }
    }

    #[must_use]
    pub fn wrap_field<F: Display>(self, field: F) -> Self {
        self.wrap_path([PathErrorStep::Field(field.to_string())])
    }

    #[must_use]
    pub fn wrap_index(self, index: usize) -> Self {
        self.wrap_path([PathErrorStep::Index(index)])
    }

    #[must_use]
    pub fn wrap_key<K: Display>(self, key: K) -> Self {
        self.wrap_path([PathErrorStep::Key(key.to_string())])
    }

    #[must_use]
    pub fn wrap_field_index<F>(self, field: F, index: usize) -> Self
    where
        F: Display,
    {
        self.wrap_path([
            PathErrorStep::Field(field.to_string()),
            PathErrorStep::Index(index),
        ])
    }

    #[must_use]
    pub fn wrap_field_key<F, K>(self, field: F, key: K) -> Self
    where
        F: Display,
        K: Display,
    {
        self.wrap_path([
            PathErrorStep::Field(field.to_string()),
            PathErrorStep::Key(key.to_string()),
        ])
    }

    #[must_use]
    pub fn wrap_request<N: Display>(self, name: N) -> Self {
        match self {
            Self::Path(error) => Self::bad_request(name, [(error.path_to_string(), error.error)]),
            Self::Generic(error) => {
                if let Some(error) = error.as_any().downcast_ref::<QueryError>() {
                    #[allow(trivial_casts)]
                    Self::bad_request(
                        name,
                        [(
                            error.get_violating_field_name(),
                            Box::new(error.clone()) as GenericErrorBox,
                        )],
                    )
                } else {
                    RequestError::Generic(error)
                }
            }
            error => error,
        }
    }

    pub fn code(&self) -> Code {
        match self {
            Self::Encode(_) | Self::Decode(_) | Self::BadRequest { .. } => Code::InvalidArgument,
            Self::Path(error) => error.code(),
            Self::Generic(error) => error.code(),
        }
    }

    pub fn details(&self) -> Vec<Any> {
        match self {
            Self::BadRequest { violations, .. } => vec![BadRequest {
                field_violations: violations
                    .iter()
                    .map(|error| FieldViolation {
                        field: error.path_to_string(),
                        description: error.error.to_string(),
                    })
                    .collect(),
            }
            .try_into()
            .unwrap()],
            Self::Path(error) => error.details(),
            Self::Generic(error) => error.details(),
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

impl PathError {
    pub fn code(&self) -> Code {
        self.error.code()
    }

    pub fn details(&self) -> Vec<Any> {
        self.error.details()
    }

    pub fn path_to_string(&self) -> String {
        let mut path = String::new();
        for (i, step) in self.path.iter().enumerate() {
            match step {
                PathErrorStep::Field(field) => {
                    if i == 0 {
                        path.push_str(field);
                    } else {
                        path.push_str(&format!(".{field}"));
                    }
                }
                PathErrorStep::Index(index) => path.push_str(&format!("[{index}]")),
                PathErrorStep::Key(key) => path.push_str(&format!("{{{key}}}")),
            }
        }
        path
    }

    pub fn parse_path<P: AsRef<str>>(path: P) -> Vec<PathErrorStep> {
        let parts: Vec<_> = path.as_ref().split('.').collect();
        let mut steps = Vec::with_capacity(parts.len());
        for part in parts {
            let part = part.trim();
            if let Some(index) = part.find('[') {
                let field = &part[..index];
                let index = part[index + 1..part.len() - 1].parse().unwrap();
                steps.push(PathErrorStep::Field(field.to_string()));
                steps.push(PathErrorStep::Index(index));
            } else if let Some(index) = part.find('{') {
                let key = &part[index + 1..part.len() - 1];
                steps.push(PathErrorStep::Key(key.to_string()));
            } else {
                steps.push(PathErrorStep::Field(part.to_string()));
            }
        }
        steps
    }
}

impl Error for PathError {}

impl From<PathError> for RequestError {
    fn from(err: PathError) -> Self {
        Self::Path(err)
    }
}

impl Display for PathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "field `{}` error: `{}`",
            self.path_to_string(),
            self.error
        )
    }
}

impl Display for PathErrorStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Field(field) => write!(f, "{field}"),
            Self::Index(index) => write!(f, "[{index}]"),
            Self::Key(key) => write!(f, "{{{key}}}"),
        }
    }
}

impl GenericError for CommonError {
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

impl<T> From<T> for GenericErrorBox
where
    T: 'static + GenericError + Send + Sync,
{
    fn from(err: T) -> Self {
        Box::new(err)
    }
}

impl<T: 'static + GenericError + Send + Sync> From<T> for RequestError {
    fn from(err: T) -> Self {
        RequestError::Generic(Box::new(err))
    }
}

pub trait RequestErrorExt {
    fn wrap<F: Display>(self, field: F) -> RequestError;

    fn wrap_index(self, index: usize) -> RequestError;

    fn wrap_key<K: Display>(self, key: K) -> RequestError;

    fn wrap_field_index<F: Display>(self, field: F, index: usize) -> RequestError;

    fn wrap_field_key<F: Display, K: Display>(self, field: F, key: K) -> RequestError;

    fn wrap_request<N: Display>(self, name: N) -> RequestError;
}

impl<T> RequestErrorExt for T
where
    T: 'static + GenericError + Send + Sync,
{
    fn wrap<F: Display>(self, field: F) -> RequestError {
        RequestError::generic(self).wrap_field(field)
    }

    fn wrap_index(self, index: usize) -> RequestError {
        RequestError::generic(self).wrap_index(index)
    }

    fn wrap_key<K: Display>(self, key: K) -> RequestError {
        RequestError::generic(self).wrap_key(key)
    }

    fn wrap_field_index<F: Display>(self, field: F, index: usize) -> RequestError {
        RequestError::generic(self).wrap_field_index(field, index)
    }

    fn wrap_field_key<F: Display, K: Display>(self, field: F, key: K) -> RequestError {
        RequestError::generic(self).wrap_field_key(field, key)
    }

    fn wrap_request<N: Display>(self, name: N) -> RequestError {
        RequestError::generic(self).wrap_request(name)
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

    #[test]
    fn field_paths() {
        assert_eq!(
            RequestError::generic(CommonError::NotFound)
                .wrap_field("value")
                .wrap_index(42)
                .wrap_field("root")
                .to_string(),
            "field `root[42].value` error: `not found`"
        );
        assert!(matches!(
            RequestError::generic(CommonError::NotFound)
                .wrap_index(42)
                .wrap_field("value")
                .wrap_request("Test"),
            RequestError::BadRequest { name, violations }
            if name == "Test" && violations.len() == 1
                && violations[0].to_string() == "field `value[42]` error: `not found`"
        ));
        assert!(matches!(
            CommonError::InvalidId.wrap("id").wrap_request("Test"),
            RequestError::BadRequest { name, violations }
            if name == "Test" && violations.len() == 1
                && violations[0].to_string() == "field `id` error: `invalid ID format`"
        ));
    }

    #[test]
    fn parse_error_field_path() {
        assert_eq!(
            PathError::parse_path("test.x.field[42].y.{key}.value"),
            vec![
                PathErrorStep::Field("test".into()),
                PathErrorStep::Field("x".into()),
                PathErrorStep::Field("field".into()),
                PathErrorStep::Index(42),
                PathErrorStep::Field("y".into()),
                PathErrorStep::Key("key".into()),
                PathErrorStep::Field("value".into()),
            ],
        );
    }
}
