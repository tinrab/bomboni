//! The page token is used to determine the next page of results.
//! How it is used to query the database is implementation-specific.
//! One way is to filter IDs greater than the last item's ID of the previous page.
//! If the query parameters change, then the page token is invalid.
//! To ensure that a valid token is used, we can encrypt it along with the query parameters and decrypt it before use.
//! Encryption is also desirable to prevent users from guessing the next page of results, or to hide sensitive information.

use std::fmt::{self, Display, Formatter};

use crate::{filter::Filter, ordering::Ordering, schema::SchemaMapped};

use super::error::QueryResult;
pub mod aes256;
pub mod base64;
pub mod plain;
pub mod rsa;
mod utility;

/// A page token containing a filter.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterPageToken {
    pub filter: Filter,
}

impl FilterPageToken {
    pub fn new(filter: Filter) -> Self {
        Self { filter }
    }
}

pub trait PageTokenBuilder {
    type PageToken: Clone + ToString;

    /// Parse a page token.
    /// [`QueryError::InvalidPageToken`] is returned if the page token is invalid for any reason.
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
    /// [`QueryError::PageTokenFailure`] is returned if the page token could not be built.
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
