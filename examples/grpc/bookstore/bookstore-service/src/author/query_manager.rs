use bomboni_request::{
    error::CommonError,
    ordering::{OrderingDirection, OrderingTerm},
    query::{
        list::{ListQuery, ListQueryConfig, PlainListQueryBuilder},
        page_token::plain::PlainPageTokenBuilder,
    },
    schema::FunctionSchemaMap,
};
use bookstore_api::{
    model::author::{AuthorId, AuthorModel},
    v1::Author,
};

use crate::{author::repository::AuthorRepositoryArc, error::AppResult};

/// Query manager for author data operations.
///
/// Handles querying authors with support for pagination, filtering, and ordering.
/// Uses a repository pattern for data access and provides list query building.
#[derive(Debug)]
pub struct AuthorQueryManager {
    author_repository: AuthorRepositoryArc,
    list_query_builder: PlainListQueryBuilder,
}

/// Result of an author list query.
///
/// Contains the list of authors, pagination token, and total count.
pub struct AuthorListResult {
    /// The list of authors returned
    pub authors: Vec<Author>,
    /// Token for retrieving the next page of results
    pub next_page_token: Option<String>,
    /// Total number of authors matching the query
    pub total_size: i64,
}

impl AuthorQueryManager {
    /// Creates a new author query manager.
    ///
    /// # Arguments
    ///
    /// * `author_repository` - Repository for author data persistence
    pub fn new(author_repository: AuthorRepositoryArc) -> Self {
        Self {
            author_repository,
            list_query_builder: PlainListQueryBuilder::new(
                AuthorModel::get_schema(),
                FunctionSchemaMap::new(),
                ListQueryConfig {
                    max_page_size: Some(20),
                    default_page_size: 10,
                    primary_ordering_term: Some(OrderingTerm {
                        name: "id".into(),
                        direction: OrderingDirection::Descending,
                    }),
                    max_filter_length: Some(100),
                    max_ordering_length: Some(100),
                },
                PlainPageTokenBuilder {},
            ),
        }
    }

    /// Queries multiple authors by their IDs.
    ///
    /// # Errors
    ///
    /// Returns an error if any author is not found.
    pub async fn query_batch(&self, ids: &[AuthorId]) -> AppResult<Vec<Author>> {
        let authors = self.author_repository.select_multiple(ids).await?;
        if authors.len() != ids.len() {
            return Err(CommonError::ResourceNotFound.into());
        }

        Ok(authors
            .into_iter()
            .map(|record| AuthorModel::from(record).into())
            .collect())
    }

    /// Queries authors with pagination and filtering.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    ///
    /// # Panics
    ///
    /// Will panic if page token building fails.
    pub async fn query_list(
        &self,
        query: ListQuery,
        show_deleted: bool,
    ) -> AppResult<AuthorListResult> {
        let author_list = self
            .author_repository
            .select_filtered(&query, show_deleted)
            .await?;
        let next_page_token = author_list.next_item.as_ref().map(|next_item| {
            self.list_query_builder
                .build_next_page_token(&query, next_item)
                .unwrap()
        });

        Ok(AuthorListResult {
            authors: author_list
                .items
                .into_iter()
                .map(|record| AuthorModel::from(record).into())
                .collect(),
            next_page_token,
            total_size: author_list.total_size,
        })
    }

    /// Gets the list query builder.
    pub const fn list_query_builder(&self) -> &PlainListQueryBuilder {
        &self.list_query_builder
    }
}
