//! # List query.
//!
//! Utility for working with Google AIP standard List method [1].
//!
//! [1]: https://google.aip.dev/132

use crate::{
    filter::Filter,
    ordering::{Ordering, OrderingTerm},
    query::{
        error::{QueryError, QueryResult},
        page_token::{
            aes256::Aes256PageTokenBuilder, base64::Base64PageTokenBuilder,
            plain::PlainPageTokenBuilder, rsa::RsaPageTokenBuilder, FilterPageToken,
            PageTokenBuilder,
        },
        utility::{parse_query_filter, parse_query_ordering},
    },
    schema::{Schema, SchemaMapped},
};

/// Represents a list query.
/// List queries list paged, filtered and ordered items.
#[derive(Debug, Clone, PartialEq)]
pub struct ListQuery<T: Clone + ToString = FilterPageToken> {
    pub page_size: i32,
    pub page_token: Option<T>,
    pub filter: Filter,
    pub ordering: Ordering,
}

/// Config for list query builder.
///
/// `primary_ordering_term` should probably never be `None`.
/// If the request does not contain an `order_by` field, usage of this function should pre-insert one.
/// The default ordering term can the primary key of the schema item.
/// If ordering is not specified, then behavior of query's page tokens [`PageToken`] is undefined.
#[derive(Debug, Clone)]
pub struct ListQueryConfig {
    pub max_page_size: Option<i32>,
    pub default_page_size: i32,
    pub primary_ordering_term: Option<OrderingTerm>,
    pub max_filter_length: Option<usize>,
    pub max_ordering_length: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ListQueryBuilder<P: PageTokenBuilder> {
    schema: Schema,
    options: ListQueryConfig,
    page_token_builder: P,
}

pub type PlainListQueryBuilder = ListQueryBuilder<PlainPageTokenBuilder>;
pub type Aes256ListQueryBuilder = ListQueryBuilder<Aes256PageTokenBuilder>;
pub type Base64ListQueryBuilder = ListQueryBuilder<Base64PageTokenBuilder>;
pub type RsaListQueryBuilder = ListQueryBuilder<RsaPageTokenBuilder>;

impl ListQuery {
    pub fn make_salt(page_size: i32) -> Vec<u8> {
        page_size.to_be_bytes().to_vec()
    }
}

impl Default for ListQueryConfig {
    fn default() -> Self {
        Self {
            max_page_size: None,
            default_page_size: 20,
            primary_ordering_term: None,
            max_filter_length: None,
            max_ordering_length: None,
        }
    }
}

impl<P: PageTokenBuilder> ListQueryBuilder<P> {
    pub fn new(schema: Schema, options: ListQueryConfig, page_token_builder: P) -> Self {
        Self {
            schema,
            options,
            page_token_builder,
        }
    }

    pub fn build(
        &self,
        page_size: Option<i32>,
        page_token: Option<&str>,
        filter: Option<&str>,
        ordering: Option<&str>,
    ) -> QueryResult<ListQuery<P::PageToken>> {
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
                Some(self.page_token_builder.parse(
                    &filter,
                    &ordering,
                    &ListQuery::make_salt(page_size),
                    page_token,
                )?)
            } else {
                None
            };

        Ok(ListQuery {
            page_size,
            page_token,
            filter,
            ordering,
        })
    }

    pub fn build_next_page_token<T: SchemaMapped>(
        &self,
        query: &ListQuery<P::PageToken>,
        next_item: &T,
    ) -> QueryResult<String> {
        self.page_token_builder.build_next(
            &query.filter,
            &query.ordering,
            &ListQuery::make_salt(query.page_size),
            next_item,
        )
    }

    pub fn page_token_builder(&self) -> &P {
        &self.page_token_builder
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
        assert!(matches!(
            q.build(Some(-1), None, None, None),
            Err(QueryError::InvalidPageSize)
        ));
        assert!(matches!(
            q.build(Some(-1), None, None, None),
            Err(QueryError::InvalidPageSize)
        ));
        assert!(matches!(
            q.build(None, None, Some("f!"), None).unwrap_err(),
            QueryError::FilterError(FilterError::Parse { start, end })
            if start == 1 && end == 1
        ));
        assert_eq!(
            q.build(None, None, Some(&("a".repeat(100))), None)
                .unwrap_err(),
            QueryError::FilterTooLong
        );
        assert_eq!(
            q.build(None, None, Some("lol"), None).unwrap_err(),
            QueryError::FilterSchemaMismatch
        );
        assert_eq!(
            q.build(None, None, None, Some(&("a".repeat(100))))
                .unwrap_err(),
            QueryError::OrderingTooLong
        );
        assert_eq!(
            q.build(None, None, None, Some("lol")).unwrap_err(),
            QueryError::OrderingSchemaMismatch
        );
    }

    #[test]
    fn page_tokens() {
        let qb = get_query_builder();
        let last_item: UserItem = UserItem {
            id: "1337".into(),
            display_name: "John".into(),
            age: 14000,
        };

        macro_rules! assert_page_token {
            ($filter1:expr, $ordering1:expr, $filter2:expr, $ordering2:expr, $expected_token:expr $(,)?) => {{
                let first_page = qb.build(Some(3), None, $filter1, $ordering1).unwrap();
                let next_page_token = qb
                    .page_token_builder
                    .build_next(&first_page.filter, &first_page.ordering, &[], &last_item)
                    .unwrap();
                let next_page: ListQuery = qb
                    .build(Some(3), Some(&next_page_token), $filter2, $ordering2)
                    .unwrap();
                assert_eq!(
                    next_page.page_token.unwrap().filter.to_string(),
                    $expected_token
                );
            }};
        }

        assert_page_token!(
            Some(r#"displayName = "John""#),
            None,
            Some(r#"displayName = "John""#),
            None,
            r#"id <= "1337""#,
        );
        assert_page_token!(
            None,
            Some("id desc, age desc"),
            None,
            Some("id desc, age desc"),
            r#"id <= "1337" AND age <= 14000"#,
        );
        assert_page_token!(
            None,
            Some("id desc, age asc"),
            None,
            Some("id desc, age desc"),
            r#"id <= "1337" AND age >= 14000"#,
        );
    }

    fn get_query_builder() -> ListQueryBuilder<PlainPageTokenBuilder> {
        ListQueryBuilder::<PlainPageTokenBuilder>::new(
            UserItem::get_schema(),
            ListQueryConfig {
                max_page_size: Some(20),
                default_page_size: 10,
                primary_ordering_term: Some(OrderingTerm {
                    name: "id".into(),
                    direction: OrderingDirection::Descending,
                }),
                max_filter_length: Some(50),
                max_ordering_length: Some(50),
            },
            PlainPageTokenBuilder {},
        )
    }
}
