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
    model::book::{BookId, BookModel},
    v1::Book,
};

use super::repository::BookRepositoryArc;

#[derive(Debug, Clone)]
pub struct BookQueryManager {
    book_repository: BookRepositoryArc,
    list_query_builder: PlainListQueryBuilder,
}

pub struct BookListResult {
    pub books: Vec<Book>,
    pub next_page_token: Option<String>,
    pub total_size: i32,
}

impl BookQueryManager {
    pub fn new(book_repository: BookRepositoryArc) -> Self {
        BookQueryManager {
            book_repository,
            list_query_builder: PlainListQueryBuilder::new(
                BookModel::get_schema(),
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

    pub async fn query_single(
        &self,
        id: BookId,
    ) -> Result<Book, Box<dyn std::error::Error + Send + Sync>> {
        let record = self
            .book_repository
            .select(id)
            .await?
            .ok_or(CommonError::ResourceNotFound)?;

        Ok(BookModel::from(record).into())
    }

    pub async fn query_list(
        &self,
        query: ListQuery,
        show_deleted: bool,
    ) -> Result<BookListResult, Box<dyn std::error::Error + Send + Sync>> {
        let book_list = self
            .book_repository
            .select_filtered(&query, show_deleted)
            .await?;

        let next_page_token = if let Some(next_item) = &book_list.next_item {
            Some(
                self.list_query_builder
                    .build_next_page_token(&query, next_item)
                    .unwrap(),
            )
        } else {
            None
        };

        Ok(BookListResult {
            books: book_list
                .items
                .into_iter()
                .map(|record| BookModel::from(record).into())
                .collect(),
            next_page_token,
            total_size: 0,
        })
    }

    pub fn list_query_builder(&self) -> &PlainListQueryBuilder {
        &self.list_query_builder
    }
}
