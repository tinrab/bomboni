use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Display, Formatter},
};

use bomboni::proto::google::{protobuf::Any, rpc::ErrorInfo};
use bomboni::request::error::{CommonError, GenericError};
use grpc_common::{COMMON_DOMAIN, get_common_error_reason, proto::common_error::CommonErrorReason};
use paste::paste;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    model::{author::AuthorId, book::BookId},
    v1::errors::{author_error::AuthorErrorReason, book_error::BookErrorReason},
};

/// Error types for the bookstore service.
///
/// This module defines comprehensive error handling for the bookstore API,
/// including domain-specific errors, metadata, and conversion utilities.

#[derive(Error, Debug, PartialEq, Eq)]
#[error(transparent)]
pub enum BookstoreError {
    /// Book-related errors.
    Book(#[from] BookError),
    /// Author-related errors.
    Author(#[from] AuthorError),
}

/// Metadata for bookstore errors.
///
/// This structure provides additional context for errors, including
/// relevant IDs and identifiers that can help with debugging and
/// error reporting.
#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookstoreErrorMetadata {
    /// The ID of the book associated with the error, if applicable.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "metadata_field_serde"
    )]
    pub book_id: Option<BookId>,
    /// The ID of the author associated with the error, if applicable.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "metadata_field_serde"
    )]
    pub author_id: Option<AuthorId>,
    /// The ISBN associated with the error, if applicable.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "metadata_field_serde"
    )]
    pub isbn: Option<String>,
}

/// Result type for bookstore operations.
pub type BookstoreResult<T> = Result<T, BookstoreError>;

/// The error domain for bookstore-specific errors.
pub const BOOKSTORE_ERROR_DOMAIN: &str = "bookstore.rabzelj.com";

impl GenericError for BookstoreError {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

mod metadata_field_serde {
    use std::str::FromStr;

    use super::Deserialize;

    #[allow(clippy::ref_option)]
    pub fn serialize<F, S>(value: &Option<F>, serializer: S) -> Result<S::Ok, S::Error>
    where
        F: ToString,
        S: serde::Serializer,
    {
        if let Some(value) = value {
            serializer.serialize_str(&value.to_string())
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, F, D>(
        deserializer: D,
    ) -> Result<Option<F>, <D as serde::Deserializer<'de>>::Error>
    where
        F: FromStr,
        D: serde::Deserializer<'de>,
    {
        let value = Option::<String>::deserialize(deserializer)?;
        value.map_or_else(
            || Ok(None),
            |value| {
                F::from_str(&value)
                    .map(Some)
                    .map_err(|_| serde::de::Error::custom("failed to parse metadata field"))
            },
        )
    }
}

impl BookstoreErrorMetadata {
    /// Converts the metadata to a map.
    ///
    /// # Panics
    ///
    /// Will panic if JSON serialization or deserialization fails.
    pub fn to_map(&self) -> BTreeMap<String, String> {
        let value = serde_json::to_value(self).unwrap();
        serde_json::from_value(value).unwrap()
    }

    /// Creates metadata from a map.
    ///
    /// # Arguments
    ///
    /// * `map` - The map containing metadata fields
    ///
    /// # Returns
    ///
    /// Some(BookstoreErrorMetadata) if parsing succeeds, None otherwise
    pub fn from_map(map: BTreeMap<String, String>) -> Option<Self> {
        let value = serde_json::to_value(map).ok()?;
        serde_json::from_value(value).ok()
    }
}

impl Display for BookstoreErrorMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_map().fmt(f)
    }
}

impl Debug for BookstoreErrorMetadata {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut d = f.debug_struct("BookstoreErrorMetadata");

        macro_rules! debug_fields {
            ($($field:ident),* $(,)?) => {
                $(
                    if let Some(value) = &self.$field {
                        d.field(stringify!($field), value);
                    }
                )*
            };
        }

        debug_fields![book_id, author_id, isbn];

        d.finish()
    }
}

