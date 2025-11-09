use bomboni_request::{
    error::CommonError,
    ordering::{OrderingDirection, OrderingTerm},
    query::{
        list::{ListQuery, ListQueryConfig, PlainListQueryBuilder},
        page_token::plain::PlainPageTokenBuilder,
    },
    schema::FunctionSchemaMap,
};
use bookstore_api::model::book::{BookId, BookModel};
use bookstore_api::v1::Book;

use crate::{book::repository::BookRepositoryArc, error::AppResult};

/// Query manager for book data operations.
///
/// Handles querying books with support for pagination, filtering, and ordering.
/// Uses a repository pattern for data access and provides list query building.
#[derive(Debug)]
pub struct BookQueryManager {
    book_repository: BookRepositoryArc,
    list_query_builder: PlainListQueryBuilder,
}

/// Result of a book list query.
///
/// Contains the list of books, pagination token, and total count.
pub struct BookListResult {
    /// The list of books returned
    pub books: Vec<Book>,
    /// Token for retrieving the next page of results
    pub next_page_token: Option<String>,
    /// Total number of books matching the query
    pub total_size: i64,
}

impl BookQueryManager {
    /// Creates a new book query manager.
    ///
    /// # Arguments
    ///
    /// * `book_repository` - Repository for book data persistence
    pub fn new(book_repository: BookRepositoryArc) -> Self {
        Self {
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

    /// Queries multiple books by their IDs.
    ///
    /// # Errors
    ///
    /// Returns an error if any book is not found.
    pub async fn query_batch(&self, ids: &[BookId]) -> AppResult<Vec<Book>> {
        let books = self.book_repository.select_multiple(ids).await?;
        if books.len() != ids.len() {
            return Err(CommonError::ResourceNotFound.into());
        }

        Ok(books
            .into_iter()
            .map(|record| BookModel::from(record).into())
            .collect())
    }

    /// Queries books with pagination and filtering.
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
    ) -> AppResult<BookListResult> {
        let book_list = self
            .book_repository
            .select_filtered(&query, show_deleted)
            .await?;
        let next_page_token = book_list.next_item.as_ref().map(|next_item| {
            self.list_query_builder
                .build_next_page_token(&query, next_item)
                .unwrap()
        });

        Ok(BookListResult {
            books: book_list
                .items
                .into_iter()
                .map(|record| BookModel::from(record).into())
                .collect(),
            next_page_token,
            total_size: book_list.total_size,
        })
    }

    /// Gets the list query builder.
    ///
    /// Returns a reference to the underlying list query builder
    /// used for parsing and building list queries.
    pub const fn list_query_builder(&self) -> &PlainListQueryBuilder {
        &self.list_query_builder
    }
}
