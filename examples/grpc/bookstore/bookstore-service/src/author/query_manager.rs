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

#[derive(Debug)]
pub struct AuthorQueryManager {
    author_repository: AuthorRepositoryArc,
    list_query_builder: PlainListQueryBuilder,
}

pub struct AuthorListResult {
    pub authors: Vec<Author>,
    pub next_page_token: Option<String>,
    pub total_size: i64,
}

impl AuthorQueryManager {
    pub fn new(author_repository: AuthorRepositoryArc) -> Self {
        AuthorQueryManager {
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

    pub async fn query_list(
        &self,
        query: ListQuery,
        show_deleted: bool,
    ) -> AppResult<AuthorListResult> {
        let author_list = self
            .author_repository
            .select_filtered(&query, show_deleted)
            .await?;
        let next_page_token = if let Some(next_item) = &author_list.next_item {
            Some(
                self.list_query_builder
                    .build_next_page_token(&query, next_item)
                    .unwrap(),
            )
        } else {
            None
        };

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

    pub fn list_query_builder(&self) -> &PlainListQueryBuilder {
        &self.list_query_builder
    }
}