/// Parsed error reason for bookstore errors.
///
/// This enum combines common error reasons with domain-specific
/// error reasons for books and authors.
pub enum ParsedBookstoreErrorReason {
    /// Common error reasons applicable to all domains.
    Common(CommonErrorReason),
    /// Book-specific error reasons.
    Book(BookErrorReason),
    /// Author-specific error reasons.
    Author(AuthorErrorReason),
}

macro_rules! impl_bookstore_error_reason_variants {
    ($( ($variant:ident, $type:ty) $(,)? )* ) => {
        $(
            impl From<$type> for ParsedBookstoreErrorReason {
                fn from(value: $type) -> Self {
                    ParsedBookstoreErrorReason::$variant(value)
                }
            }
        )*
    };
}

impl_bookstore_error_reason_variants![
    (Common, CommonErrorReason),
    (Book, BookErrorReason),
    (Author, AuthorErrorReason),
];

macro_rules! convert_bookstore_error_reason {
    ($reason:ident, $type:ty, $kind:ident) => {{
        let parsed_reason: ParsedBookstoreErrorReason = $reason.into();
        let mut common_reason = CommonErrorReason::Unspecified;
        let mut domain_reason = <$type>::Unspecified;
        match parsed_reason {
            ParsedBookstoreErrorReason::Common(parsed_reason) => {
                common_reason = parsed_reason;
            }
            ParsedBookstoreErrorReason::$kind(parsed_reason) => {
                domain_reason = parsed_reason;
            }
            _ => unreachable!(),
        }
        (domain_reason, common_reason)
    }};
}

