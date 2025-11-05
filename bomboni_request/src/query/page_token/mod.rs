//! The page token is used to determine the next page of results.
//!
//! How it is used to query the database is implementation-specific.
//! One way is to filter IDs greater than the last item's ID of the previous page.
//! If the query parameters change, then the page token is invalid.
//! To ensure that a valid token is used, we can encrypt it along with the query parameters and decrypt it before use.
//! Encryption is also desirable to prevent users from guessing the next page of results, or to hide sensitive information.

use std::fmt::{self, Display, Formatter};

use crate::{filter::Filter, ordering::Ordering, schema::SchemaMapped};

use super::error::QueryResult;
/// AES256 page token encoding.
pub mod aes256;

/// Base64 page token encoding.
pub mod base64;

/// Plain page token encoding.
pub mod plain;

/// RSA page token encoding.
pub mod rsa;
mod utility;

/// A page token containing a filter.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterPageToken {
    /// Filter.
    pub filter: Filter,
}

impl FilterPageToken {
    /// Creates a new filter page token.
    pub const fn new(filter: Filter) -> Self {
        Self { filter }
    }
}

/// Trait for building and parsing page tokens.
pub trait PageTokenBuilder {
    /// Page token type.
    type PageToken: Clone + ToString;

    /// Parse a page token.
    ///
    /// Returns [`crate::query::error::QueryError::InvalidPageToken`] if page token is invalid for any reason.
    ///
    /// # Errors
    ///
    /// # Errors
    ///
    /// Returns an error if the page token is invalid.
    fn parse(
        &self,
        filter: &Filter,
        ordering: &Ordering,
        salt: &[u8],
        page_token: &str,
    ) -> QueryResult<Self::PageToken>;

    /// Build a page token for the next page of results.
    ///
    /// Note that "last item" is not necessarily the last item of the page, but N+1th one.
    /// We can fetch `page_size+1` items from the database to determine if there are more results.
    /// [`crate::query::error::QueryError::PageTokenFailure`] is returned if the page token could not be built.
    ///
    /// # Errors
    ///
    /// Returns an error if token building fails.
    fn build_next<T: SchemaMapped>(
        &self,
        filter: &Filter,
        ordering: &Ordering,
        salt: &[u8],
        next_item: &T,
    ) -> QueryResult<String>;
}

impl Display for FilterPageToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.filter)
    }
}
