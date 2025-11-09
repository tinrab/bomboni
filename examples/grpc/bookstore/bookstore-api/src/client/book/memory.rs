use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Response, Status, metadata::MetadataMap};

use bomboni::common::date_time::UtcDateTime;
use bomboni::request::parse::RequestParse;

use crate::client::book::BookClient;
use crate::model::book::BookId;
use crate::model::book_service::{
    ParsedCreateBookRequest, ParsedDeleteBookRequest, ParsedGetBookRequest, ParsedUpdateBookRequest,
};
use crate::v1::Book;
use crate::v1::{
    CreateBookRequest, DeleteBookRequest, GetBookRequest, ListBooksRequest, ListBooksResponse,
    UpdateBookRequest,
};

/// In-memory implementation of the book client.
#[derive(Debug, Clone)]
pub struct MemoryBookClient {
    books: Arc<RwLock<HashMap<BookId, Book>>>,
}

impl Default for MemoryBookClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBookClient {
    /// Creates a new empty memory book client.
    pub fn new() -> Self {
        Self {
            books: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new memory book client with initial data.
    ///
    /// # Arguments
    ///
    /// * `books` - Vector of books to initialize the client with
    ///
    /// # Notes
    ///
    /// Books with invalid names will be filtered out during initialization.
    pub fn with_data(books: Vec<Book>) -> Self {
        let book_map = books
            .into_iter()
            .filter_map(|book| {
                let id = BookId::parse_name(&book.name)?;
                Some((id, book))
            })
            .collect();
        Self {
            books: Arc::new(RwLock::new(book_map)),
        }
    }

    /// Retrieves all books stored in memory.
    ///
    /// # Returns
    ///
    /// A vector containing all stored books
    pub async fn get_books(&self) -> Vec<Book> {
        self.books.read().await.values().cloned().collect()
    }
}

#[async_trait::async_trait]
impl BookClient for MemoryBookClient {
    async fn get_book(
        &self,
        request: GetBookRequest,
        _metadata: MetadataMap,
    ) -> Result<Response<Book>, Status> {
        let request = ParsedGetBookRequest::parse(request)?;
        self.books.read().await.get(&request.id).map_or_else(
            || Err(Status::not_found("Book not found")),
            |book| Ok(Response::new(book.clone())),
        )
    }

    async fn list_books(
        &self,
        _request: ListBooksRequest,
        _metadata: MetadataMap,
    ) -> Result<Response<ListBooksResponse>, Status> {
        let books_vec: Vec<Book> = self.books.read().await.values().cloned().collect();
        let total_size = books_vec.len() as i64;
        Ok(Response::new(ListBooksResponse {
            books: books_vec,
            next_page_token: None,
            total_size,
        }))
    }

    async fn create_book(
        &self,
        request: CreateBookRequest,
        _metadata: MetadataMap,
    ) -> Result<Response<Book>, Status> {
        let request = ParsedCreateBookRequest::parse(request)?;
        let id = BookId::new(bomboni::common::id::Id::generate());
        let now = UtcDateTime::now();
        let timestamp: bomboni::proto::google::protobuf::Timestamp = now.into();

        let book = Book {
            name: id.to_name(),
            create_time: Some(timestamp),
            update_time: Some(timestamp),
            delete_time: None,
            deleted: false,
            etag: None,
            display_name: request.display_name,
            author: request.author_id.to_name(),
            isbn: request.isbn,
            description: request.description,
            price_cents: request.price_cents,
            page_count: request.page_count,
        };

        self.books.write().await.insert(id, book.clone());
        Ok(Response::new(book))
    }

    async fn update_book(
        &self,
        request: UpdateBookRequest,
        _metadata: MetadataMap,
    ) -> Result<Response<Book>, Status> {
        let request = ParsedUpdateBookRequest::parse(request)?;
        let mut books = self.books.write().await;
        match books.get_mut(&request.id) {
            Some(book) => {
                if let Some(display_name) = request.display_name {
                    book.display_name = display_name;
                }
                if let Some(author_id) = request.author_id {
                    book.author = author_id.to_name();
                }
                if let Some(isbn) = request.isbn {
                    book.isbn = isbn;
                }
                if let Some(description) = request.description {
                    book.description = description;
                }
                if let Some(price_cents) = request.price_cents {
                    book.price_cents = price_cents;
                }
                if let Some(page_count) = request.page_count {
                    book.page_count = page_count;
                }
                let now = UtcDateTime::now();
                book.update_time = Some(now.into());
                Ok(Response::new(book.clone()))
            }
            None => Err(Status::not_found("Book not found")),
        }
    }

    async fn delete_book(
        &self,
        request: DeleteBookRequest,
        _metadata: MetadataMap,
    ) -> Result<Response<()>, Status> {
        let request = ParsedDeleteBookRequest::parse(request)?;
        let mut books = self.books.write().await;
        match books.remove(&request.id) {
            Some(_) => Ok(Response::new(())),
            None => Err(Status::not_found("Book not found")),
        }
    }
}
