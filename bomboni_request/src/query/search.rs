//! # Search query.
//!
//! Utility for working with fuzzy search queries.

use crate::{
    filter::Filter,
    ordering::{Ordering, OrderingTerm},
    schema::Schema,
};

use super::{
    error::{QueryError, QueryResult},
    page_token::{FilterPageToken, PageTokenBuilder},
    utility::{parse_query_filter, parse_query_ordering},
};

#[derive(Debug, Clone, PartialEq)]
pub struct SearchQuery<T = FilterPageToken> {
    pub query: String,
    pub page_size: i32,
    pub page_token: Option<T>,
    pub filter: Filter,
    pub ordering: Ordering,
}

/// Config for search query builder.
///
/// `primary_ordering_term` should probably never be `None`.
#[derive(Debug, Clone)]
pub struct SearchQueryConfig {
    pub max_query_length: Option<usize>,
    pub max_page_size: Option<i32>,
    pub default_page_size: i32,
    pub primary_ordering_term: Option<OrderingTerm>,
    pub max_filter_length: Option<usize>,
    pub max_ordering_length: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct SearchQueryBuilder<P: PageTokenBuilder> {
    schema: Schema,
    options: SearchQueryConfig,
    page_token_builder: P,
}

impl Default for SearchQueryConfig {
    fn default() -> Self {
        Self {
            max_query_length: None,
            max_page_size: None,
            default_page_size: 20,
            primary_ordering_term: None,
            max_filter_length: None,
            max_ordering_length: None,
        }
    }
}

impl<P: PageTokenBuilder> SearchQueryBuilder<P> {
    pub fn new(schema: Schema, options: SearchQueryConfig, page_token_builder: P) -> Self {
        Self {
            schema,
            options,
            page_token_builder,
        }
    }

    pub fn build(
        &self,
        query: &str,
        page_size: Option<i32>,
        page_token: Option<&str>,
        filter: Option<&str>,
        ordering: Option<&str>,
    ) -> QueryResult<SearchQuery<P::PageToken>> {
        if matches!(self.options.max_query_length, Some(max) if query.len() > max) {
            return Err(QueryError::QueryTooLong);
        }

        let filter = parse_query_filter(filter, &self.schema, self.options.max_filter_length)?;
        let mut ordering =
            parse_query_ordering(ordering, &self.schema, self.options.max_ordering_length)?;

        // Pre-insert primary ordering term.
        // This is needed for page tokens to work.
        if let Some(primary_ordering_term) = self.options.primary_ordering_term.as_ref() {
            if ordering
                .iter()
                .all(|term| term.name != primary_ordering_term.name)
            {
                ordering.insert(0, primary_ordering_term.clone());
            }
        }

        // Handle paging.
        let mut page_size = page_size.unwrap_or(self.options.default_page_size);
        if page_size < 0 {
            return Err(QueryError::InvalidPageSize);
        }
        if let Some(max_page_size) = self.options.max_page_size {
            // Intentionally clamp page size to max page size.
            if page_size > max_page_size {
                page_size = max_page_size;
            }
        }

        let page_token =
            if let Some(page_token) = page_token.filter(|page_token| !page_token.is_empty()) {
                Some(
                    self.page_token_builder
                        .parse(&filter, &ordering, page_token)?,
                )
            } else {
                None
            };

        Ok(SearchQuery {
            query: query.into(),
            filter,
            ordering,
            page_size,
            page_token,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        filter::error::FilterError, ordering::OrderingDirection,
        query::page_token::plain::PlainPageTokenBuilder, testing::schema::UserItem,
    };

    use super::*;

    #[test]
    fn it_works() {
        let qb = get_query_builder();
        let query = qb
            .build(
                "abc",
                Some(10_000),
                None,
                Some("displayName = \"John\""),
                Some("age desc"),
            )
            .unwrap();
        assert_eq!(query.page_size, 20);
        assert_eq!(query.filter.to_string(), "displayName = \"John\"");
        assert_eq!(query.ordering.to_string(), "id desc, age desc");
    }

    #[test]
    fn errors() {
        let q = get_query_builder();
        assert_eq!(
            q.build(
                &("a".repeat(100)),
                None,
                None,
                Some(&("a".repeat(100))),
                None
            )
            .unwrap_err(),
            QueryError::QueryTooLong
        );
        assert!(matches!(
            q.build("abc", Some(-1), None, None, None),
            Err(QueryError::InvalidPageSize)
        ));
        assert!(matches!(
            q.build("abc", Some(-1), None, None, None),
            Err(QueryError::InvalidPageSize)
        ));
        assert!(matches!(
            q.build("abc", None, None, Some("f!"), None).unwrap_err(),
            QueryError::FilterError(FilterError::Parse { start, end })
            if start == 1 && end == 1
        ));
        assert_eq!(
            q.build("abc", None, None, Some(&("a".repeat(100))), None)
                .unwrap_err(),
            QueryError::FilterTooLong
        );
        assert_eq!(
            q.build("abc", None, None, Some("lol"), None).unwrap_err(),
            QueryError::FilterSchemaMismatch
        );
        assert_eq!(
            q.build("abc", None, None, None, Some(&("a".repeat(100))))
                .unwrap_err(),
            QueryError::OrderingTooLong
        );
        assert_eq!(
            q.build("abc", None, None, None, Some("lol")).unwrap_err(),
            QueryError::OrderingSchemaMismatch
        );
    }

    fn get_query_builder() -> SearchQueryBuilder<PlainPageTokenBuilder> {
        SearchQueryBuilder::<PlainPageTokenBuilder>::new(
            UserItem::get_schema(),
            SearchQueryConfig {
                max_page_size: Some(20),
                default_page_size: 10,
                primary_ordering_term: Some(OrderingTerm {
                    name: "id".into(),
                    direction: OrderingDirection::Descending,
                }),
                max_query_length: Some(50),
                max_filter_length: Some(50),
                max_ordering_length: Some(50),
            },
            PlainPageTokenBuilder {},
        )
    }
}
