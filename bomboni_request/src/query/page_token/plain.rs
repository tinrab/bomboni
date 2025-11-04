use crate::{
    filter::Filter,
    ordering::Ordering,
    query::{
        error::{QueryError, QueryResult},
        page_token::{FilterPageToken, PageTokenBuilder, utility::get_page_filter},
    },
    schema::SchemaMapped,
};
use std::fmt::{self, Debug, Formatter};

/// Plain text page token builder.
/// Used only in insecure environments.
#[derive(Clone)]
pub struct PlainPageTokenBuilder {}

impl PageTokenBuilder for PlainPageTokenBuilder {
    type PageToken = FilterPageToken;

    fn parse(
        &self,
        _filter: &Filter,
        _ordering: &Ordering,
        _salt: &[u8],
        page_token: &str,
    ) -> QueryResult<Self::PageToken> {
        let page_filter = Filter::parse(page_token)?;
        Ok(Self::PageToken {
            filter: page_filter,
        })
    }

    fn build_next<T: SchemaMapped>(
        &self,
        _filter: &Filter,
        ordering: &Ordering,
        _salt: &[u8],
        next_item: &T,
    ) -> QueryResult<String> {
        let page_filter = get_page_filter(ordering, next_item);
        if page_filter.is_empty() {
            return Err(QueryError::PageTokenFailure);
        }
        Ok(format!("{page_filter}"))
    }
}

impl Debug for PlainPageTokenBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlainPageTokenBuilder").finish()
    }
}
