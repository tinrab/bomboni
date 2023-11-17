use crate::{
    filter::Filter,
    ordering::Ordering,
    query::error::{QueryError, QueryResult},
    schema::SchemaMapped,
};

use super::{utility::get_page_filter, FilterPageToken, PageTokenBuilder};

/// Plain text page token builder.
/// Used only for testing.
pub struct PlainPageTokenBuilder {}

impl PageTokenBuilder for PlainPageTokenBuilder {
    type PageToken = FilterPageToken;

    fn parse(
        &self,
        _filter: &Filter,
        _ordering: &Ordering,
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
        next_item: &T,
    ) -> QueryResult<String> {
        let page_filter = get_page_filter(ordering, next_item);
        if page_filter.is_empty() {
            return Err(QueryError::PageTokenFailure);
        }
        Ok(format!("{page_filter}"))
    }
}