macro_rules! impl_domain_error {
    ($name:ident) => {
        paste! {
            /// Domain-specific error for the bookstore service.
            #[derive(Error, Debug, Clone, PartialEq, Eq)]
            pub struct [<$name Error>] {
                /// The specific error reason for this domain.
                pub reason: [<$name ErrorReason>],
                /// Optional error message providing additional context.
                pub message: Option<String>,
                /// The common error reason if applicable.
                pub common_reason: CommonErrorReason,
                /// The common error if applicable.
                pub common_error: Option<CommonError>,
                /// Optional metadata providing additional error context.
                pub metadata: Option<BookstoreErrorMetadata>,
            }
        }

        paste! {
            /// Result type for domain-specific operations.
            pub type [<$name Result>]<T> = Result<T, [<$name Error>]>;
        }

        paste! {
            #[allow(dead_code)]
            impl [<$name Error>] {
                /// Creates a new error with the given reason.
                pub fn new<R: Into<ParsedBookstoreErrorReason>>(reason: R) -> Self {
                    let (reason, common_reason) =
                        convert_bookstore_error_reason!(reason, [<$name ErrorReason>], $name);
                    Self {
                        reason,
                        common_reason,
                        metadata: None,
                        message: None,
                        common_error: None,
                    }
                }

                /// Creates a new error from a common error.
                pub const fn new_common(common_error: CommonError) -> Self {
                    [<$name Error>] {
                        reason: [<$name ErrorReason>]::Unspecified,
                        common_reason: get_common_error_reason(&common_error),
                        metadata: None,
                        message: None,
                        common_error: Some(common_error),
                    }
                }

                /// Creates a new error with metadata.
                pub fn new_with_metadata<R: Into<ParsedBookstoreErrorReason>>(
                    reason: R,
                    metadata: BookstoreErrorMetadata,
                ) -> Self {
                    let (reason, common_reason) =
                        convert_bookstore_error_reason!(reason, [<$name ErrorReason>], $name);
                    Self {
                        reason,
                        common_reason,
                        metadata: Some(metadata),
                        message: None,
                        common_error: None,
                    }
                }

                /// Creates a new error with a custom message.
                pub fn new_with_message<R: Into<ParsedBookstoreErrorReason>, S: ToString>(
                    reason: R,
                    message: S,
                ) -> Self {
                    let (reason, common_reason) =
                        convert_bookstore_error_reason!(reason, [<$name ErrorReason>], $name);
                    Self {
                        reason,
                        common_reason,
                        metadata: None,
                        message: Some(message.to_string()),
                        common_error: None,
                    }
                }

                /// Sets the reason for the error.
                #[must_use]
                pub const fn with_reason(mut self, reason: [<$name ErrorReason>]) -> Self {
                    self.reason = reason;
                    self
                }

                /// Sets the message for the error.
                #[must_use]
                pub fn with_message<S: ToString>(mut self, message: S) -> Self {
                    self.message = Some(message.to_string());
                    self
                }

                fn modify_metadata<F: FnMut(&mut BookstoreErrorMetadata)>(
                    mut self,
                    mut f: F,
                ) -> Self {
                    self.metadata = if let Some(mut metadata) = self.metadata {
                        f(&mut metadata);
                        Some(metadata)
                    } else {
                        let mut metadata = BookstoreErrorMetadata::default();
                        f(&mut metadata);
                        Some(metadata)
                    };
                    self
                }
            }
        }

        impl Display for paste! { [<$name Error>] } {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                if self.common_reason != CommonErrorReason::Unspecified {
                    write!(f, "{}", self.common_reason.as_str_name())?;
                } else {
                    write!(f, "{}", self.reason.as_str_name())?;
                }

                if let Some(message) = self.message.as_ref() {
                    write!(f, ": {}", message)?;
                } else if let Some(common_error) = self.common_error.as_ref() {
                    write!(f, ": {}", common_error)?;
                } else if let Some(metadata) = self.metadata.as_ref() {
                    write!(f, ": {}", metadata)?;
                }

                Ok(())
            }
        }

        impl GenericError for paste! { [<$name Error>] } {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn details(&self) -> Vec<Any> {
                let mut details = Vec::new();
                if self.common_reason != CommonErrorReason::Unspecified {
                    details.push(
                        Any::from_msg(&ErrorInfo {
                            reason: self.common_reason.as_str_name().into(),
                            domain: COMMON_DOMAIN.into(),
                            metadata: self.metadata.clone().unwrap_or_default().to_map(),
                        })
                        .unwrap(),
                    );
                }
                if self.reason != paste! { [<$name ErrorReason>]::Unspecified } {
                    details.push(
                        Any::from_msg(&ErrorInfo {
                            reason: self.reason.as_str_name().into(),
                            domain: format!("{}/{}", BOOKSTORE_ERROR_DOMAIN, stringify!($name)),
                            metadata: self.metadata.clone().unwrap_or_default().to_map(),
                        })
                        .unwrap(),
                    );
                }
                details
            }
        }

        paste! {
            impl From<CommonError> for [<$name Error>] {
                fn from(common_error: CommonError) -> Self {
                    Self {
                        reason: [<$name ErrorReason>]::Unspecified,
                        message: None,
                        common_reason: get_common_error_reason(&common_error),
                        common_error: Some(common_error),
                        metadata: None,
                    }
                }
            }
        }
    };
}

impl_domain_error!(Book);
impl_domain_error!(Author);

macro_rules! impl_bookstore_metadata_field {
    ($ident:ident, $type:ty, $convert:ident) => {
        paste! {
            /// Creates a new error with the specified metadata field.
            pub fn $ident<R: Into<ParsedBookstoreErrorReason>>(value: $type, reason: R) -> Self {
                Self::new_with_metadata(
                    reason,
                    BookstoreErrorMetadata {
                        $ident: Some( value.$convert () ),
                        ..Default::default()
                    },
                )
            }
            /// Adds or updates the specified metadata field.
            #[must_use]
            pub fn [<with_ $ident>] (self, value: $type) -> Self {
                self.modify_metadata(|metadata| {
                    // Don't overwrite existing metadata
                    if metadata.$ident.is_none() {
                        metadata.$ident = Some( value.$convert () );
                    }
                })
            }
        }
    };
}

impl BookError {
    impl_bookstore_metadata_field!(book_id, &BookId, clone);
    impl_bookstore_metadata_field!(isbn, &str, into);
}

impl AuthorError {
    impl_bookstore_metadata_field!(author_id, &AuthorId, clone);
}
